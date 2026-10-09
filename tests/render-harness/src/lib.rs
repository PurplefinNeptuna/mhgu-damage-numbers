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
}
#[path = "../../../src/render.rs"]
mod render;
use core::cell::UnsafeCell;
use damage_numbers_core::{presentation::*, *};
struct State(UnsafeCell<Option<render::Resources>>);
// SAFETY: the C++ fixture invokes this interface on one main thread only.
unsafe impl Sync for State {}
static STATE: State = State(UnsafeCell::new(None));
#[unsafe(no_mangle)]
pub extern "C" fn rs_initialize() {
    unsafe {
        let resources = (*STATE.0.get()).get_or_insert_with(render::Resources::default);
        resources.initialize();
        resources.initialize();
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_scene(style: u32, age: f32) {
    unsafe {
        let resources = (*STATE.0.get()).as_mut().unwrap();
        let style = [Style::Wilds, Style::Classic, Style::Rise][style as usize];
        let mut model = Model::default();
        model.reset(7);
        model.configure(Settings {
            style,
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
                position: Vec3 {
                    x: (i as i32 % 3 - 1) as f32 * 0.3,
                    y: (i as i32 / 3 - 1) as f32 * 0.3,
                    z: 0.5,
                },
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
            width: 1920.0,
            height: 1080.0,
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
