//! Forge owns pooled trampolines. Disabled Patch snapshots support checked rollback at the boot boundary.
use crate::{
    platform::{self, NativeMemory},
    signatures::SIGNATURES,
};
use alloc::vec::Vec;
use core::{
    ffi::c_void,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
};
use damage_numbers_core::capture::Memory;
use forge::patch::Patch;
#[cfg(target_pointer_width = "32")]
use forge::sys::hook::Hook;
// ACTIVE is published only after all eight continuations are ready. Each
// ORIGINALS entry holds Forge's trampoline address for the matching gateway.
pub static ACTIVE: AtomicBool = AtomicBool::new(false);
pub static ORIGINALS: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<Hook>() == 24);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::offset_of!(Hook, context) == 12);
unsafe extern "C" {
    fn dn_gateway_0();
    fn dn_gateway_1();
    fn dn_gateway_2();
    fn dn_gateway_3();
    fn dn_gateway_4();
    fn dn_gateway_5();
    fn dn_gateway_6();
    fn dn_gateway_7();
}
// Order is the wire contract: indices match SIGNATURES[0..8] and the kind
// argument passed by gateway.S. Reordering only one list changes hook meaning.
pub fn detours() -> [u32; 8] {
    [
        dn_gateway_0 as *const () as u32,
        dn_gateway_1 as *const () as u32,
        dn_gateway_2 as *const () as u32,
        dn_gateway_3 as *const () as u32,
        dn_gateway_4 as *const () as u32,
        dn_gateway_5 as *const () as u32,
        dn_gateway_6 as *const () as u32,
        dn_gateway_7 as *const () as u32,
    ]
}
// Forge copies the first two instructions into a trampoline. Accept only the
// audited encodings that remain valid away from their original address.
// The masks below recognize specific ARM operations; this is not a general
// instruction decoder. PC-relative instructions need separate relocation.
fn relocatable(op: u32) -> bool {
    if [
        0xed2d8b02, 0xed2d8b0a, 0xe28db018, 0xe28db01c, 0xe5c40000, 0xecbd8b0a,
    ]
    .contains(&op)
    {
        return true;
    }
    // The top four bits are ARM's condition code; 14 means unconditional AL.
    // Conditional instructions outside the explicit whitelist are rejected.
    if op >> 28 != 14 {
        return false;
    }
    // This is PUSH's register-list encoding. Exclude PC and SP and require at
    // least one register, so copied stack saving has no location-dependent PC.
    if op & 0xffff0000 == 0xe92d0000 {
        return op & 0x8000 == 0 && op & 0x2000 == 0 && op & 0xffff != 0;
    }
    // Allow SUB sp, sp, #immediate: stack allocation is position independent.
    if op & 0xfffff000 == 0xe24dd000 {
        return true;
    }
    // Allow register MOV only when neither operand is PC (register number 15).
    if op & 0xffff0ff0 == 0xe1a00000 {
        return ((op >> 12) & 15) != 15 && (op & 15) != 15;
    }
    // Allow MOVW's literal-immediate load when its destination is not PC.
    if op & 0xfff00000 == 0xe3000000 {
        return ((op >> 12) & 15) != 15;
    }
    false
}
// Known game offsets used by Forge, Better Input and maintained plugins. Each
// redirect occupies eight bytes; signatures also reject unknown modifications.
const RESERVED: [u32; 14] = [
    0x3de064, 0x3af5b4, 0x388ee8, 0x5e8130, 0x3850b0, 0x7a3480, 0x3da8fc, 0xb8692c, 0x887810,
    0x88784c, 0x27e774, 0x27ca28, 0x678894, 0x285e2c,
];
fn overlaps(a: u32, bytes: u32, b: u32, other_bytes: u32) -> bool {
    (a as u64) < b as u64 + other_bytes as u64 && (b as u64) < a as u64 + bytes as u64
}
// Validate the complete eight-byte ARM redirects before allocating snapshots or
// calling Forge. Wider arithmetic prevents a wrapped end from passing a check.
pub(crate) fn validate_recipe(base: u32, targets: &[u32; 8], detours: &[u32; 8]) -> bool {
    for (i, (&target, &detour)) in targets.iter().zip(detours).enumerate() {
        if target == 0
            || detour == 0
            || (target | detour) & 3 != 0
            || target as u64 + 8 >= 0x1_0000_0000
            || detour as u64 + 4 > 0x1_0000_0000
            || targets.iter().any(|&other| overlaps(other, 8, detour, 4))
            || targets[..i]
                .iter()
                .any(|&other| overlaps(target, 8, other, 8))
            || RESERVED.iter().any(|&offset| {
                base.checked_add(offset)
                    .is_none_or(|address| overlaps(target, 8, address, 8))
            })
        {
            return false;
        }
    }
    true
}
// Patches stay disabled: Forge writes the redirects, and this owner must check
// the current words before restoring them. Forge's process-lifetime pool owns
// every trampoline; dropping a Hook value neither releases nor uninstalls it.
pub struct Installed {
    patches: Vec<Patch>,
}
impl Installed {
    // Roll back all sites, including a partially published pair. A word may be
    // either our branch word or its saved original, but foreign words are never
    // overwritten. The boot boundary excludes concurrent game execution here.
    fn rollback(&mut self) -> bool {
        ACTIVE.store(false, Ordering::Release);
        let memory = NativeMemory;
        let mut restored = true;
        for patch in self.patches.iter_mut().rev() {
            let mut current = [0; 8];
            let original: [u8; 8] = patch.original_bytes().try_into().unwrap();
            let replacement: [u8; 8] = patch.patch_bytes().try_into().unwrap();
            if !memory.copy(patch.addr(), &mut current) {
                restored = false;
                continue;
            }
            if current != original {
                let owned = (0..8).step_by(4).all(|at| {
                    current[at..at + 4] == original[at..at + 4]
                        || current[at..at + 4] == replacement[at..at + 4]
                });
                if !owned {
                    restored = false;
                    continue;
                }
                // Patch exposes no adopt/conditional-restore operation. Once
                // ownership is verified, enable records responsibility and
                // disable restores the snapshot. Both calls run before play.
                patch.enable();
                patch.disable();
            }
            let synchronized = platform::sync_code(patch.addr(), 8);
            restored &=
                synchronized && memory.copy(patch.addr(), &mut current) && current == original;
        }
        // All snapshots are disabled, so their Drop cannot write foreign bytes.
        self.patches.clear();
        restored
    }
    pub fn install() -> Result<Self, &'static str> {
        let base = forge::mem::text_addr();
        platform::BASE.store(base, Ordering::Relaxed);
        if base == 0 || base > 0xfe000000 || base & 3 != 0 || !platform::correct_program() {
            return Err("unsupported program");
        }
        let memory = NativeMemory;
        // Compare all eleven windows before touching any target. The last three
        // check supporting game code and the expected MHGU executable bytes.
        for &(offset, bytes) in SIGNATURES {
            let Some(address) = base
                .checked_add(offset)
                .filter(|a| a.checked_add(63).is_some())
            else {
                return Err("invalid signature address");
            };
            let mut actual = [0; 64];
            if !memory.copy(address, &mut actual) || actual != bytes {
                return Err("MHGU 1.4.0 signature mismatch");
            }
        }
        let targets = core::array::from_fn(|i| base + SIGNATURES[i].0);
        let detours = detours();
        if !validate_recipe(base, &targets, &detours) {
            return Err("invalid or overlapping hook recipe");
        }
        for &(_, bytes) in &SIGNATURES[..8] {
            for at in [0, 4] {
                if !relocatable(u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())) {
                    return Err("unsupported displaced instruction");
                }
            }
        }
        let mut installed = Self {
            patches: Vec::with_capacity(8),
        };
        // Capture all originals while the recipe still matches, before Forge
        // publishes anything. ARM LDR pc,[pc,#-4] plus its literal uses 8 bytes.
        for i in 0..8 {
            let mut branch = [0; 8];
            branch[..4].copy_from_slice(&0xe51ff004u32.to_le_bytes());
            branch[4..].copy_from_slice(&detours[i].to_le_bytes());
            installed
                .patches
                .push(Patch::new(targets[i], &branch, false));
        }
        for i in 0..8 {
            let address = targets[i];
            let mut original = core::ptr::null();
            // SAFETY: Forge invokes initialization before original_sApp_run;
            // exact ARM bytes/relocation and the complete gateway frame were checked.
            let hook = unsafe {
                forge::sys::hook::forge_hook_create(
                    address as *const c_void,
                    detours[i] as *const c_void,
                    &mut original,
                )
            };
            let continuation = original as u32;
            let mut trampoline = [0; 16];
            // The pooled ARM trampoline contains the two displaced words,
            // LDR pc,[pc,#-4], then target+8. Reject inaccessible or corrupt code.
            let valid = continuation != 0
                && continuation & 3 == 0
                && platform::executable(continuation, 16)
                && !targets
                    .iter()
                    .any(|&target| overlaps(continuation, 16, target, 8))
                && memory.copy(continuation, &mut trampoline)
                && trampoline[..8] == SIGNATURES[i].1[..8]
                && trampoline[8..12] == 0xe51ff004u32.to_le_bytes()
                && trampoline[12..] == (address + 8).to_le_bytes();
            if valid {
                // Retain even if a subsequent check or rollback fails. A leftover
                // reachable gateway still needs a valid pass-through continuation.
                ORIGINALS[i].store(continuation, Ordering::Release);
            }
            let mut actual = [0; 8];
            let branch_valid =
                memory.copy(address, &mut actual) && actual == installed.patches[i].patch_bytes();
            if !valid
                || hook.original != original
                || hook.target as u32 != address
                || hook.detour as u32 != detours[i]
                || !branch_valid
                || !platform::sync_code(address, 8)
            {
                return Err(if installed.rollback() {
                    "Forge hook installation failed"
                } else {
                    "hook rollback failed; restart required"
                });
            }
        }
        ACTIVE.store(true, Ordering::Release);
        Ok(installed)
    }
    pub fn count(&self) -> usize {
        self.patches.len()
    }
}
impl Drop for Installed {
    fn drop(&mut self) {
        // Production retains this owner until process exit. Tests may drop it
        // at the same quiescent boundary; shared Forge memory is never released.
        if !self.patches.is_empty() {
            self.rollback();
        }
    }
}
