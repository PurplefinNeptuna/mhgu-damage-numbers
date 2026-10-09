//! Damage Numbers r1: Forge callbacks own lifecycle; native gateways only copy evidence.
#![no_std]
extern crate alloc;
mod abi;
mod config;
mod hooks;
mod platform;
mod render;
mod runtime;
mod signatures;
core::arch::global_asm!(include_str!("gateway.S"), options(raw));

#[forge::entry]
fn start() {
    runtime::start();
}
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
#[forge::imgui_render]
fn panel() {
    runtime::panel();
}
#[forge::imgui_free_render]
fn overlay() {
    runtime::render();
}
