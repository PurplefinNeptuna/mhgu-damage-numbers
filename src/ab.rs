//! r37-projection-ab diagnostics: compare hook 4's captured-matrix projection
//! with the game's own projection call main+0x18a48, made from forge_onUpdate.
//! Display behaviour is unchanged; this module only reads game memory and logs.
//! Static research: work/damage-numbers/projection-call/RESEARCH.md.
use crate::platform::NativeMemory;
use damage_numbers_core::{
    capture::{Memory, Record},
    presentation::{self, Point, View},
    HitKey, Model,
};
// main-relative offsets, verified against the 1.4.0 NSO (see RESEARCH.md).
// PROJECT is also signature window 15, checked before any hook is installed.
const PROJECT: u32 = 0x18a48;
// Cell holding the sFestaCamera instance: 8 records of 0x1b0 bytes; each
// sCamera::Viewport sits at record+0x30, so mpCamera is record+0x54.
const CAMERA_CELL: u32 = 0x184d750;
// sGUI::Instance; the game's five GUI callers pass its byte +0xec as the view.
const GUI_CELL: u32 = 0x212668c;
// Optional view-override object read by the sCamera update (0xb8b178).
const OVERRIDE_CELL: u32 = 0x2126f74;
// uQuestCamera vtable: the same class hook 3/4 capture accepts.
const QUEST_CAMERA_VTABLE: u32 = 0x1730d0c;
const RECORD: u32 = 0x1b0;
// Log caps per process. Forge filters below ERROR by default, so these use
// error! like r29-diagnostics did; none of them is written from a hook.
const CONTEXT_LINES: u32 = 24;
const HIT_LINES: u32 = 64;
const TRACKS: usize = 32;

// Everything the A/B gate in RESEARCH.md depends on, read in onUpdate.
#[derive(Clone, Copy, Default, PartialEq)]
struct Context {
    camera_system: u32,
    view: u8,
    hook_view: u8,
    same_camera: bool,
    mode: u8,
    camera_state: u8,
    region: [i32; 4],
    sampling: u8,
    // Raw bits of sCamera +0x1c/+0x20 (draw offset) and +0x24 (blend factor).
    offset: [u32; 3],
    override_active: u8,
    override_view: u8,
}

#[derive(Clone, Copy, Default)]
struct Track {
    used: bool,
    key: HitKey,
    // Latest native result as a Region fraction; sampled marks a completed call.
    sampled: bool,
    native: Option<Point>,
    samples: u32,
    max: f32,
    sum: f32,
    native_only: u32,
    hook_only: u32,
    first: Option<(Point, Point)>,
}

pub struct Ab {
    tracks: [Track; TRACKS],
    context: Option<Context>,
    context_lines: u32,
    hit_lines: u32,
    // Bit per refusal reason, so each reason is logged once.
    refused: u32,
    update_thread: u32,
    render_thread: u32,
    threads_logged: bool,
    untracked: u32,
}

impl Default for Ab {
    fn default() -> Self {
        Self {
            tracks: [Track::default(); TRACKS],
            context: None,
            context_lines: 0,
            hit_lines: 0,
            refused: 0,
            update_thread: 0,
            render_thread: 0,
            threads_logged: false,
            untracked: 0,
        }
    }
}

// TPIDRURO holds the calling thread's TLS pointer: a per-thread identity that
// is only compared and logged, never dereferenced.
fn thread_id() -> u32 {
    let value: u32;
    // SAFETY: reading the user read-only thread register has no side effects.
    unsafe {
        core::arch::asm!("mrc p15, 0, {}, c13, c0, 3", out(reg) value, options(nomem, nostack, preserves_flags))
    };
    value
}

