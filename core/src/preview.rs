//! The Mod Options preview: frame geometry, world anchors and the looping example hits. Pure; the
//! plugin draws the frame and lays the hits out with the production `layout` and renderer.
use crate::presentation::{Placement, Point, Rect, View, project};
use crate::{Critical, Event, HitKey, Model, Owner, Preview as Mode, Settings, Source, Vec3};

/// The frame as fractions of the visible view: anchored middle-left with a 2.5% margin, 57% wide
/// and 38% tall, so it ends before the Options window (from about 64% of the width at 1080p).
pub fn frame(v: &View) -> Rect {
    let left = v.x + v.width * 0.025;
    let top = v.y + v.height * 0.31;
    Rect { left, top, right: left + v.width * 0.57, bottom: top + v.height * 0.38 }
}
/// The three example spots: 1/6, 1/2 and 5/6 of the frame's width, at its vertical centre.
pub fn spots(f: Rect) -> [Point; 3] {
    let y = (f.top + f.bottom) * 0.5;
    let x = |share: f32| f.left + (f.right - f.left) * share;
    [Point { x: x(1.0 / 6.0), y }, Point { x: x(0.5), y }, Point { x: x(5.0 / 6.0), y }]
}

/// Clip w (view depth) at which a spot is anchored in the world.
// shortcut: MHGU's world unit is not measured; 600 is unverified. Tune it in game if the spots
// slide too much or too little when the camera moves sideways.
pub const DEPTH: f32 = 600.0;

/// The world point at clip depth `depth` that projects to screen point `p`: clip x = ndc_x * w,
/// clip y = ndc_y * w and w = depth are three linear equations in x, y, z, solved by Cramer's rule.
pub fn unproject(p: Point, v: &View, depth: f32) -> Option<Vec3> {
    if !(v.width > 0.0 && v.height > 0.0 && depth > 0.0) {
        return None;
    }
    let m = &v.world_to_clip;
    let nx = (p.x - v.x) / v.width * 2.0 - 1.0;
    let ny = 1.0 - (p.y - v.y) / v.height * 2.0;
    let row = |r: usize, n: f32| [m[r * 4] - n * m[12], m[r * 4 + 1] - n * m[13], m[r * 4 + 2] - n * m[14], m[r * 4 + 3] - n * m[15]];
    let (a, b) = (row(0, nx), row(1, ny));
    let c = [m[12], m[13], m[14], m[15] - depth];
    let det3 = |x: [f32; 3], y: [f32; 3], z: [f32; 3]| {
        x[0] * (y[1] * z[2] - y[2] * z[1]) - x[1] * (y[0] * z[2] - y[2] * z[0]) + x[2] * (y[0] * z[1] - y[1] * z[0])
    };
    let (ca, cb, cc) = ([a[0], b[0], c[0]], [a[1], b[1], c[1]], [a[2], b[2], c[2]]);
    let rhs = [-a[3], -b[3], -c[3]];
    let det = det3(ca, cb, cc);
    if !det.is_finite() || det.abs() < 1e-12 {
        return None;
    }
    let point = Vec3 { x: det3(rhs, cb, cc) / det, y: det3(ca, rhs, cc) / det, z: det3(ca, cb, rhs) / det };
    point.valid().then_some(point)
}
/// A view with no camera: an identity matrix, so a spot anchored at (ndc x, ndc y, 0.5) stays at
/// its screen point. `layout` treats any finite non-zero matrix as a camera.
pub fn flat_view(v: &View) -> View {
    let mut flat = *v;
    flat.world_to_clip = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.];
    flat
}
fn flat_anchor(p: Point, v: &View) -> Vec3 {
    Vec3 { x: (p.x - v.x) / v.width * 2.0 - 1.0, y: 1.0 - (p.y - v.y) / v.height * 2.0, z: 0.5 }
}
fn has_camera(v: &View) -> bool {
    v.world_to_clip.iter().all(|x| x.is_finite()) && v.world_to_clip.iter().any(|&x| x != 0.0)
}

/// Seconds after a loop's start at which each source gets a hit.
pub fn spawn_times(mode: Mode) -> &'static [f32] {
    match mode {
        Mode::Single => &[0.0],
        Mode::Burst => &[0.0, 0.15, 0.3, 0.45, 0.6],
        Mode::Multi => &[0.0; 5],
    }
}
/// The last spawn, the hits' lifetime, then 0.5 s with nothing on screen.
pub fn loop_length(mode: Mode, lifetime: f32) -> f32 {
    spawn_times(mode).last().copied().unwrap_or(0.0) + lifetime + 0.5
}

