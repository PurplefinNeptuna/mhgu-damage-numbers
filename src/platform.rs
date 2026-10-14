//! Narrow native boundary: checked copies, process identity, clock and executable cache synchronization.
//! Forge provides addresses/singletons but has no checked mapping-query or process-cache-flush wrapper; these SVCs fill that gap.
use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use damage_numbers_core::capture::Memory;
use forge::singleton::SingletonManager;
pub static BASE: AtomicU32 = AtomicU32::new(0);
// Horizon's pseudo-handle denotes this running process; no handle ownership
// or closing is required for the mapping and cache operations below.
const CURRENT_PROCESS: u32 = 0xffff8001;
// Match the kernel's query-memory output layout exactly, including padding.
// repr(C) fixes field order and C alignment for the assembly SVC wrapper.
#[repr(C)]
#[derive(Default)]
struct MemoryInfo {
    address: u64,
    size: u64,
    kind: u32,
    attributes: u32,
    permissions: u32,
    ipc: u32,
    device: u32,
    padding: u32,
}
const _: () = assert!(core::mem::size_of::<MemoryInfo>() == 40);
unsafe extern "C" {
    fn mwc_svc_query(info: *mut MemoryInfo, page: *mut u32, address: u64) -> u32;
    fn mwc_svc_get_info(out: *mut u64, id: u32, handle: u32, sub_id: u64) -> u32;
    fn mwc_svc_flush(handle: u32, address: u64, bytes: u64) -> u32;
    pub fn nnosGetSystemTick() -> u64;
    pub fn nnosGetSystemTickFrequency() -> i64;
}
core::arch::global_asm!(include_str!("svc.S"), options(raw));
pub fn tick() -> u64 {
    // SAFETY: fixed SDK export with the verified ARM32 integer-return signature.
    unsafe { nnosGetSystemTick() }
}
// Return a shared time origin and ticks-per-second. Event ages are measured
// against this origin, so capture time and render time use the same clock.
pub fn clock() -> Option<(u64, u32)> {
    // SAFETY: fixed SDK export, no arguments or borrowed memory.
    let frequency = unsafe { nnosGetSystemTickFrequency() };
    (frequency > 0 && frequency <= u32::MAX as i64).then(|| (tick(), frequency as u32))
}
// GetInfo selector 18 supplies the program ID. Exact instruction signatures
// in hooks.rs then distinguish the expected executable code.
pub fn correct_program() -> bool {
    let mut title = 0; // SAFETY: writable output and current-process pseudo-handle match the pinned SVC ABI.
    unsafe {
        mwc_svc_get_info(&mut title, 18, CURRENT_PROCESS, 0) == 0 && title == 0x0100770008dd8000
    }
}
// Reject null/wrapped ARM32 spans, then require the whole copy to fit in one
// readable mapping. This checks address access; it does not take ownership of
// the game object or extend its lifetime beyond the synchronous native call.
fn mapped(address: u32, bytes: usize, permissions: u32) -> bool {
    if address == 0 || address as u64 + bytes as u64 > 0x1_0000_0000 {
        return false;
    }
    let mut info = MemoryInfo::default();
    let mut page = 0;
    // SAFETY: outputs are initialized, naturally aligned structs of the kernel's declared size.
    unsafe {
        mwc_svc_query(&mut info, &mut page, address as u64) == 0
            && (info.permissions & permissions) == permissions
            && info.address <= address as u64
            && address as u64 + bytes as u64 <= info.address + info.size
    }
}
pub fn readable(address: u32, bytes: usize) -> bool {
    mapped(address, bytes, 1)
}
// Horizon permission bits 1 (read) and 4 (execute) must both cover the complete
// trampoline. The RX alias belongs to Forge and remains live until process exit.
pub fn executable(address: u32, bytes: usize) -> bool {
    mapped(address, bytes, 5)
}
// After writing or restoring instructions, flush process caches and execute
// ARM barriers before those addresses can be used as code.
pub fn sync_code(address: u32, bytes: usize) -> bool {
    // SAFETY: caller supplies a validated executable span. This is the tested ARM32 process cache-flush contract.
    let result = unsafe { mwc_svc_flush(CURRENT_PROCESS, address as u64, bytes as u64) };
    // SAFETY: barriers change no memory contents and establish instruction visibility after kernel synchronization.
    unsafe { core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags)) };
    result == 0
}
// The portable capture code talks to the Memory trait. This adapter performs
// real checked reads; tests substitute a map of bytes with the same interface.
pub struct NativeMemory;
impl Memory for NativeMemory {
    fn base(&self) -> u32 {
        BASE.load(Ordering::Relaxed)
    }
    fn readable(&self, address: u32, bytes: usize) -> bool {
        readable(address, bytes)
    }
    fn word(&self, address: u32) -> u32 {
        // word requires four-byte alignment. Returning zero on a failed read lets
        // callers treat unavailable pointers/fields as missing capture evidence.
        if !readable(address, 4) || address & 3 != 0 {
            return 0;
        }
        // SAFETY: mapping and alignment were checked; volatile copies do not create a Rust reference to game-owned memory.
        unsafe { core::ptr::read_volatile(address as *const u32) }
    }
    fn byte(&self, address: u32) -> u8 {
        if !readable(address, 1) {
            return 0;
        }
        // SAFETY: the source byte is in a readable mapping and no reference escapes.
        unsafe { core::ptr::read_volatile(address as *const u8) }
    }
    fn copy(&self, address: u32, out: &mut [u8]) -> bool {
        if !readable(address, out.len()) {
            return false;
        }
        // SAFETY: the source mapping covers the whole span; the owned destination cannot overlap game memory.
        unsafe {
            core::ptr::copy_nonoverlapping(address as *const u8, out.as_mut_ptr(), out.len())
        };
        true
    }
    fn tick(&self) -> u64 {
        tick()
    }
}
/// Obtain only an identity pointer inside this short borrow. No singleton reference is cached across callbacks.
pub fn singleton(name: &str) -> Option<u32> {
    SingletonManager::get_by_name(name).map(|object| object as *mut _ as *mut c_void as u32)
}
// The save-data singleton and MtFile runtime type must both exist before the
// stock INI loader can mount storage and open files.
pub fn storage_ready() -> bool {
    use forge::mt::dti::CacheDti;
    singleton("sSavedata").is_some() && forge::mt::fs::MtFile::dti().is_some()
}
