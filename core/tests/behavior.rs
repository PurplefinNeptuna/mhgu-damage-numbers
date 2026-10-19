//! Behavior tests use copied records and fake game memory; no emulator or installed package is involved.
use damage_numbers_core::{capture::*, mailbox::*, presentation::*, *};
use std::{collections::BTreeMap, sync::Arc};
fn hit(serial: u32) -> Event {
    Event {
        key: HitKey {
            epoch: 1,
            serial,
            target: 2,
            application: 0,
        },
        position: Some(Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.5,
        }),
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
    let mut memory = Fake::scene();
    memory.region(0xb000, 12);
    let mut state = CaptureState::new(Some(0));
    let m = Mailbox::new();
    let mut frame = RawFrame::default();
    frame.r[1] = 0x7000;
    frame.r[2] = 0x8000;
    frame.r[3] = 0xb000;
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
fn collision_position_failures_request_centre_without_losing_native_scope() {
    // Keep a valid outer collision live: rejecting an inner position must not
    // lose its return scope or fall back to the outer collision's coordinates.
    for position in [
        None,
        Some([f32::NAN, 0.0, 0.0]),
        Some([0.0, f32::INFINITY, 0.0]),
        Some([0.0, 0.0, f32::NEG_INFINITY]),
        Some([0.0, 0.0, 0.0]),
        Some([1.0, 2.0, 3.0]),
    ] {
        let mut memory = Fake::scene();
        memory.region(0xb000, 12);
        for (i, value) in [10.0f32, 20.0, 30.0].into_iter().enumerate() {
            memory.put(0xb000 + 4 * i as u32, value.to_bits());
        }
        if let Some(values) = position {
            memory.region(0xc000, 12);
            for (i, value) in values.into_iter().enumerate() {
                memory.put(0xc000 + 4 * i as u32, value.to_bits());
            }
        }
        let mut state = CaptureState::new(Some(0));
        let mailbox = Mailbox::new();
        let mut frame = RawFrame::default();
        frame.r[1] = 0x7000;
        frame.r[2] = 0x8000;
        frame.r[3] = 0xb000;
        frame.r[13] = 0xaaaa;
        assert!(state.enter(0, &frame, 0xa000, &memory));
        frame.r[3] = 0xc000;
        frame.r[13] = 0xbbbb;
        assert!(state.enter(0, &frame, 0x9000, &memory));
        frame.r[0] = 0x8000;
        frame.r[1] = 3;
        frame.r[13] = 0xcccc;
        assert!(state.enter(1, &frame, 0x8f00, &memory));
        frame.r[0] = 123;
        assert_eq!(
            state.leave(&frame, 0x8f00, true, &memory, &mailbox),
            Some(0xcccc)
        );
        let mut cursor = Cursor::default();
        let record = mailbox.read(&mut cursor).unwrap();
        if let Some(values) = position.filter(|v| v.iter().all(|n| n.is_finite())) {
            assert_eq!(
                record.position,
                Vec3 {
                    x: values[0],
                    y: values[1],
                    z: values[2]
                }
            );
        } else {
            assert!(
                !record.position.valid(),
                "failed capture became world origin"
            );
        }
        let event = Event {
            position: record.position.valid().then_some(record.position),
            ..hit(1)
        };
        assert_eq!(
            event.position.is_some(),
            position.is_some_and(|v| v.iter().all(|n| n.is_finite()))
        );
        let mut model = Model::default();
        model.reset(1);
        assert!(model.capture(event));
        assert_eq!(
            state.leave(&frame, 0x9000, true, &memory, &mailbox),
            Some(0xbbbb)
        );
        frame.r[0] = 0x8000;
        frame.r[13] = 0xdddd;
        assert!(state.enter(1, &frame, 0x9f00, &memory));
        frame.r[0] = 456;
        assert_eq!(
            state.leave(&frame, 0x9f00, true, &memory, &mailbox),
            Some(0xdddd)
        );
        assert_eq!(
            mailbox.read(&mut cursor).unwrap().position,
            Vec3 {
                x: 10.0,
                y: 20.0,
                z: 30.0
            }
        );
        assert_eq!(
            state.leave(&frame, 0xa000, true, &memory, &mailbox),
            Some(0xaaaa)
        );
        assert_eq!(state.depth, 0);
    }
}
#[test]
fn native_projection_preserves_positions_across_display_scales() {
    // Row-vector rotation/translation gives (-0.1, 0.1, 0.6, 1), then
    // perspective gives (-0.2, 0.3, 0.6, 1.6). Reversing matrix order differs.
    let view = [
        0., 1., 0., 0., -1., 0., 0., 0., 0., 0., 1., 0., 0.2, -0.1, 0.2, 1.,
    ];
    let projection = [
        2., 0., 0., 0., 0., 3., 0., 0., 0., 0., 1., 1., 0., 0., 0., 1.,
    ];
    for rect in [[0, 0, 1280, 720], [0, 0, 1920, 1080]] {
        for (width, height) in [(1280., 720.), (1600., 900.), (1920., 1080.), (2560., 1440.)] {
            let camera = native_camera(&view, &projection, rect, width, height).unwrap();
            let point = project(
                Vec3 {
                    x: 0.2,
                    y: 0.3,
                    z: 0.4,
                },
                &camera,
            )
            .unwrap();
            assert!((point.x / width - 0.4375).abs() < 0.00001);
            assert!((point.y / height - 0.40625).abs() < 0.00001);
        }
    }
}
#[test]
fn applied_crop_is_checked_and_recopied_across_mode_changes() {
    let renderer = 0x1000;
    let mut memory = Fake::new();
    memory.region(renderer + 0xb0, 16);
    // Requested dimensions can differ from the crop that has actually been applied.
    memory.put(renderer + 0xb0, 9999);
    memory.put(renderer + 0xb4, 9999);
    for size in [[1280, 720], [1920, 1080], [1280, 720], [1600, 900]] {
        memory.put(renderer + 0xb8, size[0]);
        memory.put(renderer + 0xbc, size[1]);
        assert_eq!(visible_size(&memory, renderer, 1920., 1080.), Some(size.map(|n| n as f32)));
    }
    for size in [[0, 720], [1280, 0], [u32::MAX, 720], [1280, u32::MAX], [1921, 1080], [1920, 1081]] {
        memory.put(renderer + 0xb8, size[0]);
        memory.put(renderer + 0xbc, size[1]);
        assert!(visible_size(&memory, renderer, 1920., 1080.).is_none());
    }
    memory.put(renderer + 0xb8, 1280);
    memory.put(renderer + 0xbc, 720);
    for backing in [[0., 1080.], [-1., 1080.], [1920., f32::NAN], [f32::INFINITY, 1080.]] {
        assert!(visible_size(&memory, renderer, backing[0], backing[1]).is_none());
    }
    assert!(visible_size(&memory, 0, 1920., 1080.).is_none());
    assert!(visible_size(&memory, u32::MAX - 0xb8, 1920., 1080.).is_none());
    memory.bytes.remove(&(renderer + 0xbf));
    assert!(visible_size(&memory, renderer, 1920., 1080.).is_none());
}
#[test]
fn crop_drives_projection_fallback_and_all_style_geometry() {
    let renderer = 0x1000;
    let mut memory = Fake::new();
    memory.region(renderer + 0xb8, 8);
    let identity = [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.];
    let point = Vec3 { x: 0., y: 0., z: 0.5 };
    // Reproduce the old lower-right centre when the backing size is used in handheld.
    let wrong = native_camera(&identity, &identity, [0,0,1280,720], 1920., 1080.).unwrap();
    assert_eq!(project(point, &wrong), Some(Point { x: 960., y: 540. }));
    for (width, height) in [(1280,720), (1920,1080), (1600,900), (1280,720)] {
        memory.put(renderer + 0xb8, width);
        memory.put(renderer + 0xbc, height);
        let [w,h] = visible_size(&memory, renderer, 1920., 1080.).unwrap();
        let screen = View::screen(w,h).unwrap();
        for rect in [[0,0,1280,720], [0,0,1920,1080]] {
            let view = native_camera(&identity, &identity, rect, w,h).unwrap();
            assert_eq!(project(point, &view), Some(Point { x: w/2., y: h/2. }));
            for style in [Style::Wilds, Style::Rise, Style::Classic] {
                let projected = draw(Some(point), &view, style);
                let fallback = draw(None, &screen, style);
                let reference = draw(Some(point), &View { world_to_clip: identity, ..screen }, style);
                assert_eq!(projected.position, fallback.position);
                assert_eq!(projected.position, reference.position);
                assert_eq!(projected.pixels, reference.pixels);
                assert_eq!(projected.reserve, reference.reserve);
                assert_eq!(projected.slash.origin, reference.slash.origin);
                assert_eq!(projected.slash.clip, reference.slash.clip);
            }
        }
    }
}
fn draw(position: Option<Vec3>, view: &View, style: Style) -> DrawNumber {
    let mut model = Model::default();
    model.reset(1);
    model.configure(Settings {
        style,
        ..Settings::default()
    });
    assert!(model.capture(Event {
        position,
        critical: Critical::Positive,
        ..hit(1)
    }));
    model.update(0.2, true);
    let mut output = [DrawNumber::default(); 128];
    assert_eq!(
        layout(
            &model,
            view,
            0.2,
            &mut output,
            &mut Placement::default(),
            |text, pixels, _| {
                let width = text.iter().take_while(|&&n| n != 0).count() as f32 * pixels * 0.55;
                Some(TextMetrics {
                    advance: width,
                    ink: Rect {
                        left: 0.0,
                        top: 0.0,
                        right: width,
                        bottom: pixels,
                    },
                })
            }
        ),
        1
    );
    output[0]
}
#[test]
fn unavailable_or_clipped_projection_uses_screen_centre_in_every_style() {
    let id = [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
    for (width, height) in [(1280., 720.), (1600., 900.), (1920., 1080.)] {
        let camera = native_camera(&id, &id, [0, 0, 1280, 720], width, height).unwrap();
        let no_camera = View::screen(width, height).unwrap();
        for style in [Style::Wilds, Style::Rise, Style::Classic] {
            let centre = draw(None, &camera, style);
            assert!((centre.position.x + centre.advance * 0.5 - width * 0.5).abs() < 1.0);
            assert!((centre.position.y - height * 0.5).abs() < height * 0.1);
            for position in [
                Vec3 {
                    x: 2.0,
                    y: 0.0,
                    z: 0.5,
                },
                Vec3 {
                    x: 0.0,
                    y: -2.0,
                    z: 0.5,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: 2.0,
                },
            ] {
                let fallback = draw(Some(position), &camera, style);
                assert_eq!(fallback.position, centre.position);
                assert_eq!(fallback.slash.origin, centre.slash.origin);
            }
            let fallback = draw(hit(1).position, &no_camera, style);
            assert_eq!(fallback.position, centre.position);
            let mut invalid = camera;
            invalid.world_to_clip[0] = f32::NAN;
            assert_eq!(
                draw(hit(1).position, &invalid, style).position,
                centre.position
            );
            let mut behind = camera;
            behind.world_to_clip[15] = -1.0;
            assert_eq!(
                draw(hit(1).position, &behind, style).position,
                centre.position
            );
        }
    }
    for (width, height) in [
        (0.0, 720.0),
        (1280.0, -1.0),
        (f32::NAN, 900.0),
        (1600.0, f32::INFINITY),
    ] {
        assert!(View::screen(width, height).is_none());
    }
    assert!(!visible(
        Event {
            position: Some(Vec3 {
                x: f32::NAN,
                y: 0.0,
                z: 0.0
            }),
            ..hit(1)
        },
        Settings::default()
    ));
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
#[test]
fn kinsect_hits_resolve_to_their_hunter() {
    // main+0x4816c0 rewrites the shell hit to primary 6, keeps subtype 1 and
    // stores the owning hunter at +0x70 with secondary kind 0.
    let mut memory = Fake::scene();
    memory.bytes.insert(0x7010, 6);
    memory.region(0x7070, 4);
    memory.put(0x7070, 0x3000);
    assert_eq!(attack_owner(&memory, 0x7000), Owner::LocalPlayer);
    assert_eq!(source_outcome(0x000106, Owner::LocalPlayer), Some(Source::Attack));
    assert_eq!(source_outcome(0x000106, Owner::Unknown), None);
    assert_eq!(source_outcome(0x000206, Owner::LocalPlayer), None);
}
// A monster at 0x9000 whose work block (+0x1428) is 0x20000, rooted at (1, 2, 3).
fn monster_scene() -> Fake {
    let mut m = Fake::scene();
    m.region(0x9000, 0x7600);
    m.put(0x9000 + 0x1428, 0x20000);
    m.region(0x20000, 0x5e10);
    for (i, v) in [1.0f32, 2.0, 3.0].into_iter().enumerate() {
        m.put(0x20040 + 4 * i as u32, v.to_bits());
    }
    m
}
fn only(mailbox: &Mailbox) -> Record {
    let mut cursor = Cursor::default();
    let record = mailbox.read(&mut cursor).expect("one record");
    assert!(mailbox.read(&mut cursor).is_none());
    record
}
#[test]
fn poison_tick_publishes_the_stored_amount_on_return() {
    let mut memory = monster_scene();
    let mut state = CaptureState::new(Some(0));
    let mailbox = Mailbox::new();
    let mut frame = RawFrame::default();
    frame.r[0] = 0x9000;
    frame.r[13] = 0xcccc;
    // No tick this frame: the native function leaves work+0x5d66 at zero.
    assert!(state.enter(8, &frame, 0x8000, &memory));
    assert_eq!(state.leave(&frame, 0x8000, true, &memory, &mailbox), Some(0xcccc));
    assert!(mailbox.read(&mut Cursor::default()).is_none());
    memory.put(0x20000 + 0x5d64, 37 << 16);
    assert!(state.enter(8, &frame, 0x8000, &memory));
    assert_eq!(state.leave(&frame, 0x8000, true, &memory, &mailbox), Some(0xcccc));
    let record = only(&mailbox);
    assert_eq!((record.kind, record.monster, record.damage), (1, 0x9000, 37));
    assert_eq!(record.feedback[0].reserved, Source::Poison as u8);
    assert_eq!(record.position, Vec3 { x: 1.0, y: 2.0, z: 3.0 });
    assert_eq!(record.player, 0x3000);
}
#[test]
fn blast_and_mount_taps_read_their_native_amounts() {
    let mut memory = monster_scene();
    // FUN_00a3a078: *(monster+0x75ec)+100; blast table at +0x54, amount at +0xc.
    memory.put(0x9000 + 0x75ec, 0xc000);
    memory.region(0xc000, 0x100);
    memory.region(0xd000, 0x10);
    let mut state = CaptureState::new(Some(0));
    let mut frame = RawFrame::default();
    frame.r[0] = 0x9000;
    let mailbox = Mailbox::new();
    state.tap(9, &frame, 0, &memory, &mailbox);
    assert!(mailbox.read(&mut Cursor::default()).is_none(), "null blast table");
    memory.put(0xc000 + 100 + 0x54, 0xd000);
    memory.put(0xd00c, 120);
    state.tap(9, &frame, 0, &memory, &mailbox);
    let record = only(&mailbox);
    assert_eq!(record.damage, 120);
    assert_eq!(record.feedback[0].reserved, Source::Blast as u8);
    // Mount finisher: r4 is the amount and [sp+60] is monster+0x1428.
    let mailbox = Mailbox::new();
    memory.region(0xe000, 0x40);
    memory.put(0xe03c, 0x9000 + 0x1428);
    frame.r[4] = 350;
    state.tap(10, &frame, 0xe000, &memory, &mailbox);
    let record = only(&mailbox);
    assert_eq!((record.monster, record.damage), (0x9000, 350));
    assert_eq!(record.feedback[0].reserved, Source::Status as u8);
}
#[test]
fn remote_hits_are_published_for_accounting_but_not_for_bounces() {
    let mut memory = Fake::scene();
    // A second hunter in table slot 1 owns this direct hit.
    memory.region(0x2020, 4);
    memory.put(0x2020, 0x4000);
    memory.region(0x4000, 4);
    memory.put(0x700c, 0x4000);
    memory.region(0x5000, 12);
    let mut state = CaptureState::new(Some(0));
    let mailbox = Mailbox::new();
    let mut frame = RawFrame::default();
    frame.r[1] = 0x7000;
    frame.r[2] = 0x6000;
    frame.r[3] = 0x5000;
    assert!(state.enter(0, &frame, 0x9000, &memory));
    frame.r[0] = 0x6000;
    assert!(state.enter(1, &frame, 0x8f00, &memory));
    frame.r[0] = 55;
    state.leave(&frame, 0x8f00, true, &memory, &mailbox);
    state.leave(&frame, 0x9000, true, &memory, &mailbox);
    let record = only(&mailbox);
    assert_eq!((record.actor, record.damage), (Owner::RemotePlayer as u32, 55));
    assert_eq!(state.last_damage.serial, 0);
}
#[test]
fn status_is_always_visible_and_other_needs_opt_in() {
    let mut e = hit(1);
    e.owner = Owner::Unknown;
    e.source = Source::Status;
    let mut s = Settings::default();
    assert!(visible(e, s));
    e.source = Source::Other;
    assert!(!visible(e, s));
    s.other = true;
    assert!(visible(e, s));
}
#[test]
fn monster_list_copies_hp_and_root() {
    let mut memory = monster_scene();
    memory.region(0x1851fe0, 4);
    memory.put(0x1851fe0, 0xb000);
    memory.region(0xb000, 0x1c + 40);
    memory.put(0xb000 + 0x1c + 4, 0x9000);
    memory.put(0x20370, 900);
    memory.put(0x20374, 1000);
    assert_eq!(capture::monster_vital(&memory, 0), None);
    assert_eq!(
        capture::monster_vital(&memory, 1),
        Some((0x9000, 900, 1000, Vec3 { x: 1.0, y: 2.0, z: 3.0 }))
    );
    assert_eq!(capture::monster_vital(&memory, MONSTERS), None);
}
#[test]
fn vitals_report_only_settled_unexplained_hp_loss() {
    let mut v = Vitals::default();
    assert_eq!(v.observe(0, 0x9000, 1000, 1000, 0.0), None, "baseline");
    // A captured hit before the HP drop explains it completely.
    v.expect(0x9000, 100, 0.1);
    assert_eq!(v.observe(0, 0x9000, 900, 1000, 0.1), None);
    assert_eq!(v.observe(0, 0x9000, 900, 1000, 0.5), None);
    // A late record repays a drop inside the settle window.
    assert_eq!(v.observe(0, 0x9000, 850, 1000, 1.0), None);
    v.expect(0x9000, 50, 1.05);
    assert_eq!(v.observe(0, 0x9000, 850, 1000, 1.2), None);
    // Partly explained loss reports only the remainder after settling.
    v.expect(0x9000, 30, 2.0);
    assert_eq!(v.observe(0, 0x9000, 780, 1000, 2.0), None);
    assert_eq!(v.observe(0, 0x9000, 780, 1000, 2.1), None);
    assert_eq!(v.observe(0, 0x9000, 780, 1000, 2.2), Some(40));
    // Damage the game never subtracts (an HP floor) expires instead of hiding later loss.
    v.expect(0x9000, 500, 3.0);
    assert_eq!(v.observe(0, 0x9000, 780, 1000, 4.1), None);
    assert_eq!(v.observe(0, 0x9000, 700, 1000, 4.2), None);
    assert_eq!(v.observe(0, 0x9000, 700, 1000, 4.4), Some(80));
    // Healing, a different monster or a new maximum only move the baseline.
    assert_eq!(v.observe(0, 0x9000, 750, 1000, 5.0), None);
    assert_eq!(v.observe(0, 0x9100, 10, 2000, 5.1), None);
    assert_eq!(v.observe(0, 0x9100, 10, 2000, 6.0), None);
    assert_eq!(v.observe(0, 0, 0, 0, 6.1), None);
    assert_eq!(v.observe(MONSTERS, 0x9000, 1, 1, 6.2), None);
}
#[test]
fn poison_is_gray_and_blast_uses_the_weak_point_colour() {
    let mut e = hit(1);
    e.physical_hzv = 10;
    e.source = Source::Poison;
    assert_eq!(tint(e), Tint::Gray);
    e.source = Source::Blast;
    assert_eq!(tint(e), Tint::Weak);
    e.source = Source::Status;
    assert_eq!(tint(e), Tint::Weak);
    e.owner = Owner::Unknown;
    for source in [Source::Poison, Source::Blast, Source::Status] {
        e.source = source;
        assert!(visible(e, Settings::default()));
    }
}
#[test]
fn map_fired_shells_show_as_object_damage() {
    // Shell 0x8000 keeps secondary kind 2; its owner info at 0xa400 names
    // character 0xb000, the map's uStageShellOwner (type 9 at +0x1052).
    let mut memory = Fake::scene();
    memory.bytes.insert(0x7010, 2);
    memory.bytes.insert(0x706d, 8);
    memory.bytes.insert(0x706e, 2);
    memory.region(0x7070, 4);
    memory.put(0x7070, 0x8000);
    memory.region(0x8000, 0x1370);
    memory.put(0x8000 + 0x136c, 0xa400);
    memory.region(0xa400, 0x10);
    memory.put(0xa40c, 0xb000);
    memory.region(0xb000, 0x1053);
    memory.put(0xb00c, 1);
    assert_eq!(attack_owner(&memory, 0x7000), Owner::Unknown, "type 0 is not the map");
    memory.bytes.insert(0xb000 + 0x1052, 9);
    assert_eq!(attack_owner(&memory, 0x7000), Owner::Stage);
    assert_eq!(source_outcome(0x020802, Owner::Stage), Some(Source::Object));
    assert_eq!(source_outcome(0x020802, Owner::Unknown), None);
    memory.put(0xb00c, 0);
    assert_eq!(attack_owner(&memory, 0x7000), Owner::Unknown, "inactive owner");
}
