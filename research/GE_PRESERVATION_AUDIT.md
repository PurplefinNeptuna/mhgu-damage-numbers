# GE preservation audit

Checked 2026-10-06 for the eight current MHGU 1.4.0 capture hooks and the
released r16 plugin. GE restoration is unnecessary for these hook locations.
The gateway's general transparent-state contract originally included GE, but
the captured game computations do not consume it.

The bounded compatibility change is `msr APSR_nzcvqg, r2` to
`msr APSR_nzcvq, r2` in the shared RESTORE macro. This preserves NZCVQ and
leaves GE untouched by that instruction. It does not zero GE. This audit did
not change the production source, release package or installed plugin.

## Binary identity

- Copied original main NSO: `work/damage-numbers/inputs/main`, SHA-256
  `b3935415499549adaf4c0e89822831daa175cfd456a7990791c79be3997645ef`.
- Main build ID: `FB08F1D20FD1204FE7070672777F869B`.
- Verified decompressed main image: SHA-256
  `24c208c395c65727b21b5fa91af8381396e70cf421b0fcfee0345d8743298110`.
- Released NRO and current Ryujinx-installed NRO both have SHA-256
  `5f59506f88d5156ed8db7cffd5747651717b3bab7ecdfd398800a0402fe4f33e`.
  Installed path:
  `<recorded Ryujinx r16 package>/romfs/nativeNX/plugins/damage_numbers_capture.nro`.
- `work/damage-numbers/open-assets/plugin.elf` SHA-256
  `589f0b39e9e80f5321cbd1e2725a62ba6bc11b9f50a7d780707065de8ed245c0`.
  Converting this ELF with the existing `elf_to_nro` reproduces the released
  NRO hash, tying the callback disassembly to the installed binary.

Installation presence does not establish activation or a gameplay result.

## Original game instructions

Function boundaries came from the saved Ghidra functions index, with image
base `0x009c9000` subtracted. Instruction bytes came from the verified original
NSO, avoiding the known byte discrepancy in the Ghidra database.

| Hook | Main-relative offset | Location | Enclosing function bytes |
| --- | --- | --- | --- |
| 0 | `0x170968` | Collision function entry, around hook | 15,868 |
| 1 | `0x9d6f8` | Damage function entry, around hook | 432 |
| 2 | `0x2a87a8` | Feedback function entry, around hook | 680 |
| 3 | `0xb8909c` | Viewport function entry, around hook | 2,128 |
| 4 | `0x885844` | Camera function entry, before-only hook | 4,424 |
| 5 | `0x16dd7c` | Melee hitzone function tail, before-only hook | 284 |
| 6 | `0x16dc6c` | Melee hitzone function entry, around hook | 284 |
| 7 | `0x178660` | Shot hitzone function entry, around hook | 500 |

None of these complete function bodies contains a `SEL`, `MRS`, `MSR`, or
GE-setting parallel integer add/subtract instruction. Ordinary comparisons,
conditional branches and floating-point calculations do not consume GE[3:0].

The sole interior tap is particularly narrow. Immediately before it, the game
compares floating-point values and selects the result:

```asm
0x16dd6c: vcmpe.f32 s0, s2
0x16dd70: vmrs APSR_nzcv, fpscr
0x16dd74: movwlt r0, #1
0x16dd78: vselge.f32 s0, s0, s2
0x16dd7c: strb r0, [r4]                 // hook begins here
0x16dd80: vpop {d8-d12}
0x16dd84: pop {r4, r5, r11, pc}
```

