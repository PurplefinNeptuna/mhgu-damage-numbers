//! Forge owns trampoline construction. Patch owns reversible target bytes; hooks are installed once at boot.
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
use forge::{patch::Patch, sys::hook::Hook};
// ACTIVE is published only after all eight continuations are ready. Each
// ORIGINALS entry holds Forge's trampoline address for the matching gateway.
pub static ACTIVE: AtomicBool = AtomicBool::new(false);
pub static ORIGINALS: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<Hook>() == 76);
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
// Holding the patches keeps their modified state alive. Holding the native
// Hook values records the trampolines used for every later game call.
// The Consumer owns this structure for the plugin's process lifetime.
pub struct Installed {
    patches: Vec<Patch>,
    _hooks: Vec<Hook>,
}
impl Installed {
    pub fn install() -> Result<Self, &'static str> {
        let base = forge::mem::text_addr();
        platform::BASE.store(base, Ordering::Relaxed);
        if base == 0 || base > 0xfe000000 || !platform::correct_program() {
            return Err("unsupported program");
        }
        let memory = NativeMemory;
        // Compare all 64-byte windows before patching anything. The final three
        // windows verify supporting game code as well as the eight hook locations.
        for &(offset, bytes) in SIGNATURES {
            let mut actual = [0; 64];
            if !memory.copy(base + offset, &mut actual) || actual != bytes {
                return Err("MHGU 1.4.0 signature mismatch");
            }
        }
        let detours = detours();
        let mut patches = Vec::with_capacity(8);
        let mut hooks = Vec::with_capacity(8);
        // Prepare every rollback object before Forge writes a branch. A disabled
        // Patch records the original bytes without changing game code yet.
        for i in 0..8 {
            let address = base + SIGNATURES[i].0;
            for at in [0, 4] {
                let op = u32::from_le_bytes(SIGNATURES[i].1[at..at + 4].try_into().unwrap());
                if !relocatable(op) {
                    return Err("unsupported displaced instruction");
                }
            }
            let mut branch = [0; 8];
            // ARM LDR pc, [pc, #-4] loads the next word as an absolute destination.
            // At instruction address A, ARM reads PC as A+8, hence the literal at A+4.
            // Together this instruction and address occupy the displaced eight bytes.
            branch[..4].copy_from_slice(&0xe51ff004u32.to_le_bytes());
            branch[4..].copy_from_slice(&detours[i].to_le_bytes());
            patches.push(Patch::new(address, &branch, false));
        }
        for i in 0..8 {
            let address = base + SIGNATURES[i].0;
            let mut original = core::ptr::null();
            // SAFETY: boot is quiescent, target signatures are verified, and the detour saves the complete register frame.
            let hook = unsafe {
                forge::sys::hook::forge_hook_create(
                    address as *const c_void,
                    detours[i] as *const c_void,
                    &mut original,
                )
            };
            hooks.push(hook);
            // Forge may publish the branch before a later JIT transition fails. Track that attempt for rollback too.
            if memory.word(address) == 0xe51ff004 && memory.word(address + 4) == detours[i] {
                // The branch is already present. Enabling marks the Patch as responsible
                // for restoring its saved bytes if a subsequent installation step fails.
                patches[i].enable();
            }
            if original.is_null() || !patches[i].enabled() || !platform::sync_code(address, 8) {
                // Dropping only enabled Patch objects restores their pre-install bytes. Trampolines are never freed here.
                drop(patches);
                let mut restored = true;
                for &(offset, bytes) in &SIGNATURES[..8] {
                    restored &= platform::sync_code(base + offset, 8);
                    let mut actual = [0; 8];
                    restored &= memory.copy(base + offset, &mut actual) && actual == bytes[..8];
                }
                return Err(if restored {
                    "Forge hook installation failed"
                } else {
                    "hook rollback failed; restart required"
                });
            }
            // Release publishes the initialized continuation to gateway readers using
            // Acquire. The game must never jump to an uninitialized trampoline.
            ORIGINALS[i].store(original as u32, Ordering::Release);
        }
        // Only now can hooks produce capture scopes. Earlier calls pass through
        // the published continuations without collecting events.
        ACTIVE.store(true, Ordering::Release);
        Ok(Self {
            patches,
            _hooks: hooks,
        })
    }
    pub fn count(&self) -> usize {
        self.patches.len()
    }
}
