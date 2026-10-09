//! Projection, animation and stable overlap placement, independent of ImGui or game memory.
use crate::*;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
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
    fn overlap(self, b: Self, margin: f32) -> f32 {
        (self.right.min(b.right) + margin - self.left.max(b.left)).max(0.0)
            * (self.bottom.min(b.bottom) + margin - self.top.max(b.top)).max(0.0)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct View {
    pub world_to_clip: [f32; 16],
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct TextMetrics {
    pub advance: f32,
    pub ink: Rect,
}
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
    if rect[0] != 0
        || rect[1] != 0
        || rect[2] <= 0
        || rect[3] <= 0
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || ((rect[2] as f32 / rect[3] as f32) / (width / height) - 1.0).abs() > 0.02
        || view.iter().chain(projection).any(|x| !x.is_finite())
    {
        return None;
    }
    let mut result = View {
        width,
        height,
        ..View::default()
    };
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
    let x = m[0] * p.x + m[1] * p.y + m[2] * p.z + m[3];
    let y = m[4] * p.x + m[5] * p.y + m[6] * p.z + m[7];
    let z = m[8] * p.x + m[9] * p.y + m[10] * p.z + m[11];
    let w = m[12] * p.x + m[13] * p.y + m[14] * p.z + m[15];
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
#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub scale: f32,
    pub opacity: f32,
    pub rise: f32,
}
pub fn motion(age: f32, lifetime: f32, style: Style) -> Motion {
    if style == Style::Rise {
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
        let stamp = 1.0 - (age / 0.2).clamp(0.0, 1.0);
        let exit = ((age - (lifetime - 0.2)) / 0.2).clamp(0.0, 1.0);
        return Motion {
            scale: 1.0 + 0.2 * stamp * stamp,
            opacity: 1.0 - exit,
            rise: 30.0 * exit,
        };
    }
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
pub fn rise_slash(origin: Point, key: HitKey, age: f32, view: &View) -> Slash {
    let mut hash = key.serial.wrapping_mul(0x9e3779b9)
        ^ key.epoch.wrapping_mul(0x85ebca6b)
        ^ key.target
        ^ key.application.wrapping_mul(0xc2b2ae35);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb352d);
    hash ^= hash >> 15;
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
    for item in &mut cache.entries {
        if item.live
            && !model.numbers[..model.count].iter().any(|e| {
                e.key == item.key && now >= e.captured_at && now - e.captured_at < s.lifetime
            })
        {
            item.live = false;
        }
    }
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
    let mut count = 0;
    for &e in &model.numbers[..model.count] {
        let age = now - e.captured_at;
        if age < 0.0 || age >= s.lifetime || !visible(e, s) {
            continue;
        }
        let Some(anchor) = project(e.position, view) else {
            continue;
        };
        let m = motion(age, s.lifetime, s.style);
        let settled = base
            * if s.style == Style::Rise && tint(e) == Tint::Weak {
                1.4
            } else {
                1.0
            };
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
        if s.style == Style::Rise {
            d.slash = rise_slash(anchor, e.key, age, view);
        }
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
        let marker_unit = if s.style == Style::Wilds {
            unit
        } else {
            view.height / 720.0
        };
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
        if !d.solid.inside(view) {
            continue;
        }
        let half_height = if s.style == Style::Wilds {
            d.pixels * WILDS_STAR_HEIGHT * 0.5
        } else {
            2.0 * unit
        };
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