`vselge` uses the signed condition `N == V`, not the four GE bits. That selection
has already completed when the tap runs. The continuation stores the integer
result and returns; it does not read GE or perform another flag-dependent
selection. Preserving the captured registers and floating-point state still
matters. Instruction semantics:
[Arm instruction guide, VSEL](https://documentation-service.arm.com/static/6245c734b059dc5ff9a8bdab),
[Arm VFP condition codes](https://developer.arm.com/community/arm-community-blogs/b/architectures-and-processors-blog/posts/condition-codes-4-floating-point-comparisons-using-vfp).

At public function interfaces, the Arm calling convention defines NZCVQ and
GE as unspecified on entry and return. See
[AAPCS32, section 6.1.1](https://github.com/ARM-software/abi-aa/blob/main/aapcs32/aapcs32.rst#611core-registers).
This ABI rule alone would not justify removing preservation at an arbitrary
interior tap; the exact continuation above supplies the additional evidence.

A broader raw-encoding scan of the 20,913,644-byte main text found no ARM or
Thumb `SEL` candidate and no GE-access candidate inside the 47,986 indexed
functions. Eight ARM MRS/MSR-shaped words outside those indexed ranges were
retained as candidates, not asserted to be executable instructions. This
scan is supporting evidence, not a proof that every module or unindexed byte
is GE-free.

## Capture helper instructions

Disassembly of the matching r16 ELF found no GE arithmetic or GE consumers in
the compiled C++ code. Its nine `MRS`/`MSR` pairs belong to the eight assembly
entries and the shared return gateway. No C++ callback reads `RawFrame::cpsr`.

Capture helpers also call imported `memcpy`, SDK TLS/clock functions and a
private query-memory SVC wrapper. The imported implementations were not
proved GE-preserving in this audit. They may clobber GE under the calling
convention; GE is dead at the audited game continuations, so correctness does
not depend on those callees preserving it.

The comment marking gateways 6 and 7 as fixture-only is stale:
`capture_install()` currently installs all eight signature/detour entries.
Both were included in this audit.

## Scratch execution checks

The existing compiled gateway fixture was verified against its manifest and
source hashes. Its original binary SHA-256 is
`ffe6fb1e27c8e8483c622667fc96e12a9f6eef38c3ba70e860bccfccbddb8928`.

A scratch copy changed only nine instruction words: `e12cf002` (NZCVQ+GE)
to `e128f002` (NZCVQ). Scratch SHA-256:
`03a310467d9f72f897caf0d28052f22d506b35ce6193d1fe1bebce694557b36f`.

- The unchanged gateway passed 9,031 existing checks across 1,769 fixture calls.
- The scratch gateway passed the same 9,031 checks and 1,769 calls with GE
  excluded from the deliberately stronger full-state comparison. Integer
  registers, all d0-d31, NZCVQ, FPSCR, LR, SP, stack arguments, nesting/capacity
  behavior and copied native damage/hitzone results remained checked.
- All 16 GE patterns across all eight entries passed 128 additional full-state
  comparisons when the helper stub preserved GE.
- Another 128 comparisons deliberately changed GE in the helper. The scratch
  gateway correctly left that changed GE in place while all other checked
  state matched. This confirms the reduced contract explicitly.

Evidence is retained under `work/damage-numbers/ge-audit/`, including original
hook-function disassembly, encoding candidates, scratch gateway binary and
`gateway-probe.json`. Tests used pinned Unicorn 2.1.4. Capture callbacks and
native service dependencies are fixture stubs; these are not live Ryujinx or
Eden gameplay tests.

## R17 implementation — 2026-10-06

The audited change is now applied in the production `gateway.S` restore macro.
The original r16/scratch evidence above remains a historical record.
[R17 compatibility notes](RUNTIME.md) document the reduced
contract, production binary comparison, fresh fixtures and user-confirmed
Eden/Ryujinx gameplay. The stale entries-6/7 comment is corrected; all eight entries
remain production hooks. No hook location or captured frame layout changes.

The original Ryujinx-Nextendo translator at reference commit
`05577ab543f4322cb7534c1f5caca0e0079791bf` handles MSR mask 8 (NZCVQ) and throws
`APSR_g` for mask 4 (GE). The proposed instruction therefore avoids this
specific unsupported translation path. Its MRS also omits GE, but the capture
helpers do not consume the saved GE bits.

## Limits

This result applies to the recorded game build, r16 helper binary and current
eight hook sites. Recheck new hooks, especially a tap between GE-setting
parallel arithmetic and a `SEL`, or code that reads GE through `MRS`.
A generic transparent gateway should retain GE preservation when such
liveness has not been ruled out. Gameplay compatibility of a changed NRO
still requires a user-run emulator test.

R18 revalidation (2026-10-07): gateway source is byte-identical to the audited r17 assembly. All nine emitted NZCVQ restores and all original/compiled gateway state fixtures passed. No hook site or continuation changed; r18 carries this liveness conclusion. This is static/compiled evidence without a fresh gameplay run.
