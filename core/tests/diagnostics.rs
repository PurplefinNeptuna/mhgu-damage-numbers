//! Exercise the actual diagnostic formatter with a capturing logger, without Forge.
#[path = "../../src/diagnostics.rs"]
mod diagnostics;
use damage_numbers_core::{capture::Record, presentation::*, *};
use std::sync::Mutex;

struct Logger;
static LINES: Mutex<Vec<String>> = Mutex::new(Vec::new());
impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool { true }
    fn log(&self, record: &log::Record) {
        assert_eq!(record.level(), log::Level::Error);
        LINES.lock().unwrap().push(record.args().to_string());
    }
    fn flush(&self) {}
}
#[test]
fn diagnostics_deduplicate_and_stop_at_process_limits() {
    log::set_logger(&Logger).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    let mut diagnostics = diagnostics::Diagnostics::default();
    let view = View::screen(1280.0, 720.0).unwrap();
    // Include extreme finite inputs to check Forge's fixed-size message boundary.
    let mut camera = Record { kind: 3, view: [f32::MAX; 16], projection: [f32::MIN_POSITIVE; 16], ..Record::default() };
    for i in 0..100 {
        let display = [1280.0 + i as f32, 720.0, 1.0, 1.0, 0.0, 0.0, 1280.0, 720.0];
        diagnostics.display(display);
        diagnostics.display(display);
        camera.actor = i;
        diagnostics.camera(&camera);
        diagnostics.camera(&camera);
        let event = Event { key: HitKey { serial: i, ..HitKey::default() }, total_damage: 54,
            position: Some(Vec3 { x: 0.0, y: 0.0, z: 0.5 }), ..Event::default() };
        let number = DrawNumber { key: event.key, ..DrawNumber::default() };
        diagnostics.hit(event, &camera, &view, false, 100, Some(&number));
        diagnostics.hit(event, &camera, &view, false, 100, None);
    }
    let lines = LINES.lock().unwrap();
    assert_eq!(lines.iter().filter(|s| s.starts_with("[DN-DIAG] camera#")).count(), 16);
    assert_eq!(lines.iter().filter(|s| s.starts_with("[DN-DIAG] hit#")).count(), 32);
    assert_eq!(lines.iter().filter(|s| s.starts_with("[DN-DIAG] display#")).count(), 4);
    assert_eq!(lines.iter().filter(|s| s.contains("limit reached")).count(), 2);
    assert!(lines.iter().any(|s| s.contains("fallback=true")));
    assert!(lines.iter().any(|s| s.contains("native_camera_used=false")));
    assert!(lines.iter().all(|s| s.len() < 500), "Forge writes at most 511 message bytes");
    drop(lines);
    LINES.lock().unwrap().clear();
    let mut diagnostics = diagnostics::Diagnostics::default();
    diagnostics.camera(&Record::default());
    assert!(LINES.lock().unwrap().is_empty());
    diagnostics.camera(&camera);
    camera.tick += 1;
    diagnostics.camera(&camera);
    camera.attack = 0x12340000;
    diagnostics.camera(&camera);
    let mut view = view;
    view.world_to_clip = [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.];
    let event = Event { position: Some(Vec3 { x: 0., y: 0., z: 0.5 }), ..Event::default() };
    diagnostics.hit(event, &camera, &view, true, 101, None);
    let lines = LINES.lock().unwrap();
    assert_eq!(lines.iter().filter(|s| s.starts_with("[DN-DIAG] camera#")).count(), 2);
    assert!(lines.iter().any(|s| s.contains("fallback=false")));
    assert!(lines.iter().any(|s| s.contains("native_camera_used=true")));
    assert!(lines.iter().any(|s| s.contains("text omitted by layout")));
}
