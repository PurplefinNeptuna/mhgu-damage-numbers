//! Portable copied-value damage model. Native pointers and Forge calls stay in the plugin.
#![no_std]
pub mod capture;
pub mod mailbox;
pub mod presentation;
pub mod preview;
pub mod purple_lib;
use core::fmt;
use core::str::FromStr;
use core::sync::atomic::{AtomicU32, Ordering};

// Source describes which native damage family produced the finalized amount.
// Owner describes who controls an attack. Keeping them separate allows
// monster/object damage while filtering remote hunter and Palico attacks.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    Attack,
    Monster,
    Object,
    Tick,
    /// Mount finisher; native status code, not a hit. Drawn in the weak-point colour.
    Status,
    // The mailbox carries `source as u8` and the consumer decodes 6 and 7, so these values are fixed.
    /// Poison tick from main+0xa2664; drawn gray like a bounce.
    Poison = 6,
    /// Blast detonation from main+0xaac54; drawn in the weak-point colour.
    Blast = 7,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Owner {
    LocalPlayer,
    LocalPalico,
    RemotePlayer,
    RemotePalico,
    #[default]
    Unknown,
    /// The map's uStageShellOwner (character type 9) fired this shell, e.g. the rail cannon.
    Stage,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Critical {
    #[default]
    None,
    Positive,
    Negative,
}
// Tint is the shared damage classification. Each visual style supplies its
// own RGB palette without changing ownership or weak-hitzone rules.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tint {
    #[default]
    White,
    Gray,
    Weak,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Style {
    #[default]
    Wilds,
    Classic,
    Rise,
}
// These lowercase names are the stable INI values; Display writes the same
// names so a save/load round trip preserves the selected style.
impl FromStr for Style {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "wilds" => Ok(Self::Wilds),
            "classic" => Ok(Self::Classic),
            "rise" => Ok(Self::Rise),
            _ => Err(()),
        }
    }
}
impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wilds => "wilds",
            Self::Classic => "classic",
            Self::Rise => "rise",
        })
    }
}
/// How the Mod Options preview spawns hits at each source point.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preview {
    /// One hit, then a pause.
    #[default]
    Single,
    /// Five hits, one every 0.15 s.
    Burst,
    /// Five hits at once.
    Multi,
}
// Lowercase names are the stable INI values, as for Style.
impl FromStr for Preview {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "single" => Ok(Self::Single),
            "burst" => Ok(Self::Burst),
            "multi" => Ok(Self::Multi),
            _ => Err(()),
        }
    }
}
impl fmt::Display for Preview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Single => "single",
            Self::Burst => "burst",
            Self::Multi => "multi",
        })
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}
// A serial is unique only within one producer. application identifies that
// mailbox; target identifies the monster. epoch separates camera/player
// sessions so an old queued hit cannot become a hit in a new display session.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitKey {
    pub epoch: u32,
    pub serial: u32,
    pub target: u32,
    pub application: u32,
}
// A self-contained hit snapshot after native filtering. It has no borrowed
// game pointers, so the display can retain it until its lifetime expires.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Event {
    pub key: HitKey,
    /// None keeps valid damage visible at screen centre when impact capture failed.
    pub position: Option<Vec3>,
    pub total_damage: i32,
    /// Physical HZV before damage weighting; -1 means unavailable.
    pub physical_hzv: i16,
    pub source: Source,
    pub owner: Owner,
    pub critical: Critical,
    pub bounced: bool,
    pub captured_at: f32,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub enabled: bool,
    pub style: Style,
    pub scale: f32,
    pub lifetime: f32,
    /// Multiplier on the placement offsets that keep numbers apart; 0 stacks them on the hit.
    pub spread: f32,
    /// Mod Options preview only: how hits spawn, and whether their types are random.
    pub preview: Preview,
    pub random: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            style: Style::Wilds,
            scale: 1.0,
            lifetime: 0.9,
            spread: 0.8,
            preview: Preview::Single,
            random: false,
        }
    }
}
impl Settings {
    pub fn valid(self) -> bool {
        self.scale.is_finite()
            && (0.5..=3.0).contains(&self.scale)
            && self.lifetime.is_finite()
            && (0.4..=2.0).contains(&self.lifetime)
            && self.spread.is_finite()
            && (0.0..=2.0).contains(&self.spread)
    }
}
/// The current settings, readable and writable from any thread without a lock. Purple-lib's
/// get/set run in the game's update and may not lock; rendering copies this into the Model.
// shortcut: fields are stored one by one, so two writers at once can mix their edits; the
// ImGui panel and the Options window are never edited together. Upgrade to one packed word
// if a writer ever runs concurrently with another.
pub struct SharedSettings {
    // bit 0 enabled, bit 1 random, bits 8..16 style, bits 16..24 preview.
    flags: AtomicU32,
    scale: AtomicU32,
    lifetime: AtomicU32,
    spread: AtomicU32,
}
impl SharedSettings {
    pub const fn new() -> Self {
        Self {
            flags: AtomicU32::new(1),
            scale: AtomicU32::new(1.0f32.to_bits()),
            lifetime: AtomicU32::new(0.9f32.to_bits()),
            spread: AtomicU32::new(0.8f32.to_bits()),
        }
    }
    pub fn load(&self) -> Settings {
        let flags = self.flags.load(Ordering::Acquire);
        Settings {
            enabled: flags & 1 != 0,
            random: flags & 2 != 0,
            style: match (flags >> 8) as u8 {
                1 => Style::Classic,
                2 => Style::Rise,
                _ => Style::Wilds,
            },
            preview: match (flags >> 16) as u8 {
                1 => Preview::Burst,
                2 => Preview::Multi,
                _ => Preview::Single,
            },
            scale: f32::from_bits(self.scale.load(Ordering::Acquire)),
            lifetime: f32::from_bits(self.lifetime.load(Ordering::Acquire)),
            spread: f32::from_bits(self.spread.load(Ordering::Acquire)),
        }
    }
    /// Stores `s` if it is valid; returns whether it did.
    pub fn store(&self, s: Settings) -> bool {
        if !s.valid() {
            return false;
        }
        self.scale.store(s.scale.to_bits(), Ordering::Release);
        self.lifetime.store(s.lifetime.to_bits(), Ordering::Release);
        self.spread.store(s.spread.to_bits(), Ordering::Release);
        let flags = s.enabled as u32
            | (s.random as u32) << 1
            | (s.style as u32) << 8
            | (s.preview as u32) << 16;
        self.flags.store(flags, Ordering::Release);
        true
    }
}
impl Default for SharedSettings {
    fn default() -> Self {
        Self::new()
    }
}
// Apply the same admission rules to pending events and already shown numbers.
// Unknown hitzones (-1) and missing positions are allowed. Non-finite supplied
// positions/times and zero damage are rejected; missing positions use screen centre.
pub fn visible(e: Event, s: Settings) -> bool {
    s.enabled
        && s.valid()
        && e.total_damage > 0
        && e.position.is_none_or(Vec3::valid)
        && e.captured_at.is_finite()
        && e.key.serial != 0
        && e.key.target != 0
        && e.physical_hzv >= -1
        && match e.source {
            Source::Attack => matches!(e.owner, Owner::LocalPlayer | Owner::LocalPalico),
            Source::Monster | Source::Object | Source::Status | Source::Poison | Source::Blast => true,
            Source::Tick => false,
        }
}
// Bounce feedback takes precedence over the weak-hitzone threshold of 45.
// An unavailable hitzone falls through to ordinary white digits.
// Poison, blast and mount knockdown keep their family colour regardless of hitzone.
pub fn tint(e: Event) -> Tint {
    if e.bounced || e.source == Source::Poison {
        Tint::Gray
    } else if e.physical_hzv >= 45 || matches!(e.source, Source::Blast | Source::Status) {
        Tint::Weak
    } else {
        Tint::White
    }
}
pub fn rgb(t: Tint, style: Style) -> u32 {
    match (style, t) {
        (Style::Rise, Tint::Gray) => 0x808080,
        (Style::Rise, Tint::Weak) => 0xf0b41e,
        (Style::Wilds, Tint::Gray) => 0x9a9a9a,
        (Style::Wilds, Tint::Weak) => 0xf2c45b,
        (Style::Wilds, Tint::White) => 0xf4f1ec,
        (_, Tint::Gray) => 0x9a9a9a,
        (_, Tint::Weak) => 0xffbf3f,
        _ => 0xffffff,
    }
}
/// Only a positive finalized HP contribution is display damage; native healing type 0x1d is excluded.
pub fn damage_outcome(native_type: u8, contribution: i32) -> bool {
    native_type != 0x1d && contribution > 0
}
// The native encoding uses 1 for negative and 2 for positive critical.
// Other flag values produce no critical marker.
pub fn critical_outcome(flag: u8) -> Critical {
    match flag {
        1 => Critical::Negative,
        2 => Critical::Positive,
        _ => Critical::None,
    }
}
/// Native tags pack primary kind, subtype and secondary kind in the low three bytes.
/// Ownership keeps remote projectiles and unknown generic impacts out of the local attack family.
pub fn source_outcome(tags: u32, owner: Owner) -> Option<Source> {
    let (primary, subtype, secondary) = (tags as u8, (tags >> 8) as u8, (tags >> 16) as u8);
    // Accept the audited direct and secondary-source attack tag combinations
    // only with a resolved owner. The visibility rule later decides local/remote.
    if ((primary == 0 || primary == 3)
        || (primary == 2
            && ((subtype == 1 && secondary == 0)
                || (subtype == 4 && secondary == 3)
                || (subtype == 8 && secondary == 3)))
        // Kinsect: its shell rewrites the hit's primary kind to 6 after the
        // hunter-shell setup (main+0x4816c0), keeping subtype 1 and the hunter at +0x70.
        || (primary == 6 && subtype == 1 && secondary == 0))
        && owner != Owner::Unknown
    {
        return Some(Source::Attack);
    }
    // Monster-origin damage does not need a hunter owner. The object case below
    // still requires ownership evidence to identify its admitted native family.
    if subtype == 2 && (primary == 1 || (primary == 2 && secondary == 1)) {
        return Some(Source::Monster);
    }
    if primary == 2 && subtype == 8 && secondary == 0 && owner != Owner::Unknown {
        return Some(Source::Object);
    }
    // Map-fired shells keep secondary kind 2 because main+0x4a2790 rewrites the
    // owner only for hunters, Palicoes and monsters; attack_owner identifies them.
    if primary == 2 && secondary == 2 && owner == Owner::Stage {
        return Some(Source::Object);
    }
    None
}
/// Consumer-owned bounded histories. Each impact remains independent; nearby hits are never summed.
pub struct Model {
    // queue is the pending ring: head is its oldest item, pending its length.
    // numbers is a packed array of visible events. recent is a separate ring of
    // identities already admitted, preventing repeated publication of one hit.
    queue: [Event; 256],
    head: usize,
    pending: usize,
    pub numbers: [Event; 128],
    pub count: usize,
    recent: [HitKey; 512],
    recent_count: usize,
    recent_next: usize,
    pub epoch: u32,
    pub settings: Settings,
    pub duplicates: u32,
    pub filtered: u32,
    pub queue_drops: u32,
    pub active_drops: u32,
}
impl Default for Model {
    fn default() -> Self {
        Self {
            queue: [Event::default(); 256],
            head: 0,
            pending: 0,
            numbers: [Event::default(); 128],
            count: 0,
            recent: [HitKey::default(); 512],
            recent_count: 0,
            recent_next: 0,
            epoch: 0,
            settings: Settings::default(),
            duplicates: 0,
            filtered: 0,
            queue_drops: 0,
            active_drops: 0,
        }
    }
}
impl Model {
    // Clear queue, visible numbers and deduplication together. Counters and user
    // settings survive this display reset; no old epoch entries are retained.
    pub fn reset(&mut self, epoch: u32) {
        self.epoch = epoch;
        self.count = 0;
        self.recent_count = 0;
        self.recent_next = 0;
        self.head = 0;
        self.pending = 0;
    }
    pub fn pending(&self) -> usize {
        self.pending
    }
    fn push(&mut self, e: Event) {
        // When full, discard the oldest pending hit before appending the newest.
        // Fixed storage keeps the event path free of per-hit heap allocation.
        if self.pending == 256 {
            self.head = (self.head + 1) % 256;
            self.pending -= 1;
            self.queue_drops = self.queue_drops.wrapping_add(1);
        }
        self.queue[(self.head + self.pending) % 256] = e;
        self.pending += 1;
    }
    fn pop(&mut self) -> Option<Event> {
        if self.pending == 0 {
            return None;
        }
        let e = self.queue[self.head];
        self.head = (self.head + 1) % 256;
        self.pending -= 1;
        Some(e)
    }
    // Accept a validated snapshot, then compact both collections under the new
    // visibility policy. Disabling clears hits immediately; it does not merely
    // hide them until the switch is enabled again.
    pub fn configure(&mut self, next: Settings) -> bool {
        if !next.valid() {
            return false;
        }
        self.settings = next;
        let mut kept = 0;
        for i in 0..self.count {
            if visible(self.numbers[i], next) {
                self.numbers[kept] = self.numbers[i];
                kept += 1;
            }
        }
        self.count = kept;
        // Visit only the original pending count. Accepted hits go back to the tail
        // in their original order, without being processed again by this loop.
        let n = self.pending;
        for _ in 0..n {
            let e = self.pop().unwrap();
            if visible(e, next) {
                self.push(e);
            }
        }
        true
    }
    // Check epoch/visibility first, then remember the complete identity before
    // queuing. Deduplication compares keys, never damage amount or proximity.
    pub fn capture(&mut self, e: Event) -> bool {
        if e.key.epoch != self.epoch || !visible(e, self.settings) {
            self.filtered = self.filtered.wrapping_add(1);
            return false;
        }
        if self.recent[..self.recent_count].contains(&e.key) {
            self.duplicates = self.duplicates.wrapping_add(1);
            return false;
        }
        self.recent[self.recent_next] = e.key;
        self.recent_next = (self.recent_next + 1) % 512;
        self.recent_count = (self.recent_count + 1).min(512);
        self.push(e);
        true
    }
    // Feedback can arrive before or after a hit becomes visible. Find that exact
    // identity in either collection and update its tint without adding a new hit.
    pub fn bounce(&mut self, key: HitKey) -> bool {
        if key.epoch != self.epoch {
            return false;
        }
        for i in 0..self.pending {
            let e = &mut self.queue[(self.head + i) % 256];
            if e.key == key {
                e.bounced = true;
                return true;
            }
        }
        for e in &mut self.numbers[..self.count] {
            if e.key == key {
                e.bounced = true;
                return true;
            }
        }
        false
    }
    // One display frame: clear if drawing is unavailable, expire old numbers,
    // then admit still-fresh queued hits. Age starts at capture, not dequeue time.
    pub fn update(&mut self, now: f32, ready: bool) {
        if !ready || !now.is_finite() || !self.settings.enabled {
            self.reset(self.epoch);
            return;
        }
        let mut kept = 0;
        for i in 0..self.count {
            let e = self.numbers[i];
            let age = now - e.captured_at;
            if age >= 0.0 && age < self.settings.lifetime && visible(e, self.settings) {
                self.numbers[kept] = e;
                kept += 1;
            }
        }
        self.count = kept;
        while let Some(e) = self.pop() {
            let age = now - e.captured_at;
            if e.key.epoch != self.epoch
                || !visible(e, self.settings)
                || age < 0.0
                || age >= self.settings.lifetime
            {
                continue;
            }
            // At the visible limit, shift out the oldest event to keep the newest 128.
            // Individual hits remain separate even when their screen positions coincide.
            if self.count == 128 {
                self.numbers.copy_within(1..128, 0);
                self.count -= 1;
                self.active_drops = self.active_drops.wrapping_add(1);
            }
            self.numbers[self.count] = e;
            self.count += 1;
        }
    }
}
/// RGB-to-ImGui conversion; round alpha exactly as the C++ renderer does.
pub fn imgui_color(rgb: u32, opacity: f32) -> u32 {
    if !opacity.is_finite() {
        return 0;
    }
    // Input is 0xRRGGBB. ImGui's packed value puts R in the low byte and alpha
    // in the high byte; adding 0.5 reproduces r25's byte rounding.
    ((rgb >> 16) & 255)
        | (rgb & 0xff00)
        | ((rgb & 255) << 16)
        | (((opacity.clamp(0.0, 1.0) * 255.0 + 0.5) as u32) << 24)
}
// Convert an integer clock difference to seconds only after subtraction.
// Option signals an invalid clock/order instead of fabricating an event age.
pub fn seconds(tick: u64, origin: u64, frequency: u32) -> Option<f32> {
    if frequency == 0 || tick < origin {
        return None;
    }
    let value = (tick - origin) as f32 / frequency as f32;
    value.is_finite().then_some(value)
}
// A zero stamp means no captured sample. Reject future or expired samples
// using the same age conversion as visible events.
pub fn fresh(tick: u64, now: u64, frequency: u32, limit: f32) -> bool {
    tick != 0 && seconds(now, tick, frequency).is_some_and(|age| age < limit)
}
