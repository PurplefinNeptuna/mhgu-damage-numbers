//! The production installer and upstream Patch wrapper run against a deterministic native API double.
extern crate alloc;
extern crate self as forge;
pub extern crate sys;
use core::ffi::c_void;
use std::{
    collections::BTreeMap,
    sync::{LazyLock, Mutex},
};
pub mod patch {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../vendor/forge-rs/forge/src/patch.rs"
    ));
}
pub mod mem {
    pub fn text_addr() -> u32 {
        0x10000000
    }
}
#[path = "../../../src/hooks.rs"]
mod hooks;
#[path = "../../../src/signatures.rs"]
mod signatures;
#[derive(Default)]
struct Native {
    bytes: BTreeMap<u32, u8>,
    calls: usize,
    fail: Option<(usize, bool)>,
    sync_fail: Option<usize>,
    sync_calls: usize,
    patches: usize,
}
static NATIVE: LazyLock<Mutex<Native>> = LazyLock::new(|| Mutex::new(Native::default()));
static SERIAL: Mutex<()> = Mutex::new(());
fn read(n: &Native, addr: u32, length: usize) -> Vec<u8> {
    (0..length)
        .map(|i| *n.bytes.get(&(addr + i as u32)).unwrap_or(&0))
        .collect()
}
fn write(n: &mut Native, addr: u32, bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        n.bytes.insert(addr + i as u32, *b);
    }
}
pub mod platform {
    use super::*;
    use damage_numbers_core::capture::Memory;
    pub static BASE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
    pub struct NativeMemory;
    impl Memory for NativeMemory {
        fn base(&self) -> u32 {
            mem::text_addr()
        }
        fn readable(&self, _: u32, _: usize) -> bool {
            true
        }
        fn word(&self, a: u32) -> u32 {
            u32::from_le_bytes(read(&NATIVE.lock().unwrap(), a, 4).try_into().unwrap())
        }
        fn byte(&self, a: u32) -> u8 {
            read(&NATIVE.lock().unwrap(), a, 1)[0]
        }
        fn tick(&self) -> u64 {
            0
        }
        fn copy(&self, address: u32, out: &mut [u8]) -> bool {
            let n = NATIVE.lock().unwrap();
            out.copy_from_slice(&read(&n, address, out.len()));
            true
        }
    }
    pub fn correct_program() -> bool {
        true
    }
    pub fn sync_code(_: u32, _: usize) -> bool {
        let mut n = NATIVE.lock().unwrap();
        let i = n.sync_calls;
        n.sync_calls += 1;
        n.sync_fail != Some(i)
    }
}
// SAFETY: the pinned Patch wrapper supplies valid live objects and byte spans; the mock owns the copied allocations.
#[unsafe(no_mangle)]
unsafe extern "C" fn forge_patch_create(
    addr: u32,
    bytes: *const c_void,
    length: u32,
    enable: bool,
) -> sys::patch::Patch {
    unsafe {
        let mut n = NATIVE.lock().unwrap();
        let original = read(&n, addr, length as usize).into_boxed_slice();
        let replacement = core::slice::from_raw_parts(bytes.cast(), length as usize)
            .to_vec()
            .into_boxed_slice();
        if enable {
            write(&mut n, addr, &replacement);
        }
        n.patches += 1;
        sys::patch::Patch {
            addr,
            size: length,
            original_bytes: Box::into_raw(original) as *const u8 as _,
            patch_bytes: Box::into_raw(replacement) as *const u8 as _,
            enabled: enable,
        }
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn forge_patch_enable(p: *mut sys::patch::Patch) {
    unsafe {
        let p = &mut *p;
        write(
            &mut NATIVE.lock().unwrap(),
            p.addr,
            core::slice::from_raw_parts(p.patch_bytes.cast(), p.size as usize),
        );
        p.enabled = true;
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn forge_patch_disable(p: *mut sys::patch::Patch) {
    unsafe {
        let p = &mut *p;
        if p.enabled {
            write(
                &mut NATIVE.lock().unwrap(),
                p.addr,
                core::slice::from_raw_parts(p.original_bytes.cast(), p.size as usize),
            );
            p.enabled = false;
        }
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn forge_patch_destroy(p: *mut sys::patch::Patch) {
    unsafe {
        forge_patch_disable(p);
        let p = &*p;
        drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(
            p.original_bytes as *mut u8,
            p.size as usize,
        )));
        drop(Box::from_raw(core::ptr::slice_from_raw_parts_mut(
            p.patch_bytes as *mut u8,
            p.size as usize,
        )));
        NATIVE.lock().unwrap().patches -= 1;
    }
}
#[unsafe(no_mangle)]
unsafe extern "C" fn forge_hook_create(
    target: *const c_void,
    detour: *const c_void,
    original: *mut *const c_void,
) -> sys::hook::Hook {
    unsafe {
        let mut n = NATIVE.lock().unwrap();
        let i = n.calls;
        n.calls += 1;
        let failure = n.fail.filter(|(at, _)| *at == i);
        if failure != Some((i, false)) {
            let mut branch = 0xe51ff004u32.to_le_bytes().to_vec();
            branch.extend_from_slice(&(detour as u32).to_le_bytes());
            write(&mut n, target as u32, &branch);
        }
        *original = if failure.is_some() {
            core::ptr::null()
        } else {
            (0x30000000 + 16 * i) as *const c_void
        };
        core::mem::zeroed()
    }
}
macro_rules! gateways{($($name:ident),*)=>{$(#[unsafe(no_mangle)]pub extern "C" fn $name(){})*}}
gateways!(
    dn_gateway_0,
    dn_gateway_1,
    dn_gateway_2,
    dn_gateway_3,
    dn_gateway_4,
    dn_gateway_5,
    dn_gateway_6,
    dn_gateway_7
);
fn reset(fail: Option<(usize, bool)>, sync_fail: Option<usize>) {
    let mut n = NATIVE.lock().unwrap();
    *n = Native {
        fail,
        sync_fail,
        ..Native::default()
    };
    for &(offset, bytes) in signatures::SIGNATURES {
        write(&mut n, mem::text_addr() + offset, &bytes);
    }
    hooks::ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
    for p in &hooks::ORIGINALS {
        p.store(0, core::sync::atomic::Ordering::Relaxed);
    }
}
fn originals_restored() {
    let n = NATIVE.lock().unwrap();
    assert_eq!(n.patches, 0);
    for &(offset, bytes) in signatures::SIGNATURES {
        assert_eq!(read(&n, mem::text_addr() + offset, 64), bytes);
    }
    assert!(!hooks::ACTIVE.load(core::sync::atomic::Ordering::Acquire));
}
#[test]
fn failures_before_and_after_branch_at_every_site_restore_all_bytes() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..8 {
        for published in [false, true] {
            reset(Some((i, published)), None);
            assert!(hooks::Installed::install().is_err());
            originals_restored();
        }
    }
}
#[test]
fn instruction_cache_failure_rolls_back_all_patches() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..8 {
        reset(None, Some(i));
        assert!(hooks::Installed::install().is_err());
        originals_restored();
    }
}
#[test]
fn successful_install_enables_only_after_all_eight_hooks() {
    let _guard = SERIAL.lock().unwrap();
    reset(None, None);
    let installed = hooks::Installed::install().unwrap();
    assert_eq!(installed.count(), 8);
    assert!(hooks::ACTIVE.load(core::sync::atomic::Ordering::Acquire));
    let n = NATIVE.lock().unwrap();
    assert_eq!((n.calls, n.patches), (8, 8));
    drop(n);
    drop(installed);
    hooks::ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
    originals_restored();
}
