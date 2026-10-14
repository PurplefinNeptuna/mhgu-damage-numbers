//! Compiler-derived native layouts retained in the NRO so verification checks the shipped target build.
core::arch::global_asm!(".hidden dn_abi");
use core::mem::{align_of, offset_of, size_of};
use damage_numbers_core::capture::RawFrame;
use forge::sys::{hook::Hook, imgui::*};
// used keeps this verification table even though gameplay never reads it.
// Each entry is calculated by the ARM Rust compiler, in the same order as the
// C++ table in scripts/verify.py. A mismatched field offset fails verification.
#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".rodata.dn_abi")]
pub static dn_abi: [u32; 30] = [
    size_of::<Hook>() as u32,
    align_of::<Hook>() as u32,
    offset_of!(Hook, target) as u32,
    offset_of!(Hook, detour) as u32,
    offset_of!(Hook, original) as u32,
    offset_of!(Hook, context) as u32,
    offset_of!(Hook, ctx_cell) as u32,
    size_of::<RawFrame>() as u32,
    offset_of!(RawFrame, cpsr) as u32,
    offset_of!(RawFrame, fpscr) as u32,
    offset_of!(RawFrame, high_d) as u32,
    offset_of!(RawFrame, low_d) as u32,
    offset_of!(RawFrame, r) as u32,
    offset_of!(ImGuiIO, Fonts) as u32,
    offset_of!(ImGuiIO, BackendFlags) as u32,
    offset_of!(ImFontConfig, FontDataOwnedByAtlas) as u32,
    offset_of!(ImFontConfig, Flags) as u32,
    offset_of!(ImFontBaked, Size) as u32,
    size_of::<ImFontGlyph>() as u32,
    offset_of!(ImFontGlyph, AdvanceX) as u32,
    offset_of!(ImFontGlyph, X0) as u32,
    size_of::<ImTextureData>() as u32,
    offset_of!(ImTextureData, Pixels) as u32,
    offset_of!(ImTextureData, Status) as u32,
    offset_of!(ImTextureData, RefCount) as u32,
    size_of::<ImGuiContextHook>() as u32,
    offset_of!(ImGuiContextHook, Callback) as u32,
    offset_of!(ImGuiContextHook, UserData) as u32,
    size_of::<ImDrawVert>() as u32,
    offset_of!(ImDrawList, VtxBuffer) as u32,
];
