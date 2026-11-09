//! Projection, animation and stable overlap placement, independent of ImGui or game memory.
use crate::*;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
// Screen-space rectangle in pixels; right/bottom are edges, not width/height.
// The same representation describes visible ink and reserved placement bounds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl Rect {
    pub fn valid(self) -> bool {
        self.left.is_finite()
            && self.top.is_finite()
            && self.right.is_finite()
            && self.bottom.is_finite()
            && self.right > self.left
            && self.bottom > self.top
    }
    pub fn shifted(self, p: Point) -> Self {
        Self {
            left: self.left + p.x,
            top: self.top + p.y,
            right: self.right + p.x,
            bottom: self.bottom + p.y,
        }
    }
    // Smallest rectangle containing both shapes, used to include critical markers
    // and every visible glyph in the number's bounding box.
    pub fn united(self, b: Self) -> Self {
        Self {
            left: self.left.min(b.left),
            top: self.top.min(b.top),
            right: self.right.max(b.right),
            bottom: self.bottom.max(b.bottom),
        }
    }
    pub fn expand(self, n: f32) -> Self {
        Self {
            left: self.left - n,
            top: self.top - n,
            right: self.right + n,
            bottom: self.bottom + n,
        }
    }
    fn inside(self, v: &View) -> bool {
        self.left >= v.x
            && self.top >= v.y
            && self.right <= v.x + v.width
            && self.bottom <= v.y + v.height
    }
    // Score overlap area in square pixels. margin penalizes nearly touching
    // numbers too; min/max compute intersection edges and max(0) rejects gaps.
    fn overlap(self, b: Self, margin: f32) -> f32 {
        (self.right.min(b.right) + margin - self.left.max(b.left)).max(0.0)
            * (self.bottom.min(b.bottom) + margin - self.top.max(b.top)).max(0.0)
    }
}
// A copied world-to-clip matrix and its screen viewport. No native camera
// object is needed after this value has been constructed.
#[derive(Clone, Copy, Debug, Default)]
pub struct View {
    pub world_to_clip: [f32; 16],
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl View {
    /// A valid display without a camera. The zero matrix makes projection use screen centre.
    pub fn screen(width: f32, height: f32) -> Option<Self> {
        (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0).then_some(Self {
            width,
            height,
            ..Self::default()
        })
    }
}
// advance is where the next character would start, including glyph spacing.
// ink is the actual visible glyph rectangle relative to the text origin;
// centering uses advance while clipping/collision checks use visible ink.
#[derive(Clone, Copy, Debug, Default)]
pub struct TextMetrics {
    pub advance: f32,
    pub ink: Rect,
}
// Rise critical effect geometry, independent of the digits' placement offset.
// origin is the raw projected impact; clip limits the long slash to the viewport.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Slash {
    pub origin: Point,
    pub slope: f32,
    pub extent: f32,
    pub width: f32,
    pub opacity: f32,
    pub clip: Rect,
}
// One ready-to-draw number. position is the displaced text origin; offset is
// the cached displacement from its projected anchor. star/slash locate effects.
// pixels/opacity are the current animation state, ink/solid are current bounds,
// and reserve is the larger peak-size rectangle used to place later hits.
// text is a fixed NUL-terminated digit buffer understood by raw ImGui calls.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawNumber {
    pub position: Point,
    pub offset: Point,
    pub star: Point,
    pub slash: Slash,
    pub pixels: f32,
    pub opacity: f32,
    pub advance: f32,
    pub unit: f32,
    pub classic_stroke: f32,
    pub ink: Rect,
    pub solid: Rect,
    pub reserve: Rect,
    pub color: u32,
    pub critical: Critical,
    pub style: Style,
    pub key: HitKey,
    pub text: [u8; 12],
}
// r25 sizes at a 720-pixel-high viewport. layout scales by height/720 and the
// user's size setting. Rise's smaller base is separate from its width stretch.
pub const BASE_PIXELS: f32 = 24.0;
pub const RISE_BASE_PIXELS: f32 = 20.0;
pub const RISE_GLYPH_WIDTH: f32 = 1.08;
pub const WILDS_CRIT_SCALE: f32 = 1.5;
pub const WILDS_STAR_HEIGHT: f32 = 0.31612110 * WILDS_CRIT_SCALE;
/// Native cameras multiply row vectors. Transpose view*projection for the column-vector projector.
pub fn native_camera(
    view: &[f32; 16],
    projection: &[f32; 16],
    rect: [i32; 4],
    width: f32,
    height: f32,
) -> Option<View> {
    let mut result = View::screen(width, height)?;
    // Accept an origin-aligned viewport whose shape matches the display (within 2%).
    // Dimensions may differ from the display under resolution scaling; this does
    // not prove that a same-aspect offscreen view belongs to the presented frame.
    let near = |a: f32, b: f32| (a / b - 1.0).abs() <= 0.02;
    let display = width / height;
    if rect[0] != 0
        || rect[1] != 0
        || rect[2] <= 0
        || rect[3] <= 0
        || view.iter().chain(projection).any(|x| !x.is_finite())
    {
        return None;
    }
    let rect_aspect = rect[2] as f32 / rect[3] as f32;
    if !near(rect_aspect, display) {
        // A port can present the picture inside a larger overlay (black bars) or
        // stretch it to a resized window. The perspective builder main+0x7c617c
        // stores cot(fov/2)/aspect at [0] and cot(fov/2) at [5], so [5]/[0] is
        // the picture's shape; it must match the viewport or the display. The
        // picture is the largest of that shape that fits, centred.
        let picture = projection[5] / projection[0];
        if !(picture.is_finite() && picture > 0.0 && (near(picture, rect_aspect) || near(picture, display))) {
            return None;
        }
        if picture > display {
            result.height = width / picture;
            result.y = (height - result.height) * 0.5;
        } else {
            result.width = height * picture;
            result.x = (width - result.width) * 0.5;
        }
    }
    // For each output element, sum four products from view and projection.
    // The swapped row/column indices also transpose the native row-vector result,
    // so project can multiply column vectors with ordinary row-major indexing.
    for row in 0..4 {
        for col in 0..4 {
            let mut value = 0.0;
            for k in 0..4 {
                value += view[col * 4 + k] * projection[k * 4 + row];
            }
            if !value.is_finite() {
                return None;
            }
            result.world_to_clip[row * 4 + col] = value;
        }
    }
    Some(result)
}
// Transform an impact from world coordinates to clip coordinates, reject
// points outside the camera, then divide by w to obtain normalized coordinates.
// Option lets layout skip clipped/invalid points without drawing at (0,0).
pub fn project(p: Vec3, v: &View) -> Option<Point> {
    if !p.valid()
        || ![v.x, v.y, v.width, v.height].iter().all(|x| x.is_finite())
        || v.width <= 0.0
        || v.height <= 0.0
        || v.world_to_clip.iter().any(|x| !x.is_finite())
    {
        return None;
    }
    let m = &v.world_to_clip;
    // Each matrix row multiplies [world_x, world_y, world_z, 1]. w carries
    // perspective depth; x/y/z are not screen pixels until divided by w.
    let x = m[0] * p.x + m[1] * p.y + m[2] * p.z + m[3];
    let y = m[4] * p.x + m[5] * p.y + m[6] * p.z + m[7];
    let z = m[8] * p.x + m[9] * p.y + m[10] * p.z + m[11];
    let w = m[12] * p.x + m[13] * p.y + m[14] * p.z + m[15];
    // Require positive w and the native clip volume: x/y in [-w,w], z in [0,w].
    // This avoids dividing by zero and drawing hits behind/outside the camera.
    if ![x, y, z, w].iter().all(|x| x.is_finite())
        || w <= 0.00001
        || x < -w
        || x > w
        || y < -w
        || y > w
        || z > w
        || z < 0.0
    {
        return None;
    }
    // Map normalized x from [-1,1] to the viewport's horizontal pixel range.
    // Invert y because screen pixels increase downward while clip y increases up.
    let point = Point {
        x: v.x + (x / w + 1.0) * v.width * 0.5,
        y: v.y + (1.0 - y / w) * v.height * 0.5,
    };
    (point.x.is_finite() && point.y.is_finite()).then_some(point)
}
/// Positive int32 damage fits ten digits plus NUL; rendering never formats through the heap.
pub fn digits(mut value: i32) -> [u8; 12] {
    let mut out = [0; 12];
    if value <= 0 {
        return out;
    }
    // Modulo 10 extracts digits right-to-left; reverse them into the output.
    // The unused zero bytes supply the terminator required by ImGui text calls.
    let mut reverse = [0; 10];
    let mut n = 0;
    while value != 0 {
        reverse[n] = b'0' + (value % 10) as u8;
        n += 1;
        value /= 10;
    }
    for i in 0..n {
        out[i] = reverse[n - 1 - i];
    }
    out
}
// scale multiplies settled font size; opacity controls alpha. rise is upward
// travel measured in 720p reference pixels and is scaled by layout.
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub scale: f32,
    pub opacity: f32,
    pub rise: f32,
}
// age is seconds since capture, so late consumption starts partway through
// the animation. Each style reproduces r25's timing instead of a shared tween.
pub fn motion(age: f32, lifetime: f32, style: Style) -> Motion {
    if style == Style::Rise {
        // Rise entrance splits nine timing units into 2/5/2: grow 0.64 to 1.20,
        // shrink to 0.88, then settle at 1.00. Cap entrance/exit by lifetime fractions
        // so a short configured lifetime still leaves a readable middle phase.
        let entrance = 0.3f32.min(lifetime * 0.3);
        let exit_time = (10.0f32 / 30.0).min(lifetime * 0.35);
        let t = (age / entrance).clamp(0.0, 1.0);
        let scale = if t < 2.0 / 9.0 {
            0.64 + 0.56 * t * 9.0 / 2.0
        } else if t < 7.0 / 9.0 {
            1.2 - 0.32 * (t * 9.0 - 2.0) / 5.0
        } else {
            0.88 + 0.12 * (t * 9.0 - 7.0) / 2.0
        };
        let exit = ((age - (lifetime - exit_time)) / exit_time).clamp(0.0, 1.0);
        // The first tenth of Rise's exit nudges downward by four reference pixels
        // (negative rise), then travels upward while fading through the remaining exit.
        let kick = (exit * 10.0).min(1.0);
        return Motion {
            scale,
            opacity: 1.0 - exit,
            rise: if exit <= 0.1 {
                -4.0 * kick
            } else {
                -4.0 + 54.0 * (exit - 0.1) / 0.9
            },
        };
    }
    if style == Style::Wilds {
        // Wilds starts 20% oversized and eases quadratically to its settled size.
        // Its final 0.2 seconds fade out while rising 30 reference pixels.
        let stamp = 1.0 - (age / 0.2).clamp(0.0, 1.0);
        let exit = ((age - (lifetime - 0.2)) / 0.2).clamp(0.0, 1.0);
        return Motion {
            scale: 1.0 + 0.2 * stamp * stamp,
            opacity: 1.0 - exit,
            rise: 30.0 * exit,
        };
    }
    // Classic starts 25% oversized, settles in 0.1 seconds, rises throughout
    // its lifetime and fades only during the last 0.25 seconds.
    Motion {
        scale: if age < 0.1 {
            1.0 + 0.25 * (1.0 - age / 0.1)
        } else {
            1.0
        },
        opacity: if age > lifetime - 0.25 {
            (lifetime - age) / 0.25
        } else {
            1.0
        },
        rise: age / lifetime * 30.0,
    }
}
// Hash the hit identity for a stable slight lean. This avoids per-frame random
// changes; the same hit always produces the same slash orientation.
pub fn rise_slash(origin: Point, key: HitKey, age: f32, view: &View) -> Slash {
    let mut hash = key.serial.wrapping_mul(0x9e3779b9)
        ^ key.epoch.wrapping_mul(0x85ebca6b)
        ^ key.target
        ^ key.application.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb352d);
    hash ^= hash >> 15;
    // Grow for 0.05 seconds, then fade over 0.15. The slash stays centered on the
    // original impact and spans roughly three quarters of the viewport height.
    let grow = (age / 0.05).clamp(0.0, 1.0);
    let fade = ((age - 0.05) / 0.15).clamp(0.0, 1.0);
    Slash {
        origin,
        slope: ((hash & 65535) as f32 / 65535.0 - 0.5) * 0.5,
        extent: view.height * 0.75 * (0.72 + 0.28 * grow),
        width: view.height / 720.0 * (24.0 + 24.0 * grow) * (1.0 - 0.8 * fade),
        opacity: if age < 0.0 {
            0.0
        } else {
            (0.35 + 0.65 * grow) * (1.0 - fade) * (1.0 - fade)
        },
        clip: Rect {
            left: view.x,
            top: view.y,
            right: view.x + view.width,
            bottom: view.y + view.height,
        },
    }
}
// Construct clockwise corners around the center: the slope shifts the top
// and bottom in opposite horizontal directions, while width expands each edge.
// Renderer maps one half of the positive/negative critical atlas onto this quad.
pub fn slash_quad(s: Slash) -> [Point; 4] {
    let half = s.extent * 0.5;
    let width = s.width * 0.5;
    [
        Point {
            x: s.origin.x + s.slope * half - width,
            y: s.origin.y - half,
        },
        Point {
            x: s.origin.x + s.slope * half + width,
            y: s.origin.y - half,
        },
        Point {
            x: s.origin.x - s.slope * half + width,
            y: s.origin.y + half,
        },
        Point {
            x: s.origin.x - s.slope * half - width,
            y: s.origin.y + half,
        },
    ]
}
// Include critical-marker geometry in the number bounds. Rise has a separate
// viewport-clipped slash, so its slash does not enlarge the digit placement box.
// Wilds reserves stars; Classic reserves its top/bottom critical line.
fn solid_bounds(
    ink: Rect,
    pos: Point,
    pixels: f32,
    advance: f32,
    unit: f32,
    critical: Critical,
    style: Style,
) -> Rect {
    if critical == Critical::None || style == Style::Rise {
        return ink;
    }
    if style == Style::Wilds {
        let center = (ink.left + ink.right) * 0.5;
        let half_height = pixels * WILDS_STAR_HEIGHT * 0.5;
        let y = if critical == Critical::Positive {
            ink.top - 2.0 * unit - half_height
        } else {
            ink.bottom + 2.0 * unit + half_height
        };
        let width = pixels * 0.38078224 * WILDS_CRIT_SCALE;
        return ink.united(Rect {
            left: center - width,
            top: y - half_height,
            right: center + width,
            bottom: y + half_height,
        });
    }
    let stroke = 1.25 * unit;
    let y = if critical == Critical::Positive {
        pos.y - 2.0 * stroke
    } else {
        pos.y + pixels + 2.0 * stroke
    };
    ink.united(Rect {
        left: pos.x - 2.0 * stroke,
        top: y - 2.0 * stroke,
        right: pos.x + advance + 2.0 * stroke,
        bottom: y + 2.0 * stroke,
    })
}
#[derive(Clone, Copy, Default)]
// Remember one offset for each live hit identity. Reusing it across frames
// prevents numbers from jumping around as neighbors appear, fade or expire.
struct PlacementEntry {
    key: HitKey,
    offset: Point,
    live: bool,
}
pub struct Placement {
    entries: [PlacementEntry; 128],
    viewport: [f32; 4],
    scale: f32,
    style: Style,
    epoch: u32,
}
impl Default for Placement {
    fn default() -> Self {
        Self {
            entries: [PlacementEntry::default(); 128],
            viewport: [0.0; 4],
            scale: 0.0,
            style: Style::Wilds,
            epoch: 0,
        }
    }
}
impl Placement {
    pub fn reset(&mut self) {
        for entry in &mut self.entries {
            entry.live = false;
        }
        self.viewport = [0.0; 4];
    }
}
/// Reserve peak-animation bounds once per hit, then keep its chosen offset across frames.
pub fn layout(
    model: &Model,
    view: &View,
    now: f32,
    out: &mut [DrawNumber; 128],
    cache: &mut Placement,
    mut measure: impl FnMut(&[u8; 12], f32, Style) -> Option<TextMetrics>,
) -> usize {
    let s = model.settings;
    // Convert r25's reference size into this viewport's pixels, then apply user
    // scale. unit is the spacing unit used for margins and Wilds critical markers.
    let base = (if s.style == Style::Rise {
        RISE_BASE_PIXELS
    } else {
        BASE_PIXELS
    }) * view.height
        / 720.0
        * s.scale;
    let unit = base / 24.0;
    if !now.is_finite()
        || !s.enabled
        || !s.valid()
        || !unit.is_finite()
        || unit <= 0.0
        || model.count == 0
    {
        cache.reset();
        return 0;
    }
    let epoch = model.numbers[0].key.epoch;
    let viewport = [view.x, view.y, view.width, view.height];
    // A changed viewport/style/scale/epoch invalidates previous offsets because
    // their boxes were measured under different geometry or a different session.
    if cache.viewport != viewport
        || cache.scale != s.scale
        || cache.style != s.style
        || cache.epoch != epoch
    {
        cache.reset();
    }
    cache.viewport = viewport;
    cache.scale = s.scale;
    cache.style = s.style;
    cache.epoch = epoch;
    // Free cached slots after their matching live event expires. Surviving keys
    // retain their offsets; an unrelated new hit may reuse an inactive slot.
    for item in &mut cache.entries {
        if item.live
            && !model.numbers[..model.count].iter().any(|e| {
                e.key == item.key && now >= e.captured_at && now - e.captured_at < s.lifetime
            })
        {
            item.live = false;
        }
    }
    // Try a fixed set of nearby offsets in base-size units. The centered choice
    // comes first; area/distance scoring later chooses the least crowded valid box.
    const CANDIDATES: [(f32, f32); 17] = [
        (0.0, 0.0),
        (-0.8, 0.0),
        (0.8, 0.0),
        (0.0, -0.5),
        (0.0, 0.5),
        (-0.8, -0.5),
        (0.8, -0.5),
        (-0.8, 0.5),
        (0.8, 0.5),
        (-1.6, 0.0),
        (1.6, 0.0),
        (-1.6, -1.0),
        (1.6, -1.0),
        (-1.6, 1.0),
        (1.6, 1.0),
        (0.0, -1.0),
        (0.0, 1.0),
    ];
    // View::screen carries a zero matrix; only a finite non-zero matrix is a camera.
    let camera = view.world_to_clip.iter().all(|x| x.is_finite())
        && view.world_to_clip.iter().any(|&x| x != 0.0);
    let mut count = 0;
    for &e in &model.numbers[..model.count] {
        let age = now - e.captured_at;
        if age < 0.0 || age >= s.lifetime || !visible(e, s) {
            continue;
        }
        // With a camera, an out-of-view hit stays hidden at its world position and
        // appears once the view turns to it. Failed captures/cameras use screen centre.
        let anchor = match e.position {
            Some(p) if camera => match project(p, view) {
                Some(anchor) => anchor,
                None => continue,
            },
            _ => Point {
                x: view.x + view.width * 0.5,
                y: view.y + view.height * 0.5,
            },
        };
        let m = motion(age, s.lifetime, s.style);
        // Rise weak-hitzone digits are 1.4 times the ordinary settled size. Other
        // styles indicate weak hitzones with tint while keeping their base size.
        let settled = base
            * if s.style == Style::Rise && tint(e) == Tint::Weak {
                1.4
            } else {
                1.0
            };
        // Reserve the entrance's largest size before choosing an offset. Placement
        // based only on the current shrinking glyphs would admit overlapping new hits.
        let peak_pixels = settled * if s.style == Style::Classic { 1.25 } else { 1.2 };
        let mut d = DrawNumber {
            pixels: settled * m.scale,
            unit,
            classic_stroke: 1.25 * view.height / 720.0,
            opacity: m.opacity,
            style: s.style,
            color: rgb(tint(e), s.style),
            critical: if e.source == Source::Tick {
                Critical::None
            } else {
                e.critical
            },
            key: e.key,
            text: digits(e.total_damage),
            ..DrawNumber::default()
        };
        // Keep the slash anchored to the world impact before applying digit offsets.
        // Moving digits to avoid overlap must not move the critical slash with them.
        if s.style == Style::Rise {
            d.slash = rise_slash(anchor, e.key, age, view);
        }
        // Measure actual baked glyph ink at both animated and peak sizes. An absent
        // font/glyph prevents drawing this number; guessed widths would change layout.
        let (Some(current), Some(peak)) = (
            measure(&d.text, d.pixels, s.style),
            measure(&d.text, peak_pixels, s.style),
        ) else {
            continue;
        };
        if !current.ink.valid()
            || !peak.ink.valid()
            || !current.advance.is_finite()
            || !peak.advance.is_finite()
        {
            continue;
        }
        // Place text above the impact by eight reference pixels and subtract upward
        // animation travel. Horizontal centering uses total advance, including spacing.
        let cy = anchor.y - 8.0 * view.height / 720.0 - m.rise * view.height / 720.0;
        d.position = Point {
            x: anchor.x - current.advance * 0.5,
            y: cy,
        };
        d.advance = current.advance;
        let peak_pos = Point {
            x: anchor.x - peak.advance * 0.5,
            y: cy,
        };
        // Wilds marker gaps follow the user-scaled number size. Classic marker
        // spacing/stroke follows viewport scale independently, matching r25.
        let marker_unit = if s.style == Style::Wilds {
            unit
        } else {
            view.height / 720.0
        };
        // The reservation contains peak digit ink, critical markers and shadow
        // padding. Later hits compare against this box rather than just text width.
        let reserved = solid_bounds(
            peak.ink.shifted(peak_pos),
            peak_pos,
            peak_pixels,
            peak.advance,
            marker_unit,
            d.critical,
            s.style,
        );
        let shadow = match s.style {
            Style::Rise => peak_pixels * 0.13,
            Style::Wilds => unit,
            Style::Classic => 1.25 * view.height / 720.0,
        };
        let reserved = reserved.expand(shadow + if s.style != Style::Classic { 1.0 } else { 0.0 });
        let existing = cache
            .entries
            .iter()
            .position(|item| item.live && item.key == d.key);
        // An existing hit keeps its offset. For a new hit, score candidates against
        // numbers already laid out this frame; choose less overlap first, then less
        // distance from the original anchor. Reject boxes outside the viewport.
        let entry = if let Some(index) = existing {
            index
        } else {
            let mut best: Option<(Point, f32, f32)> = None;
            for (x, y) in CANDIDATES {
                let offset = Point {
                    x: x * base,
                    y: y * base,
                };
                let r = reserved.shifted(offset);
                if !r.inside(view) {
                    continue;
                }
                let area = out[..count]
                    .iter()
                    .map(|n| r.overlap(n.reserve, unit))
                    .sum::<f32>();
                // Squared distance orders candidates without a square root. Ties preserve
                // the fixed candidate ordering because only strictly better scores replace best.
                let distance = offset.x * offset.x + offset.y * offset.y;
                if best.is_none_or(|(_, a, b)| area < a || (area == a && distance < b)) {
                    best = Some((offset, area, distance));
                }
            }
            let Some((offset, _, _)) = best else {
                continue;
            };
            let Some(index) = cache.entries.iter().position(|item| !item.live) else {
                continue;
            };
            cache.entries[index] = PlacementEntry {
                key: d.key,
                offset,
                live: true,
            };
            index
        };
        d.offset = cache.entries[entry].offset;
        d.position.x += d.offset.x;
        d.position.y += d.offset.y;
        // Wilds/Rise truncate text origins to whole pixels for their atlas effects.
        // Classic retains subpixel positioning from the projected/animated coordinates.
        if s.style != Style::Classic {
            d.position.x = d.position.x as i32 as f32;
            d.position.y = d.position.y as i32 as f32;
        }
        d.ink = current.ink.shifted(d.position);
        d.reserve = reserved.shifted(d.offset);
        d.solid = solid_bounds(
            d.ink,
            d.position,
            d.pixels,
            d.advance,
            marker_unit,
            d.critical,
            s.style,
        )
        .expand(shadow);
        // Even a cached placement is skipped if current motion pushes its visible
        // bounds offscreen; the offset itself remains stable for the hit.
        if !d.solid.inside(view) {
            continue;
        }
        let half_height = if s.style == Style::Wilds {
            d.pixels * WILDS_STAR_HEIGHT * 0.5
        } else {
            2.0 * unit
        };
        // Positive critical markers go above the visible ink; negative markers go
        // below. Center on ink rather than advance so asymmetric digits stay aligned.
        d.star = Point {
            x: (d.ink.left + d.ink.right) * 0.5,
            y: if d.critical == Critical::Positive {
                d.ink.top - 2.0 * unit - half_height
            } else {
                d.ink.bottom + 2.0 * unit + half_height
            },
        };
        out[count] = d;
        count += 1;
    }
    count
}
