//! Callback-owned display state and independent producer mailboxes. Native hooks never lock the display mutex.
use crate::{
    config,
    diagnostics::Diagnostics,
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
// OnceLock initializes the shared owner once. ThreadLocal gives each native
// thread its own capture stack, so nested callbacks cannot mix return chains.
// MAILBOXES is separate static storage; TLS destruction never drops its records.
static TLS: OnceLock<ThreadLocal<CaptureState>> = OnceLock::new();
static CLAIMED: AtomicU32 = AtomicU32::new(0);
static CONSUMER: AtomicU32 = AtomicU32::new(0);
static MAILBOXES: [Mailbox; 16] = [const { Mailbox::new() }; 16];
static STATE: OnceLock<Shared> = OnceLock::new();
// Mutex protects the Consumer, while UnsafeCell permits mutable access
// through a shared static reference. with_state is the sole access gate.
// Native capture uses TLS/mailboxes and never takes this display lock.
struct Shared {
    lock: Mutex,
    value: UnsafeCell<Consumer>,
}
// SAFETY: value is accessed only under lock. ImGui work is further confined to Forge's elected render thread.
unsafe impl Sync for Shared {}
// SAFETY: Shared remains in its process-lifetime cell; moving its initialized state does not expose raw resource pointers.
unsafe impl Send for Shared {}
// The callback side owns model/history, mailbox cursors, placement cache,
// settings-load state and ImGui resources. None is borrowed by a game hook.
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
    diagnostics: Diagnostics,
}
// Call the closure while holding Forge's scoped mutex guard. Both the mutable
// Consumer borrow and lock end when the closure returns, including early exits.
fn with_state<R>(f: impl FnOnce(&mut Consumer) -> R) -> Option<R> {
    let shared = STATE.get()?;
    let _guard = ScopedLock::new(&shared.lock);
    // SAFETY: every reference to this cell is obtained under its single mutex; hooks never access the cell.
    Some(f(unsafe { &mut *shared.value.get() }))
}
fn create_capture() -> CaptureState {
    // Claim each mailbox at most once for the process lifetime. Slots are never
    // recycled after thread exit, because the consumer may still have unread data.
    // After 16 claims, new thread states pass through without capturing.
    let index = CLAIMED.fetch_add(1, Ordering::SeqCst) as usize;
    CaptureState::new((index < 16).then_some(index))
}
// Create TLS before publishing hooks. Keep the installation result in STATE
// so successful patches/trampolines remain owned across all later callbacks.
pub fn start() {
    TLS.get_or_init(|| ThreadLocal::new(create_capture));
    let clock = platform::clock();
    let installed = if clock.is_some() {
        match Installed::install() {
            Ok(value) => {
                log::error!("[DN-DIAG] r29-diagnostics loaded: {} hooks; camera limit=16, hit limit=32; Cyan=screen centre, Magenta=projected hit, Yellow=fallback; diagnostic ERROR lines are expected", value.count());
                Some(value)
            }
            Err(reason) => {
                log::warn!("[damage-numbers] {}", reason);
                None
            }
        }
    } else {
        log::warn!("[damage-numbers] invalid native clock");
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
            diagnostics: Diagnostics::default(),
        }),
    });
}
// The first eligible ImGui callback elects one consumer thread by its mailbox
// slot+1; zero is the unclaimed sentinel. Other threads skip UI/drawing so
// raw context resources and cursor ownership remain on that elected thread.
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
        // Check storage readiness at most once per second. Set loaded only once the
        // storage types exist, then make one load attempt instead of retrying an I/O
        // failure every frame. Session settings remain available after a failure.
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
                log::warn!("[damage-numbers] config: {}", reason);
            }
        }
    }
    // Advance display epoch and discard old placements/events together. Keep zero
    // reserved by skipping it when the u32 epoch wraps.
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
        // A changed local-player identity ends the display session: camera and queued
        // hits from the previous player must not appear for this one.
        if player != self.player {
            self.player = player;
            self.camera = Record::default();
            self.reset();
        }
        let (mut empty, mut hits, mut records) = (0, 0, 0);
        // Round-robin across all producer mailboxes. Stop after a full empty sweep,
        // eight damage records or 64 total records, bounding callback work and avoiding
        // one busy producer starving camera/feedback records from other threads.
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
                // Damage: decode the numeric source/owner fields, verify player identity
                // and freshness, then construct a pointer-free Event for the model.
                // The mailbox slot becomes application in HitKey to disambiguate serials.
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
                                position: record.position.valid().then_some(record.position),
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
                // Feedback: hzv=1 denotes a successful native bounce match, and damage
                // contains that earlier hit's serial. Apply it only within 0.1 seconds.
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
                // Camera: actor is camera identity and monster is viewport identity in
                // this record variant. Identity/rectangle changes start a new display epoch.
                3 => {
                    self.diagnostics.camera(&record);
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
        let io = forge::imgui::get_io();
        let size = io.DisplaySize;
        let scale = io.DisplayFramebufferScale;
        // ImGui owns this viewport for the current callback. Copy its values only.
        let viewport = unsafe { forge::sys::imgui::igGetMainViewport().as_ref() };
        let bounds = viewport.map_or([f32::NAN; 4], |v| [v.Pos.x, v.Pos.y, v.Size.x, v.Size.y]);
        state.diagnostics.display([size.x, size.y, scale.x, scale.y, bounds[0], bounds[1], bounds[2], bounds[3]]);
        // Only a camera sample for this player, younger than 0.2 seconds, may drive
        // world projection. Missing/stale cameras keep numbers at screen centre.
        let native_view = if state.player != 0
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
        let native_camera_used = native_view.is_some();
        let view = native_view.or_else(|| presentation::View::screen(size.x, size.y));
        let time = seconds(now, origin, frequency).unwrap_or(f32::NAN);
        state.model.update(time, view.is_some());
        let Some(view) = view else {
            state.placement.reset();
            return;
        };
        // layout converts world positions into copied draw commands using real glyph
        // metrics; draw then submits them to ImGui. This frame buffer has fixed capacity.
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
        let draw = forge::imgui::DrawList::background();
        let centre = presentation::Point { x: view.x + view.width * 0.5, y: view.y + view.height * 0.5 };
        diagnostic_cross(draw, centre, 0xffffff00);
        draw.add_text(forge::sys::imgui::ImVec2::new(centre.x + 10.0, centre.y + 10.0), 0xffffff00, "DN diag centre");
        for &event in &state.model.numbers[..state.model.count] {
            let number = numbers[..count].iter().find(|number| number.key == event.key);
            state.diagnostics.hit(event, &state.camera, &view, native_camera_used, now, number);
            if number.is_some() {
                let projected = event.position.and_then(|p| presentation::project(p, &view));
                diagnostic_cross(draw, projected.unwrap_or(centre), if projected.is_some() { 0xffff00ff } else { 0xff00ffff });
            }
        }
    });
}
// These fixed-size reference marks bypass number animation, glyph metrics and
// overlap placement. Cyan checks Forge coordinates; Magenta/Yellow check the anchor.
fn diagnostic_cross(draw: forge::imgui::DrawList, point: presentation::Point, color: u32) {
    use forge::sys::imgui::ImVec2;
    draw.add_line(ImVec2::new(point.x - 8.0, point.y), ImVec2::new(point.x + 8.0, point.y), color, 2.0);
    draw.add_line(ImVec2::new(point.x, point.y - 8.0), ImVec2::new(point.x, point.y + 8.0), color, 2.0);
}
/// Closing the settings tree affects only this callback. The overlay keeps processing through free_render.
pub fn panel() {
    if !render_thread() {
        return;
    }
    if !forge::imgui::tree_node(c"Damage Numbers##damage-numbers") {
        return;
    }
    // This small Drop guard pairs TreePop with the successful TreeNode even if
    // a closure returns early. The closed row never suppresses free-render work.
    struct Tree;
    impl Drop for Tree {
        fn drop(&mut self) {
            forge::imgui::tree_pop();
        }
    }
    let _tree = Tree;
    with_state(|state| {
        state.consume();
        // Widgets edit a copy, then configure validates and applies it as one snapshot.
        // The ## suffixes give each setting a stable plugin-specific ImGui identity.
        let mut settings = state.model.settings;
        let mut changed = forge::imgui::checkbox(
            c"Show damage numbers##damage-numbers",
            &mut settings.enabled,
        );
        let preview = match settings.style {
            Style::Wilds => c"Wilds",
            Style::Classic => c"Classic",
            Style::Rise => c"Rise",
        };
        if forge::imgui::begin_combo(
            c"Style##damage-numbers",
            preview,
            forge::sys::imgui::ImGuiComboFlags::None,
        ) {
            for (style, name) in [
                (Style::Wilds, c"Wilds"),
                (Style::Rise, c"Rise"),
                (Style::Classic, c"Classic"),
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
            c"Number size##damage-numbers",
            &mut settings.scale,
            0.5,
            3.0,
            forge::sys::imgui::ImGuiSliderFlags::AlwaysClamp,
        );
        changed |= forge::imgui::slider_float(
            c"Duration##damage-numbers",
            &mut settings.lifetime,
            0.4,
            2.0,
            forge::sys::imgui::ImGuiSliderFlags::AlwaysClamp,
        );
        if forge::imgui::button(
            c"Restore defaults##damage-numbers",
            forge::sys::imgui::ImVec2::new(0.0, 0.0),
        ) {
            settings = Settings::default();
            changed = true;
        }
        // Mark load complete after a user edit so a delayed disk load cannot replace
        // what the player just chose. Reset follows this same in-memory path.
        if changed && state.model.configure(settings) {
            state.loaded = true;
            state.feedback = alloc::string::String::from("Settings changed");
        }
        if forge::imgui::button(
            c"Save settings##damage-numbers",
            forge::sys::imgui::ImVec2::new(0.0, 0.0),
        ) {
            state.loaded = true;
            match config::save(state.model.settings) {
                Ok(()) => {
                    state.feedback = alloc::string::String::from(
                        "Saved damage_numbers.ini in MHGU's save folder",
                    );
                    log::info!("[damage-numbers] settings saved");
                }
                Err(reason) => {
                    state.feedback = alloc::format!("Save failed: {}", reason);
                    log::warn!("[damage-numbers] {}", state.feedback);
                }
            }
        }
        forge::imgui::text(&state.feedback);
    });
}
// Each gateway needs the corresponding trampoline even when capture is off.
// Acquire pairs with install's Release store of the initialized address.
fn original(kind: u32) -> u32 {
    hooks::ORIGINALS
        .get(kind as usize)
        .map_or(0, |cell| cell.load(Ordering::Acquire))
}
// Assembly calls this before the original function. Returning a trampoline
// address tells assembly where to branch after restoring native registers.
// Requirements: kind identifies an installed around hook; frame points to its
// complete aligned RawFrame and stays live until this helper returns.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dn_gateway_enter(kind: u32, frame: *mut RawFrame) -> u32 {
    let continuation = original(kind);
    if !hooks::ACTIVE.load(Ordering::Acquire) {
        return continuation;
    }
    // SAFETY: gateway.S passes its aligned 328-byte stack image; the borrow ends before assembly resumes.
    let saved = unsafe { &mut *frame };
    // frame is the gateway's lowered SP. Recover entry SP before passing it to
    // capture, which reads native stack arguments and later verifies return identity.
    let sp = frame as u32 + 328;
    if let Some(tls) = TLS.get() {
        tls.with_mut(|state| {
            if state.enter(kind, saved, sp, &NativeMemory) {
                // Replace the saved LR only after recording its original caller in Scope.
                // The original now returns through our after gateway, which restores that caller.
                // The TLS mutable borrow ends here before assembly executes the trampoline.
                saved.r[13] = dn_gateway_after as *const () as u32;
            }
        });
    }
    continuation
}
// The after gateway calls this with the native return registers saved.
// It pops one scope, publishes finalized evidence and returns the exact caller.
// Requirements: the RawFrame is live/aligned and this thread has the matching
// scope recorded when enter substituted the native return address.
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
        panic!("damage-numbers gateway return scope mismatch");
    };
    // If native LR still contains our substituted address, restore the real
    // caller in the saved register image as well as returning it to assembly.
    if saved.r[13] == dn_gateway_after as *const () as u32 {
        saved.r[13] = caller;
    }
    caller
}
// Camera/melee taps read transient registers before displaced instructions.
// They do not change LR or create a return scope. Requirements: kind is an
// installed tap and frame is its live, complete aligned register snapshot.
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
