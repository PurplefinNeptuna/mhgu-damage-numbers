//! Compile the production renderer against actual host ImGui and its original forge-rs wrappers.
extern crate alloc;
extern crate self as forge;
pub extern crate sys;
pub mod fs {
    pub mod romfs {
        pub fn mount_point() -> &'static str {
            "app"
        }
    }
}
pub mod imgui {
    pub fn get_io() -> &'static mut sys::imgui::ImGuiIO {
        unsafe { &mut *sys::imgui::igGetIO_Nil() }
    }
    pub fn get_current_context() -> &'static mut sys::imgui::ImGuiContext {
        unsafe { &mut *sys::imgui::igGetCurrentContext() }
    }
    // Modules use the same pinned wrapper implementations as the ARM plugin.
    pub mod font {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../vendor/forge-rs/forge/src/imgui/font.rs"
        ));
    }
    pub use font::ImFont;
    pub mod draw {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../vendor/forge-rs/forge/src/imgui/draw.rs"
        ));
    }
    pub use draw::DrawList;
    // forge-rs layout.rs's calc_text_size, cast for the host's signed C char.
    pub fn calc_text_size(text: &str, hide_text_after_double_hash: bool, wrap_width: f32) -> sys::imgui::ImVec2 {
        unsafe {
            sys::imgui::igCalcTextSize(
                text.as_ptr().cast(),
                text.as_ptr().add(text.len()).cast(),
                hide_text_after_double_hash,
                wrap_width,
            )
        }
    }
}
#[path = "../../../src/render.rs"]
mod render;
use core::cell::UnsafeCell;
use damage_numbers_core::{presentation::*, *};
struct State(UnsafeCell<Option<render::Resources>>);
// SAFETY: the C++ fixture invokes this interface on one main thread only.
unsafe impl Sync for State {}
static STATE: State = State(UnsafeCell::new(None));
// The host fixture observes failure deduplication without changing production logging.
static WARNINGS: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        if record.level() == log::Level::Warn {
            WARNINGS.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        }
    }
    fn flush(&self) {}
}
static LOGGER: Logger = Logger;
#[unsafe(no_mangle)]
pub extern "C" fn rs_warning_count() -> u32 {
    WARNINGS.load(core::sync::atomic::Ordering::Relaxed)
}
#[unsafe(no_mangle)]
pub extern "C" fn rs_initialize() {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Warn);
    unsafe {
        let resources = (*STATE.0.get()).get_or_insert_with(render::Resources::default);
        resources.initialize();
        resources.initialize();
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_scene(style: u32, age: f32, width: f32, height: f32, scale: f32) {
    unsafe {
        let resources = (*STATE.0.get()).as_mut().unwrap();
        let style = [Style::Wilds, Style::Classic, Style::Rise][style as usize];
        let mut model = Model::default();
        model.reset(7);
        model.configure(Settings {
            style,
            scale,
            // The r2 scenes were captured with r42's placement, which is spread 1.0.
            spread: 1.0,
            ..Settings::default()
        });
        for i in 0..9 {
            model.capture(Event {
                key: HitKey {
                    epoch: 7,
                    serial: i + 1,
                    target: 42,
                    application: 0,
                },
                position: Some(Vec3 {
                    x: (i as i32 % 3 - 1) as f32 * 0.3,
                    y: (i as i32 / 3 - 1) as f32 * 0.3,
                    z: 0.5,
                }),
                total_damage: [40, 47, 12, 6, 155, 111, 888, 1, 9999][i as usize],
                physical_hzv: if i % 3 == 0 { 44 } else { 45 },
                source: Source::Attack,
                owner: Owner::LocalPlayer,
                critical: [Critical::None, Critical::Positive, Critical::Negative][i as usize % 3],
                bounced: i == 3,
                captured_at: 0.0,
            });
        }
        model.update(age, true);
        let view = View {
            world_to_clip: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
            width,
            height,
            ..View::default()
        };
        let mut placement = Placement::default();
        let mut numbers = [DrawNumber::default(); 128];
        let count = layout(
            &model,
            &view,
            age,
            &mut numbers,
            &mut placement,
            |text, pixels, style| resources.measure(text, pixels, style),
        );
        resources.draw(&view, &numbers[..count]);
    }
}
