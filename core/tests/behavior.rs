//! Behavior tests use copied records and fake game memory; no emulator or installed package is involved.
use damage_numbers_core::{capture::*, mailbox::*, *};
use std::{collections::BTreeMap, sync::Arc};
fn hit(serial: u32) -> Event {
    Event {
        key: HitKey {
            epoch: 1,
            serial,
            target: 2,
            application: 0,
        },
        position: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.5,
        },
        total_damage: 123,
        physical_hzv: 45,
        owner: Owner::LocalPlayer,
        captured_at: 0.1,
        ..Event::default()
    }
}
#[test]
fn filters_duplicates_and_bounds() {
    let mut m = Model::default();
    m.reset(1);
    assert!(m.capture(hit(1)));
    assert!(!m.capture(hit(1)));
    let mut remote = hit(2);
    remote.owner = Owner::RemotePlayer;
    assert!(!m.capture(remote));
    remote.source = Source::Monster;
    assert!(m.capture(remote));
    for serial in 3..400 {
        m.capture(hit(serial));
    }
    assert_eq!(m.pending(), 256);
    assert_eq!(m.queue_drops, 143);
    m.update(0.2, true);
    assert_eq!(m.count, 128);
    assert_eq!(m.active_drops, 128);
    assert_eq!(m.numbers[0].key.serial, 272);
    m.update(1.1, true);
    assert_eq!(m.count, 0);
}
#[test]
fn feedback_updates_pending_and_visible() {
    let mut m = Model::default();
    m.reset(1);
    let e = hit(1);
    m.capture(e);
    assert!(m.bounce(e.key));
    m.update(0.2, true);
    assert_eq!(tint(m.numbers[0]), Tint::Gray);
    m.reset(2);
    assert!(!m.bounce(e.key));
}
#[test]
fn disabling_clears_hits_and_invalid_settings_preserve_previous() {
    let mut m = Model::default();
    m.reset(1);
    m.capture(hit(1));
    let old = m.settings;
    assert!(!m.configure(Settings {
        scale: f32::NAN,
        ..old
    }));
    assert_eq!(m.settings, old);
    assert!(m.configure(Settings {
        enabled: false,
        ..old
    }));
    assert_eq!(m.pending(), 0);
}
#[test]
fn sprite_bounds_and_overlapping_runs() {
    let mut pixels = [0; 12];
    assert!(decode_sprite(&[0, 7, 136, 1, 0], &mut pixels));
    assert_eq!(pixels, [7; 12]);
    assert!(!decode_sprite(&[128, 0, 0], &mut pixels));
    assert!(!decode_sprite(&[255, 0], &mut pixels));
    assert!(!decode_sprite(&[1, 1], &mut pixels));
}
#[test]
fn mailbox_overflow_retains_newest() {
    let m = Mailbox::new();
    for serial in 1..=300 {
        m.publish(Record {
            serial,
            ..Record::default()
        });
    }
    let mut cursor = Cursor::default();
    for serial in 45..=300 {
        assert_eq!(m.read(&mut cursor).unwrap().serial, serial);
    }
    assert!(m.read(&mut cursor).is_none());
    assert_eq!(cursor.overwritten, 44);
}
#[test]
fn concurrent_mailbox_snapshots_are_never_torn() {
    let m = Arc::new(Mailbox::new());
    let producer = m.clone();
    let thread = std::thread::spawn(move || {
        for serial in 1..=20000 {
            producer.publish(Record {
                serial,
                damage: serial as i32,
                flags: serial ^ 0xdeadbeef,
                tick: serial as u64 * 3,
                ..Record::default()
            });
        }
    });
    let mut cursor = Cursor::default();
    while !thread.is_finished() {
        if let Some(r) = m.read(&mut cursor) {
            assert_eq!(r.damage, r.serial as i32);
            assert_eq!(r.flags, r.serial ^ 0xdeadbeef);
            assert_eq!(r.tick, r.serial as u64 * 3);
        }
    }
    thread.join().unwrap();
    while let Some(r) = m.read(&mut cursor) {
        assert_eq!(r.damage, r.serial as i32);
    }
}
struct Fake {
    bytes: BTreeMap<u32, u8>,
}
impl Fake {
    fn new() -> Self {
        Self {
            bytes: BTreeMap::new(),
        }
    }
    fn region(&mut self, a: u32, n: usize) {
        for i in 0..n {
            self.bytes.insert(a + i as u32, 0);
        }
    }
    fn put(&mut self, a: u32, v: u32) {
        for (i, b) in v.to_le_bytes().iter().enumerate() {
            self.bytes.insert(a + i as u32, *b);
        }
    }
    fn scene() -> Self {
        let mut m = Self::new();
        m.region(0x188ca48, 4);
        m.put(0x188ca48, 0x1000);
        m.region(0x1885b58, 4);
        m.put(0x1885b58, 0x2000);
        m.region(0x1000, 0x34);
        m.region(0x2000, 0x2c);
        m.put(0x201c, 0x3000);
        m.region(0x3000, 0x3200);
        m.region(0x7000, 0xb8);
        m.put(0x700c, 0x3000);
        m.bytes.insert(0x706d, 1);
        m
    }
}
impl Memory for Fake {
    fn base(&self) -> u32 {
        0
    }
    fn tick(&self) -> u64 {
        100
    }
    fn readable(&self, a: u32, n: usize) -> bool {
        a != 0 && (0..n).all(|i| self.bytes.contains_key(&(a + i as u32)))
    }
    fn word(&self, a: u32) -> u32 {
        u32::from_le_bytes(core::array::from_fn(|i| self.byte(a + i as u32)))
    }
    fn byte(&self, a: u32) -> u8 {
        self.bytes.get(&a).copied().unwrap_or(0)
    }
    fn copy(&self, a: u32, out: &mut [u8]) -> bool {
        if !self.readable(a, out.len()) {
            return false;
        }
        for (i, b) in out.iter_mut().enumerate() {
            *b = self.byte(a + i as u32);
        }
        true
    }
}
#[test]
fn nested_collision_and_damage_keep_native_return_identity() {
    let memory = Fake::scene();
    let mut state = CaptureState::new(Some(0));
    let m = Mailbox::new();
    let mut frame = RawFrame::default();
    frame.r[1] = 0x7000;
    frame.r[2] = 0x8000;
    frame.r[13] = 0xaaaa;
    assert!(state.enter(0, &frame, 0x9000, &memory));
    frame.r[0] = 0x8000;
    frame.r[1] = 3;
    frame.r[13] = 0xbbbb;
    assert!(state.enter(1, &frame, 0x8f00, &memory));
    frame.r[0] = 123;
    assert_eq!(state.leave(&frame, 0x8f00, true, &memory, &m), Some(0xbbbb));
    let r = m.read(&mut Cursor::default()).unwrap();
    assert_eq!((r.damage, r.monster, r.part, r.actor), (123, 0x8000, 3, 0));
    assert_eq!(state.leave(&frame, 0x9000, true, &memory, &m), Some(0xaaaa));
    assert!(state.leave(&frame, 0x9000, true, &memory, &m).is_none());
}
#[test]
fn overflow_quarantines_until_unwind() {
    let memory = Fake::scene();
    let mut state = CaptureState::new(Some(0));
    let m = Mailbox::new();
    let frame = RawFrame::default();
    for i in 0..64 {
        assert!(state.enter(2, &frame, i, &memory));
    }
    assert!(!state.enter(2, &frame, 64, &memory));
    assert!(state.quarantine);
    for i in (0..64).rev() {
        assert_eq!(state.leave(&frame, i, true, &memory, &m), Some(0));
    }
    assert!(!state.quarantine);
    assert_eq!(state.depth, 0);
}
#[test]
fn thread_exit_does_not_destroy_published_records() {
    let m = Arc::new(Mailbox::new());
    let producer = m.clone();
    std::thread::spawn(move || {
        let _state = CaptureState::new(Some(0));
        producer.publish(Record {
            serial: 77,
            ..Record::default()
        });
    })
    .join()
    .unwrap();
    assert_eq!(m.read(&mut Cursor::default()).unwrap().serial, 77);
}
#[test]
fn source_policy_keeps_palico_bombs_owned() {
    assert_eq!(
        source_outcome(0x030802, Owner::LocalPalico),
        Some(Source::Attack)
    );
    assert_eq!(
        source_outcome(0x000802, Owner::LocalPlayer),
        Some(Source::Object)
    );
    assert_eq!(
        source_outcome(0x010202, Owner::Unknown),
        Some(Source::Monster)
    );
    assert_eq!(source_outcome(0x000102, Owner::Unknown), None);
}
