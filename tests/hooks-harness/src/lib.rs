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
    partial: Option<usize>,
    corrupt_trampoline: Option<usize>,
    mismatch_return: Option<usize>,
    foreign: Option<(usize, u32)>,
    restore_fail: Option<u32>,
    wrong_program: bool,
    deny_execute: Option<u32>,
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
        !NATIVE.lock().unwrap().wrong_program
    }
    pub fn executable(address: u32, bytes: usize) -> bool {
        let n = NATIVE.lock().unwrap();
        n.deny_execute != Some(address)
            && (0..bytes).all(|at| n.bytes.contains_key(&(address + at as u32)))
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
        if p.enabled && NATIVE.lock().unwrap().restore_fail != Some(p.addr) {
            write(
                &mut NATIVE.lock().unwrap(),
                p.addr,
                core::slice::from_raw_parts(p.original_bytes.cast(), p.size as usize),
            );
        }
        p.enabled = false;
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
        let address = target as u32;
        let continuation = 0x30000000 + 16 * i as u32;
        // Match Forge's pooled ARM trampoline rather than returning an opaque
        // address: production verifies displaced bytes and the final jump.
        let mut trampoline = read(&n, address, 8);
        trampoline.extend_from_slice(&0xe51ff004u32.to_le_bytes());
        trampoline.extend_from_slice(&(address + 8).to_le_bytes());
        if n.corrupt_trampoline == Some(i) {
            trampoline[0] ^= 1;
        }
        write(&mut n, continuation, &trampoline);
        if failure != Some((i, false)) {
            let mut branch = 0xe51ff004u32.to_le_bytes().to_vec();
            branch.extend_from_slice(&(detour as u32).to_le_bytes());
            let bytes = if n.partial == Some(i) {
                &branch[..4]
            } else {
                &branch
            };
            write(&mut n, address, bytes);
        }
        if let Some((at, foreign_address)) = n.foreign {
            if at == i {
                write(&mut n, foreign_address, &0xe1a00000u32.to_le_bytes());
            }
        }
        *original = if failure.is_some() {
            core::ptr::null()
        } else {
            continuation as *const c_void
        };
        let mut hook: sys::hook::Hook = core::mem::zeroed();
        hook.target = target;
        hook.detour = detour;
        hook.original = if n.mismatch_return == Some(i) {
            core::ptr::null()
        } else {
            *original
        };
        hook
    }
}
// Host RET stubs explicitly satisfy the ARM recipe's four-byte address alignment.
core::arch::global_asm!(
    r#"
.text
.balign 4
.global dn_gateway_0
dn_gateway_0:
ret
.balign 4
.global dn_gateway_1
dn_gateway_1:
ret
.balign 4
.global dn_gateway_2
dn_gateway_2:
ret
.balign 4
.global dn_gateway_3
dn_gateway_3:
ret
.balign 4
.global dn_gateway_4
dn_gateway_4:
ret
.balign 4
.global dn_gateway_5
dn_gateway_5:
ret
.balign 4
.global dn_gateway_6
dn_gateway_6:
ret
.balign 4
.global dn_gateway_7
dn_gateway_7:
ret
.balign 4
.global dn_gateway_8
dn_gateway_8:
ret
.balign 4
.global dn_gateway_9
dn_gateway_9:
ret
.balign 4
.global dn_gateway_10
dn_gateway_10:
ret
"#
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
    for i in 0..hooks::COUNT {
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
    for i in 0..hooks::COUNT {
        reset(None, Some(i));
        assert!(hooks::Installed::install().is_err());
        originals_restored();
    }
}
#[test]
fn successful_install_enables_only_after_all_hooks() {
    let _guard = SERIAL.lock().unwrap();
    reset(None, None);
    let installed = hooks::Installed::install().unwrap();
    assert_eq!(installed.count(), hooks::COUNT);
    assert!(hooks::ACTIVE.load(core::sync::atomic::Ordering::Acquire));
    let n = NATIVE.lock().unwrap();
    assert_eq!((n.calls, n.patches), (hooks::COUNT, hooks::COUNT));
    drop(n);
    drop(installed);
    hooks::ACTIVE.store(false, core::sync::atomic::Ordering::Relaxed);
    originals_restored();
}

#[test]
fn preflight_rejects_bad_addresses_overlaps_and_reserved_sites() {
    let base = mem::text_addr();
    let targets = core::array::from_fn(|i| base + signatures::SIGNATURES[i].0);
    let detours = core::array::from_fn(|i| 0x40000000 + i as u32 * 4);
    assert!(hooks::validate_recipe(base, &targets, &detours));
    for bad in [
        0,
        1,
        u32::MAX - 3,
        u32::MAX - 7,
        base + 0x3da8fc,
        targets[1],
        targets[1] + 4,
    ] {
        let mut modified = targets;
        modified[0] = bad;
        assert!(!hooks::validate_recipe(base, &modified, &detours));
    }
    for bad in [0, 1, targets[0], targets[0] + 4] {
        let mut modified = detours;
        modified[0] = bad;
        assert!(!hooks::validate_recipe(base, &targets, &modified));
    }
}
#[test]
fn signature_and_program_rejection_never_call_forge() {
    let _guard = SERIAL.lock().unwrap();
    for window in 0..signatures::SIGNATURES.len() {
        reset(None, None);
        let address = mem::text_addr() + signatures::SIGNATURES[window].0;
        write(&mut NATIVE.lock().unwrap(), address, &[0xff]);
        assert!(hooks::Installed::install().is_err());
        let n = NATIVE.lock().unwrap();
        assert_eq!((n.calls, n.patches), (0, 0));
    }
    reset(None, None);
    NATIVE.lock().unwrap().wrong_program = true;
    assert!(hooks::Installed::install().is_err());
    assert_eq!(NATIVE.lock().unwrap().calls, 0);
}
#[test]
fn partial_publication_and_invalid_continuations_restore_all_sites() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..hooks::COUNT {
        for mode in 0..4 {
            reset(None, None);
            {
                let mut n = NATIVE.lock().unwrap();
                match mode {
                    0 => n.partial = Some(i),
                    1 => n.corrupt_trampoline = Some(i),
                    2 => n.mismatch_return = Some(i),
                    _ => n.deny_execute = Some(0x30000000 + 16 * i as u32),
                }
            }
            assert!(hooks::Installed::install().is_err());
            originals_restored();
        }
    }
}
#[test]
fn entry_hooks_chain_onto_a_foreign_forge_hook_and_interior_taps_refuse_it() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..hooks::COUNT {
        reset(None, None);
        let address = mem::text_addr() + signatures::SIGNATURES[i].0;
        let foreign = [0xe51ff004u32.to_le_bytes(), 0x0320_1000u32.to_le_bytes()].concat();
        write(&mut NATIVE.lock().unwrap(), address, &foreign);
        let result = hooks::Installed::install();
        if i == 5 || i == 10 {
            assert!(result.is_err(), "hook {i}");
            assert_eq!(NATIVE.lock().unwrap().calls, 0);
        } else {
            let installed = result.unwrap_or_else(|e| panic!("hook {i}: {e}"));
            let continuation = hooks::ORIGINALS[i].load(core::sync::atomic::Ordering::Acquire);
            assert_eq!(read(&NATIVE.lock().unwrap(), continuation, 8), foreign, "hook {i}");
            core::mem::forget(installed);
        }
    }
}
#[test]
fn foreign_changes_survive_rollback_with_continuations_retained() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..hooks::COUNT {
        reset(None, None);
        let address = mem::text_addr() + signatures::SIGNATURES[i].0;
        NATIVE.lock().unwrap().foreign = Some((i, address));
        let result = hooks::Installed::install();
        assert!(matches!(
            result,
            Err("hook rollback failed; restart required")
        ));
        let n = NATIVE.lock().unwrap();
        assert_eq!(read(&n, address, 4), 0xe1a00000u32.to_le_bytes());
        assert_eq!(n.patches, 0);
        assert!(!hooks::ACTIVE.load(core::sync::atomic::Ordering::Acquire));
        assert_eq!(
            hooks::ORIGINALS[i].load(core::sync::atomic::Ordering::Acquire),
            0x30000000 + 16 * i as u32
        );
    }
}
#[test]
fn failed_restore_or_rollback_cache_sync_requires_restart() {
    let _guard = SERIAL.lock().unwrap();
    for i in 0..hooks::COUNT {
        reset(None, Some(i));
        let address = mem::text_addr() + signatures::SIGNATURES[i].0;
        NATIVE.lock().unwrap().restore_fail = Some(address);
        let failure = hooks::Installed::install().err();
        assert_eq!(
            failure,
            Some("hook rollback failed; restart required"),
            "site {i}, detours {:?}",
            hooks::detours()
        );
        assert_eq!(
            hooks::ORIGINALS[i].load(core::sync::atomic::Ordering::Acquire),
            0x30000000 + 16 * i as u32
        );
        assert!(!hooks::ACTIVE.load(core::sync::atomic::Ordering::Acquire));
        assert_eq!(NATIVE.lock().unwrap().patches, 0);
    }
    reset(Some((0, false)), Some(0));
    assert!(matches!(
        hooks::Installed::install(),
        Err("hook rollback failed; restart required")
    ));
    originals_restored();
}