/// A hit type: physical hitzone value (45 and up is a weak point), bounce and critical.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kind {
    pub hzv: i16,
    pub bounced: bool,
    pub critical: Critical,
}
const HIGH: i16 = 60;
const LOW: i16 = 20;
/// Without random types: left weak point with positive critical, middle low hitzone with
/// negative critical, right a bounce with no critical.
pub const NORMAL: [Kind; 3] = [
    Kind { hzv: HIGH, bounced: false, critical: Critical::Positive },
    Kind { hzv: LOW, bounced: false, critical: Critical::Negative },
    Kind { hzv: LOW, bounced: true, critical: Critical::None },
];

struct Run {
    start: f32,
    length: f32,
    mode: Mode,
    spawned: usize,
    anchors: [Vec3; 3],
    flat: bool,
}

/// The preview's own hits and placement, separate from real hits' bookkeeping.
pub struct PreviewState {
    pub model: Model,
    pub placement: Placement,
    run: Option<Run>,
    serial: u32,
    random: u32,
}
impl Default for PreviewState {
    fn default() -> Self {
        let mut model = Model::default();
        model.reset(1);
        Self { model, placement: Placement::default(), run: None, serial: 0, random: 0x9e37_79b9 }
    }
}
impl PreviewState {
    /// The page left the screen: drop every hit so the next opening starts a fresh loop.
    pub fn stop(&mut self) {
        if self.run.take().is_some() || self.model.count > 0 || self.model.pending() > 0 {
            self.model.reset(1);
            self.placement.reset();
        }
    }
    // xorshift32: enough variety for example damage; not for anything else.
    fn next(&mut self, below: u32) -> u32 {
        let mut x = self.random;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.random = x;
        x % below
    }
    /// One frame at time `now` (seconds). Returns the view to lay the preview out with: `view`, or
    /// its flat form when the loop was anchored without a camera.
    pub fn update(&mut self, now: f32, settings: Settings, view: &View) -> View {
        // The preview shows numbers even with Show damage numbers off.
        let s = Settings { enabled: true, ..settings };
        if s != self.model.settings {
            self.model.configure(s);
        }
        let camera = has_camera(view);
        // Spots keep their world points across loops, like real hits, until one leaves the screen
        // or the camera appears or vanishes; then they are anchored in the frame again.
        let kept = self
            .run
            .as_ref()
            .filter(|r| r.flat != camera && (r.flat || r.anchors.iter().all(|&a| project(a, view).is_some())))
            .map(|r| (r.anchors, r.flat));
        let restart = kept.is_none()
            || self.run.as_ref().is_none_or(|r| !(now >= r.start && now - r.start < r.length) || r.mode != s.preview);
        if restart {
            let (anchors, flat) = kept.unwrap_or_else(|| {
                let points = spots(frame(view));
                let world = camera.then(|| points.map(|p| unproject(p, view, DEPTH)));
                match world {
                    Some([Some(a), Some(b), Some(c)]) => ([a, b, c], false),
                    _ => (points.map(|p| flat_anchor(p, view)), true),
                }
            });
            self.random ^= now.to_bits() | 1;
            self.run = Some(Run { start: now, length: loop_length(s.preview, s.lifetime), mode: s.preview, spawned: 0, anchors, flat });
        }
        let Some(run) = self.run.as_mut() else { return *view };
        let (start, anchors, flat) = (run.start, run.anchors, run.flat);
        let times = spawn_times(run.mode);
        let mut due = 0;
        while run.spawned < times.len() && times[run.spawned] <= now - start {
            run.spawned += 1;
            due += 1;
        }
        let first = run.spawned - due;
        for &at in &times[first..first + due] {
            for (spot, &anchor) in anchors.iter().enumerate() {
                let kind = if s.random {
                    // Weak point, normal or bounce, then any critical, each equally likely.
                    let base = [NORMAL[0], NORMAL[1], NORMAL[2]][self.next(3) as usize];
                    let critical = [Critical::None, Critical::Positive, Critical::Negative][self.next(3) as usize];
                    Kind { critical, ..base }
                } else {
                    NORMAL[spot]
                };
                self.serial = self.serial.wrapping_add(1).max(1);
                let damage = 1 + self.next(150) as i32;
                self.model.capture(Event {
                    key: HitKey { epoch: self.model.epoch, serial: self.serial, target: spot as u32 + 1, application: 0 },
                    position: Some(anchor),
                    total_damage: damage,
                    physical_hzv: kind.hzv,
                    source: Source::Attack,
                    owner: Owner::LocalPlayer,
                    critical: kind.critical,
                    bounced: kind.bounced,
                    captured_at: start + at,
                });
            }
        }
        self.model.update(now, true);
        if flat { flat_view(view) } else { *view }
    }
    /// Where the loop's spots are in the world (or flat space), for drawing their dots.
    pub fn anchors(&self) -> Option<[Vec3; 3]> {
        self.run.as_ref().map(|r| r.anchors)
    }
    /// Whether the spots are anchored in the world (true) or on screen for want of a camera (false).
    pub fn in_world(&self) -> Option<bool> {
        self.run.as_ref().map(|r| !r.flat)
    }
}