impl Ab {
    fn refuse(&mut self, bit: u32, reason: &str) -> Option<Context> {
        if self.refused & bit == 0 {
            self.refused |= bit;
            log::error!("[damage-numbers] ab: native projection skipped: {}", reason);
        }
        None
    }
    // Validate every pointer the native function will follow, mirroring the
    // checks its game callers rely on: sCamera, sGUI, view < 8, a live
    // uQuestCamera (state 1 or 2, the states the sCamera update keeps) and
    // its two matrix getters.
    fn read_context(&mut self, memory: &NativeMemory, hook: &Record) -> Option<Context> {
        let base = memory.base();
        let camera_system = memory.cell(CAMERA_CELL);
        if !memory.readable(camera_system, 0xdc0) {
            return self.refuse(1, "camera system");
        }
        let gui = memory.cell(GUI_CELL);
        if !memory.readable(gui + 0xec, 1) {
            return self.refuse(2, "gui");
        }
        let view = memory.byte(gui + 0xec);
        if view >= 8 {
            return self.refuse(4, "view index");
        }
        let record = camera_system + view as u32 * RECORD;
        let camera = memory.word(record + 0x54);
        if !memory.readable(camera, 0x70) || memory.word(camera) != base.wrapping_add(QUEST_CAMERA_VTABLE) {
            return self.refuse(8, "no quest camera");
        }
        let camera_state = (memory.word(camera + 0xc) & 7) as u8;
        if !(1..=2).contains(&camera_state) {
            return self.refuse(16, "camera state");
        }
        let table = memory.word(camera);
        if !memory.readable(table + 0x4c, 8) {
            return self.refuse(32, "camera vtable");
        }
        // Hook 3's subject is the Viewport it drew (record + 0x30).
        let hook_view = hook
            .monster
            .checked_sub(camera_system + 0x30)
            .filter(|offset| offset % RECORD == 0 && offset / RECORD < 8)
            .map_or(255, |offset| (offset / RECORD) as u8);
        let region = core::array::from_fn(|i| memory.word(record + 0x68 + 4 * i as u32) as i32);
        let override_object = memory.cell(OVERRIDE_CELL);
        let (override_active, override_view) = if memory.readable(override_object, 0x7b0 + 8 * 0x40) {
            (memory.byte(override_object + 0x7d), memory.byte(override_object + 0x7b0 + view as u32 * 0x40))
        } else {
            (255, 255)
        };
        Some(Context {
            camera_system,
            view,
            hook_view,
            same_camera: hook.actor == camera,
            mode: memory.byte(record + 0x63),
            camera_state,
            region,
            sampling: memory.byte(camera_system + 0x59),
            offset: [0x1c, 0x20, 0x24].map(|at| memory.word(camera_system + at)),
            override_active,
            override_view,
        })
    }
    fn track(&mut self, key: HitKey) -> Option<&mut Track> {
        let index = self
            .tracks
            .iter()
            .position(|t| t.used && t.key == key)
            .or_else(|| self.tracks.iter().position(|t| !t.used));
        let Some(index) = index else {
            self.untracked = self.untracked.wrapping_add(1);
            return None;
        };
        let track = &mut self.tracks[index];
        if !track.used {
            *track = Track { used: true, key, ..Track::default() };
        }
        Some(track)
    }
    /// forge_onUpdate: project every live number through main+0x18a48.
    /// Only runs when all hooks and signature windows (including 0x18a48) passed.
    pub fn sample(&mut self, model: &Model, player: u32, hook: &Record) {
        self.update_thread = thread_id();
        if player == 0 || model.count == 0 {
            return;
        }
        let memory = NativeMemory;
        let Some(context) = self.read_context(&memory, hook) else {
            return;
        };
        if self.context != Some(context) && self.context_lines < CONTEXT_LINES {
            self.context_lines += 1;
            let o = context.offset.map(f32::from_bits);
            log::error!(
                "[damage-numbers] ab: context view={} hook_view={} same_camera={} mode={} state={} region={},{},{},{} sampling={} offset={:.3},{:.3} blend={:.3} override={},{}",
                context.view, context.hook_view, context.same_camera, context.mode, context.camera_state,
                context.region[0], context.region[1], context.region[2], context.region[3],
                context.sampling, o[0], o[1], o[2], context.override_active, context.override_view
            );
        }
        self.context = Some(context);
        // SAFETY: window 15 matched the original bytes of this ARM function before
        // install; read_context checked every pointer it follows. Its signature is
        // void(float out[4], sCamera*, const float world[3], u32 view), and it writes
        // only out. Called on the game update thread, like its five native callers.
        let project: unsafe extern "C" fn(*mut f32, u32, *const f32, u32) =
            unsafe { core::mem::transmute(memory.base().wrapping_add(PROJECT) as usize) };
        for event in &model.numbers[..model.count] {
            let Some(position) = event.position else { continue };
            let world = [position.x, position.y, position.z, 0.0];
            let mut out = [0.0f32; 4];
            // SAFETY: see above; out and world are live stack arrays of four floats.
            unsafe { project(out.as_mut_ptr(), context.camera_system, world.as_ptr(), context.view as u32) };
            let native = presentation::native_fraction(out, context.region);
            if let Some(track) = self.track(event.key) {
                track.sampled = true;
                track.native = native;
            }
        }
    }
    /// Render thread, after model.update: compare against hook 4's projection
    /// for the same numbers, then log and free tracks whose number expired.
    pub fn compare(&mut self, model: &Model, view: Option<&View>, size: [f32; 2]) {
        self.render_thread = thread_id();
        if !self.threads_logged && self.update_thread != 0 {
            self.threads_logged = true;
            log::error!(
                "[damage-numbers] ab: threads update={:08x} render={:08x}",
                self.update_thread, self.render_thread
            );
        }
        let live = &model.numbers[..model.count];
        for index in 0..TRACKS {
            let track = self.tracks[index];
            if track.used && !live.iter().any(|e| e.key == track.key) {
                self.finish(index, size);
            }
        }
        let Some(view) = view else { return };
        for event in live {
            let Some(position) = event.position else { continue };
            let Some(track) = self.tracks.iter_mut().find(|t| t.used && t.key == event.key) else {
                continue;
            };
            if !track.sampled {
                continue;
            }
            let hook = presentation::project(position, view).map(|p| Point {
                x: p.x / size[0],
                y: p.y / size[1],
            });
            match (track.native, hook) {
                (Some(native), Some(hook)) => {
                    // Larger per-axis difference in display pixels (no sqrt in no_std core).
                    let delta = ((native.x - hook.x) * size[0]).abs().max(((native.y - hook.y) * size[1]).abs());
                    track.samples += 1;
                    track.sum += delta;
                    track.max = track.max.max(delta);
                    track.first.get_or_insert((native, hook));
                }
                (Some(_), None) => track.native_only += 1,
                (None, Some(_)) => track.hook_only += 1,
                (None, None) => {}
            }
        }
    }
    fn finish(&mut self, index: usize, size: [f32; 2]) {
        let track = self.tracks[index];
        self.tracks[index].used = false;
        if track.samples + track.native_only + track.hook_only == 0 || self.hit_lines >= HIT_LINES {
            return;
        }
        self.hit_lines += 1;
        let (native, hook) = track.first.unwrap_or_default();
        log::error!(
            "[damage-numbers] ab: hit {} samples={} max_axis={:.2}px mean_axis={:.2}px native_only={} hook_only={} first native={:.1},{:.1} hook={:.1},{:.1} size={}x{} untracked={}",
            track.key.serial, track.samples, track.max,
            if track.samples > 0 { track.sum / track.samples as f32 } else { 0.0 },
            track.native_only, track.hook_only,
            native.x * size[0], native.y * size[1], hook.x * size[0], hook.y * size[1],
            size[0], size[1], self.untracked
        );
    }
}
