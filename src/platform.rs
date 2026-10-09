//! Narrow native boundary: checked copies, process identity, clock and executable cache synchronization.
//! Forge provides addresses/singletons but has no checked mapping-query or process-cache-flush wrapper; these SVCs fill that gap.
use core::ffi::c_void;
use core::sync::atomic::{AtomicU32, Ordering};
use damage_numbers_core::capture::Memory;
use forge::singleton::SingletonManager;
pub static BASE: AtomicU32 = AtomicU32::new(0);
const CURRENT_PROCESS: u32 = 0xffff8001;
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
pub fn clock() -> Option<(u64, u32)> {
    // SAFETY: fixed SDK export, no arguments or borrowed memory.
    let frequency = unsafe { nnosGetSystemTickFrequency() };
    (frequency > 0 && frequency <= u32::MAX as i64).then(|| (tick(), frequency as u32))
}
pub fn correct_program() -> bool {
    let mut title = 0; // SAFETY: writable output and current-process pseudo-handle match the pinned SVC ABI.
    unsafe {
        mwc_svc_get_info(&mut title, 18, CURRENT_PROCESS, 0) == 0 && title == 0x0100770008dd8000
    }
}
pub fn readable(address: u32, bytes: usize) -> bool {
    if address == 0 || address as u64 + bytes as u64 > 0x1_0000_0000 {
        return false;
    }
    let mut info = MemoryInfo::default();
    let mut page = 0;
    // SAFETY: outputs are initialized, naturally aligned structs of the kernel's declared size.
    unsafe {
        mwc_svc_query(&mut info, &mut page, address as u64) == 0
            && (info.permissions & 1) != 0
            && info.address <= address as u64
            && address as u64 + bytes as u64 <= info.address + info.size
    }
}
pub fn sync_code(address: u32, bytes: usize) -> bool {
    // SAFETY: caller supplies a validated executable span. This is the tested ARM32 process cache-flush contract.
    let result = unsafe { mwc_svc_flush(CURRENT_PROCESS, address as u64, bytes as u64) };
    // SAFETY: barriers change no memory contents and establish instruction visibility after kernel synchronization.
    unsafe { core::arch::asm!("dsb sy", "isb", options(nostack, preserves_flags)) };
    result == 0
}
pub struct NativeMemory;
impl Memory for NativeMemory {
    fn base(&self) -> u32 {
        BASE.load(Ordering::Relaxed)
    }
    fn readable(&self, address: u32, bytes: usize) -> bool {
        readable(address, bytes)
    }
    fn word(&self, address: u32) -> u32 {
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
pub fn storage_ready() -> bool {
    use forge::mt::dti::CacheDti;
    singleton("sSavedata").is_some() && forge::mt::fs::MtFile::dti().is_some()
}
