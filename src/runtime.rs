//! Callback-owned display state and independent producer mailboxes. Native hooks never lock the display mutex.
use crate::{
    config,
    hooks::{self, Installed},
    platform::{self, NativeMemory},
    render::Resources,
};
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU32, Ordering},
};
use damage_numbers_core::{
    capture::{CaptureState, Memory, RawFrame, Record},
    mailbox::{Cursor, Mailbox},
    presentation::{self, DrawNumber, Placement},
    *,
};
use forge::os::{lock::ScopedLock, mutex::Mutex, once_lock::OnceLock, tls::ThreadLocal};
static TLS: OnceLock<ThreadLocal<CaptureState>> = OnceLock::new();
static CLAIMED: AtomicU32 = AtomicU32::new(0);
static CONSUMER: AtomicU32 = AtomicU32::new(0);
static MAILBOXES: [Mailbox; 16] = [const { Mailbox::new() }; 16];
static STATE: OnceLock<Shared> = OnceLock::new();
struct Shared {
    lock: Mutex,
    value: UnsafeCell<Consumer>,
}
// SAFETY: value is accessed only under lock. ImGui work is further confined to Forge's elected render thread.
unsafe impl Sync for Shared {}
// SAFETY: Shared remains in its process-lifetime cell; moving its initialized state does not expose raw resource pointers.
unsafe impl Send for Shared {}
struct Consumer {
    installed: Option<Installed>,
    clock: Option<(u64, u32)>,
    model: Model,
    placement: Placement,
    camera: Record,
    player: u32,
    cursors: [Cursor; 16],
    next: usize,
    resources: Resources,
    loaded: bool,
    retry: u64,
    feedback: alloc::string::String,
}
fn with_state<R>(f: impl FnOnce(&mut Consumer) -> R) -> Option<R> {
    let shared = STATE.get()?;
    let _guard = ScopedLock::new(&shared.lock);
    // SAFETY: every reference to this cell is obtained under its single mutex; hooks never access the cell.
    Some(f(unsafe { &mut *shared.value.get() }))
}
fn create_capture() -> CaptureState {
    let index = CLAIMED.fetch_add(1, Ordering::SeqCst) as usize;
    CaptureState::new((index < 16).then_some(index))
}
pub fn start() {
    TLS.get_or_init(|| ThreadLocal::new(create_capture));
    let clock = platform::clock();
    let installed = if clock.is_some() {
        match Installed::install() {
            Ok(value) => {
                log::info!(
                    "[damage-numbers-rs] r1 loaded: {} Forge hooks",
                    value.count()
                );
                Some(value)
            }
            Err(reason) => {
                log::warn!("[damage-numbers-rs] {}", reason);
                None
            }
        }
    } else {
        log::warn!("[damage-numbers-rs] invalid native clock");
        None
    };
    STATE.get_or_init(|| Shared {
        lock: Mutex::new(),
        value: UnsafeCell::new(Consumer {
            installed,
            clock,
            model: Model::default(),
            placement: Placement::default(),
            camera: Record::default(),
            player: 0,
            cursors: core::array::from_fn(|_| Cursor::default()),
            next: 0,
            resources: Resources::default(),
            loaded: false,
            retry: 0,
            feedback: alloc::string::String::new(),
        }),
    });
}
fn render_thread() -> bool {
    let Some(tls) = TLS.get() else {
        return false;
    };
    let id = tls.with(|state| state.slot.map(|slot| slot as u32 + 1));
    let Some(id) = id else {
        return false;
    };
    let _ = CONSUMER.compare_exchange(0, id, Ordering::SeqCst, Ordering::SeqCst);
    CONSUMER.load(Ordering::SeqCst) == id
}
impl Consumer {
    fn load_config(&mut self) {
        if self.loaded {
            return;
        }
        let now = platform::tick();
        if now < self.retry {
            return;
        }
        self.retry = now + self.clock.map_or(19_200_000, |(_, f)| f) as u64;
        if !platform::storage_ready() {
            return;
        }
        self.loaded = true;
        match config::load() {
            Ok(settings) => {
                self.model.configure(settings);
            }
            Err(reason) => {
                log::warn!("[damage-numbers-rs] config: {}", reason);
            }
        }
    }
    fn reset(&mut self) {
        let mut epoch = self.model.epoch.wrapping_add(1);
        if epoch == 0 {
            epoch = 1;
        }
        self.model.reset(epoch);
        self.placement.reset();
    }
    fn consume(&mut self) {
        if self.installed.is_none() {
            return;
        }
        self.load_config();
        let memory = NativeMemory;
        let player = memory.local_player();
        let now = platform::tick();
        let Some((origin, frequency)) = self.clock else {
            return;
        };
        if player != self.player {
            self.player = player;
            self.camera = Record::default();
            self.reset();
        }
        let (mut empty, mut hits, mut records) = (0, 0, 0);
        while empty < 16 && hits < 8 && records < 64 {
            let slot = self.next;
            self.next = (slot + 1) % 16;
            let Some(record) = MAILBOXES[slot].read(&mut self.cursors[slot]) else {
                empty += 1;
                continue;
            };
            empty = 0;
            records += 1;
            match record.kind {
                1 => {
                    hits += 1;
                    let source = match record.feedback[0].reserved {
                        0 => Some(Source::Attack),
                        1 => Some(Source::Monster),
                        2 => Some(Source::Object),
                        _ => None,
                    };
                    let owner = match record.actor {
                        0 => Owner::LocalPlayer,
                        1 => Owner::LocalPalico,
                        2 => Owner::RemotePlayer,
                        3 => Owner::RemotePalico,
                        _ => Owner::Unknown,
                    };
                    if player != 0
                        && record.player == player
                        && fresh(record.tick, now, frequency, self.model.settings.lifetime)
                    {
                        if let (Some(source), Some(captured_at)) =
                            (source, seconds(record.tick, origin, frequency))
                        {
                            self.model.capture(Event {
                                key: HitKey {
                                    epoch: self.model.epoch,
                                    serial: record.serial,
                                    target: record.monster,
                                    application: slot as u32,
                                },
                                position: record.position,
                                total_damage: record.damage,
                                physical_hzv: record.hzv as i16,
                                source,
                                owner,
                                critical: critical_outcome(record.critical as u8),
                                bounced: false,
                                captured_at,
                            });
                        }
                    }
                }
                2 => {
                    if player != 0
                        && record.hzv == 1
                        && record.player == player
                        && fresh(record.tick, now, frequency, 0.1)
                    {
                        self.model.bounce(HitKey {
                            epoch: self.model.epoch,
                            serial: record.damage as u32,
                            target: record.monster,
                            application: slot as u32,
                        });
                    }
                }
                3 => {
                    if player != 0 && record.player == player {
                        if self.camera.actor != 0
                            && (self.camera.actor != record.actor
                                || self.camera.monster != record.monster
                                || self.camera.rect != record.rect)
                        {
                            self.reset();
                        }
                        self.camera = record;
                    }
                }
                _ => {}
            }
        }
    }
}
pub fn update() {
    with_state(|state| state.load_config());
}
pub fn init_imgui() {
    with_state(|state| {
        state.resources.initialize();
        state.placement.reset();
    });
}
pub fn render() {
    if !render_thread() {
        return;
    }
    with_state(|state| {
        state.consume();
        if state.installed.is_none() {
            return;
        }
        let Some((origin, frequency)) = state.clock else {
            return;
        };
        let now = platform::tick();
        let size = forge::imgui::get_io().DisplaySize;
        let view = if state.player != 0
            && state.camera.player == state.player
            && fresh(state.camera.tick, now, frequency, 0.2)
        {
            presentation::native_camera(
                &state.camera.view,
                &state.camera.projection,
                state.camera.rect,
                size.x,
                size.y,
            )
        } else {
            None
        };
        let time = seconds(now, origin, frequency).unwrap_or(f32::NAN);
        state.model.update(time, view.is_some());
        let Some(view) = view else {
            state.placement.reset();
            return;
        };
        let mut numbers = [DrawNumber::default(); 128];
        let count = presentation::layout(
            &state.model,
            &view,
            time,
            &mut numbers,
            &mut state.placement,
            |text, pixels, style| state.resources.measure(text, pixels, style),
        );
        state.resources.draw(&view, &numbers[..count]);
    });
}
/// Closing the settings tree affects only this callback. The overlay keeps processing through free_render.
pub fn panel() {
    if !render_thread() {
        return;
    }
    if !forge::imgui::tree_node(c"Damage numbers (Rust)##damage-numbers-rs") {
        return;
    }
    struct Tree;
    impl Drop for Tree {
        fn drop(&mut self) {
            forge::imgui::tree_pop();
        }
    }
    let _tree = Tree;
    with_state(|state| {
        state.consume();
        let mut settings = state.model.settings;
        let mut changed = forge::imgui::checkbox(
            c"Show damage numbers##damage-numbers-rs",
            &mut settings.enabled,
        );
        let preview = match settings.style {
            Style::Wilds => c"Wilds",
            Style::Classic => c"Classic",
            Style::Rise => c"Rise",
        };
        if forge::imgui::begin_combo(
            c"Style##damage-numbers-rs",
            preview,
            forge::sys::imgui::ImGuiComboFlags::None,
        ) {
            for (style, name) in [
                (Style::Wilds, c"Wilds"),
                (Style::Classic, c"Classic"),
                (Style::Rise, c"Rise"),
            ] {
                if forge::imgui::selectable(
                    name,
                    settings.style == style,
                    forge::sys::imgui::ImGuiSelectableFlags::None,
                    forge::sys::imgui::ImVec2::new(0.0, 0.0),
                ) {
                    settings.style = style;
                    changed = true;
                }
            }
            forge::imgui::end_combo();
        }
        changed |= forge::imgui::slider_float(
            c"Number size##damage-numbers-rs",
            &mut settings.scale,
            0.5,
            3.0,
            forge::sys::imgui::ImGuiSliderFlags::AlwaysClamp,
        );
        changed |= forge::imgui::slider_float(
            c"Duration##damage-numbers-rs",
            &mut settings.lifetime,
            0.4,
            2.0,
            forge::sys::imgui::ImGuiSliderFlags::AlwaysClamp,
        );
        if forge::imgui::button(
            c"Restore defaults##damage-numbers-rs",
            forge::sys::imgui::ImVec2::new(0.0, 0.0),
        ) {
            settings = Settings::default();
            changed = true;
        }
        if changed && state.model.configure(settings) {
            state.loaded = true;
            state.feedback = alloc::string::String::from("Settings changed");
        }
        if forge::imgui::button(
            c"Save settings##damage-numbers-rs",
            forge::sys::imgui::ImVec2::new(0.0, 0.0),
        ) {
            state.loaded = true;
            match config::save(state.model.settings) {
                Ok(()) => {
                    state.feedback = alloc::string::String::from(
                        "Saved damage_numbers_rs.ini in MHGU's save folder",
                    );
                    log::info!("[damage-numbers-rs] settings saved");
                }
                Err(reason) => {
                    state.feedback = alloc::format!("Save failed: {}", reason);
                    log::warn!("[damage-numbers-rs] {}", state.feedback);
                }
            }
        }
        forge::imgui::text(&state.feedback);
    });
}
fn original(kind: u32) -> u32 {
    hooks::ORIGINALS
        .get(kind as usize)
        .map_or(0, |cell| cell.load(Ordering::Acquire))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dn_gateway_enter(kind: u32, frame: *mut RawFrame) -> u32 {
    let continuation = original(kind);
    if !hooks::ACTIVE.load(Ordering::Acquire) {
        return continuation;
    }
    // SAFETY: gateway.S passes its aligned 328-byte stack image; the borrow ends before assembly resumes.
    let saved = unsafe { &mut *frame };
    let sp = frame as u32 + 328;
    if let Some(tls) = TLS.get() {
        tls.with_mut(|state| {
            if state.enter(kind, saved, sp, &NativeMemory) {
                saved.r[13] = dn_gateway_after as *const () as u32;
            }
        });
    }
    continuation
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dn_gateway_leave(frame: *mut RawFrame) -> u32 {
    // SAFETY: same gateway stack contract as enter; LR substitution guarantees an existing thread-local scope.
    let saved = unsafe { &mut *frame };
    let sp = frame as u32 + 328;
    let continuation = TLS.get().and_then(|tls| {
        tls.with_mut(|state| {
            state.slot.and_then(|slot| {
                state.leave(
                    saved,
                    sp,
                    hooks::ACTIVE.load(Ordering::Acquire),
                    &NativeMemory,
                    &MAILBOXES[slot],
                )
            })
        })
    });
    let Some(caller) = continuation else {
        panic!("damage-numbers-rs gateway return scope mismatch");
    };
    if saved.r[13] == dn_gateway_after as *const () as u32 {
        saved.r[13] = caller;
    }
    caller
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dn_gateway_tap(kind: u32, frame: *mut RawFrame) -> u32 {
    let continuation = original(kind);
    if hooks::ACTIVE.load(Ordering::Acquire) {
        if let Some(tls) = TLS.get() {
            // SAFETY: the gateway owns this initialized frame until the callback returns.
            let saved = unsafe { &*frame };
            tls.with_mut(|state| {
                if let Some(slot) = state.slot {
                    state.tap(kind, saved, &NativeMemory, &MAILBOXES[slot]);
                }
            });
        }
    }
    continuation
}
unsafe extern "C" {
    fn dn_gateway_after();
}
