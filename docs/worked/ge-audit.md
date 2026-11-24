# Damage Numbers: GE preservation audit

Worked example from `mods/damage-numbers/`. Game facts it rests on are in `mhgu-understanding`.

## GE preservation audit

Checked for capture hooks 0 to 7 and extended to hooks 8 to 10. GE restoration is unnecessary at these sites. The shared RESTORE macro uses `msr APSR_nzcvq, r2`, which preserves NZCVQ and leaves GE untouched. Ryujinx-Nextendo at `05577ab543f4322cb7534c1f5caca0e0079791bf` handles MSR mask 8 (NZCVQ) and throws `APSR_g` for mask 4 (GE), so this avoids that translation path.

- None of the hooked function bodies, plus the poison tick's caller 0xc8ddc (2,056 bytes), contains `SEL`, `MRS`, `MSR` or a GE-setting parallel add or subtract.
- Melee tail hook 5 has already passed the selection when it runs. `vselge` uses `N == V`, not GE, and the continuation stores an integer and returns:

  ```asm
  0x16dd6c: vcmpe.f32 s0, s2
  0x16dd70: vmrs APSR_nzcv, fpscr
  0x16dd74: movwlt r0, #1
  0x16dd78: vselge.f32 s0, s0, s2
  0x16dd7c: strb r0, [r4]                 // hook begins here
  0x16dd80: vpop {d8-d12}
  0x16dd84: pop {r4, r5, r11, pc}
  ```

  AAPCS32 leaves NZCVQ and GE unspecified across public interfaces, but that alone does not justify dropping preservation at an interior tap. This continuation is the evidence.
- A raw scan of the 20,913,644-byte main text found no `SEL` and no GE access in the 47,986 indexed functions. Eight MRS/MSR-shaped words outside those ranges stay candidates, so this supports the result and does not prove every byte is GE-free.
- Imported helpers (`memcpy`, SDK TLS and clock, the query-memory SVC) were not proved GE-preserving. GE is dead at the audited continuations, so correctness does not depend on them.
- Scratch gateway checks passed 9,031 checks across 1,769 fixture calls, and all 16 GE patterns across all eight entries. These are Unicorn fixtures, not gameplay. r17 was user-confirmed on Eden and Ryujinx with this restore.

Recheck any new hook, especially a tap between GE-setting arithmetic and a `SEL`, or code that reads GE through `MRS`. A generic gateway should keep GE preservation unless liveness is ruled out.
