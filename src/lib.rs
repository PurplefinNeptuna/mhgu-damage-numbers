//! Damage Numbers r38: out-of-view hits stay hidden until seen, map-fired siege shells, kinsect, gray poison, weak-point-coloured blast and mount knockdown, plus opt-in unattributed HP loss.
// The Switch plugin has no Rust standard library. alloc provides owned buffers
// through Forge's allocator; portable logic lives in the separate core crate.
#![no_std]
extern crate alloc;
mod abi;
mod config;
mod hooks;
mod platform;
mod render;
mod runtime;
mod signatures;
// These gateways are ARM instructions assembled into the plugin. Their saved
// register layout is shared with capture::RawFrame and checked by verify.
core::arch::global_asm!(include_str!("gateway.S"), options(raw));

// Forge supplies a 70-byte output record; its Rust binding names only the
// six-byte version prefix. Keep the native name and required loader explicit.
#[repr(C)]
struct PluginMetadata {
    version: forge::ForgeVersion,
    name: [u8; 64],
}
const _: () = assert!(core::mem::size_of::<PluginMetadata>() == 70);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn forge_onLoad(params: *mut forge::PluginInitParams) {
    if params.is_null() { return; }
    let mut name = [0; 64];
    let label = b"Damage Numbers r38";
    name[..label.len()].copy_from_slice(label);
    // SAFETY: Forge owns the writable native metadata record for this call.
    unsafe { params.cast::<PluginMetadata>().write(PluginMetadata {
        version: forge::ForgeVersion { major: 1, minor: 5, patch: 1 }, name,
    }); }
}
#[unsafe(no_mangle)]
pub extern "C" fn forge_onInit() {
    forge::log::init().expect("Failed to initialize logger");
    runtime::start();
}
// no_mangle keeps the name Forge looks up, and extern "C" selects the native
// calling convention. This callback can load settings before ImGui is ready.
#[unsafe(no_mangle)]
pub extern "C" fn forge_onUpdate(_delta: f32) {
    runtime::update();
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn forge_onImGuiInit(
    ctx: *mut core::ffi::c_void,
    alloc: *mut core::ffi::c_void,
    free: *mut core::ffi::c_void,
) {
    // SAFETY: Forge supplies its live context and matching allocator callbacks. These references remain scoped to initialization.
    unsafe {
        let context = &mut *(ctx as *mut forge::sys::imgui::ImGuiContext);
        forge::imgui::set_current_context(context);
        // ImGui must use the same allocation/free pair as Forge. These raw function
        // addresses come from the loader; transmute assigns their declared ABI types.
        let alloc_fn = core::mem::transmute::<
            *mut core::ffi::c_void,
            forge::sys::imgui::ImGuiMemAllocFunc,
        >(alloc);
        let free_fn = core::mem::transmute::<
            *mut core::ffi::c_void,
            forge::sys::imgui::ImGuiMemFreeFunc,
        >(free);
        forge::imgui::set_allocator_functions(alloc_fn, free_fn);
    }
    runtime::init_imgui();
}
// Forge calls this inside Plugin UI, so it owns only the settings row.
#[forge::imgui_render]
fn panel() {
    runtime::panel();
}
// The free-render callback draws every eligible frame even when the row is closed.
#[forge::imgui_free_render]
fn overlay() {
    runtime::render();
}
