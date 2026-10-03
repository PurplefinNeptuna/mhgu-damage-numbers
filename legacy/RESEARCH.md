# Damage-number binding research

All addresses below are offsets relative to `main`, after subtracting the
Ghidra image base `0x009c9000`. The original NSO is authoritative; the imported
database has one altered text byte and conflicting types. Selected exports
below report zero instruction-byte differences from the original.

Original MHGU 1.4.0 `main` SHA-256:
`b3935415499549adaf4c0e89822831daa175cfd456a7990791c79be3997645ef`.
Uncompressed memory-image SHA-256:
`24c208c395c65727b21b5fa91af8381396e70cf421b0fcfee0345d8743298110`.
Preserved source identity and segment hashes are in
`work/damage-numbers/inputs/provenance.json`.

## Damage and hit metadata

| Native offset | Evidence and use |
| --- | --- |
| `0x168960` | Attack initialization; player arrays use a `0xb8` attack stride. The imported 182-byte layout is unsuitable. |
| `0x16915c` | Stores affinity and marks it enabled; this is not the critical outcome. |
| `0x16c0c0` | Monster collision traversal supplies the actual collision vector and hurtbox record. |
| `0x170968` | Routes an impact to source-specific monster damage processing. Its imported body combines shared branches; verify ABI from instructions. |
| `0x9d6f8` | Applies defense/sleep/rage normalization and returns the signed contribution added to the per-part accumulator. Original-byte fixture checks passed for ordinary parts. |
| `0x97ddc` | Consumes per-part contributions. The sum is subtracted from current HP (`monster parameter +0x370`) before clamps and phase-specific HP floors. |
| `0x17885c` | Resolves actual affinity outcomes, including Weakness Exploit, Critical Boost and Bitter Affinity. Attack byte `+0x54` is 0/1/2 for none/negative/positive native outcomes. |
| `0xbab60` | Returns a monster's physical/elemental hitzone record; ordinary indices are below 8. |
| `0x16dc6c` | Selects weighted cut/blunt damage and applies monster-state HZV adjustments. Uses VFP float arguments/results. |
| `0x1748a8`, `0x178660` | Melee sharpness/bounce-effect and shot/fixed-damage paths. Effect codes alone do not prove a hunter bounce. |

Attack fields checked in native accesses include primary owner pointer `+0xc`
and kind `+0x10`, secondary owner kind `+0x6e` and pointer `+0x70`, impact vector
`+0x20..+0x28`, damage type `+0x3a`, effect code `+6`, critical outcome `+0x54`
and hurtbox pointer `+0xac`. Hurtbox byte `+6` selects the HZV record; byte `+8`
selects the accumulation part. Damage type `0x1d` takes a healing branch and
must be rejected explicitly, in addition to rejecting nonpositive contributions.

`0x9d6f8` adds its signed return to `monster parameter +0x488+2*part`.
The consumer reads each eligible part into the HP sum before applying distinct
part-break modifiers. The HP-subtraction store is at `0x98b04`; nearby clamps
would make an HP-difference-only display lose lethal-hit values and merge hits.
The callback at enemy vtable `+0x2f8` is an on-hit notification whose base
implementation is a no-op. `0x9dd40` tracks attacker threat; neither is an HP
application hook.

Ordinary collision paths combine accompanying elemental damage with physical
damage before native integer conversion and normalization. Capture must use
that completed contribution and preserve the call's impact identity. The native
normalizer's extended-part path calls vtable `+0x310` and returns zero; it needs
separate investigation before claiming coverage of every monster part.

## Ownership

The native local getter `0x276278` obtains the session index through the
singleton cell `main+0x188ca48` and resolves the player from the table at
`main+0x1885b58`, offset `+0x1c+4*index`. Valid ordinary player indices are 0–3.
`ownership.h` agrees with original native execution across all four local slots.

The palico master getter `0x285c6c`, for its palico branch, reads the current
work pointer at `object+0x3ac4`, then its master byte at `work+1`, and calls
`0x274ce4` to look up the player. The native constructor supplies inline work;
the saved DTI reports `obw->mMasterId` at `+0x3ad1`. Use the current pointer,
which can differ from inline storage. Byte `work+0x34d` is a separate serialized
field and is not the master ID.

The ownership fixture redirects work away from inline storage and compares
local/remote palico classification with the native master getter. Invalid
indices, missing work and absent master players are hidden. Constructed attack
snapshots cover direct and secondary-owned projectiles; live source attribution
still needs solo/Prowler and multiplayer trials.

## Camera

Native look-at `0x7c5e4c`, perspective `0x7c617c` and matrix product `0x29d48`
use row-vector matrices: `world * view * projection`. The perspective matrix
uses negative camera-space Z and a zero-to-one depth interval. `camera.h`
transposes the native view/projection product for the presentation helper.
The same C++ helper agrees with copied native matrices across rotated and
translated cameras, FOVs and aspect ratios. The fixture's tangent import is an
explicit float32 stub; it does not select a live camera.

The viewport render path `0xb8909c` reads a camera pointer at `viewport+0x24`,
rectangle edges at `+0x38..+0x44`, view/projection matrices at `+0xb0/+0xf0`
and alternate matrices at `+0x130/+0x170`. It can interpolate matrices and
adjust projection for viewport/render scale. The eventual adapter must capture
the matrices actually sent to rendering, identify the gameplay viewport and
map its rectangle to Forge's display coordinates. Reading only camera FOV or
an unblended matrix is insufficient.

Final upload routine `0x885844` receives distance in `s0`, render context in
`r0`, final view/projection pointers in `r1/r2`, and the integer rectangle
pointer in `r3`. Its first eight original bytes are a register push and frame
pointer adjustment. The capture candidate copies these matrices only inside
the viewport-render scope when the camera has the saved `uQuestCamera` vtable
`main+0x1730d0c`. This is a camera candidate, pending live selection and render
scale checks.

R4 uses this binding for an opt-in projection preview. Forge's inspected NVN
backend obtains `ImGuiIO::DisplaySize` from its swapchain textures and sets
framebuffer scale to one. The preview accepts only zero-origin full-screen
native rectangles with a matching aspect, then maps normalized clip coordinates
into that display size. Offset/split viewports remain unsupported. Camera data
older than 200 ms is hidden, and copied camera/viewport identity changes clear
the transient display. No camera pointer is dereferenced by the renderer.

The preserved SDK C wrapper `nnosGetSystemTick` at SDK `0x9e640` converts the
C++ Tick aggregate to a 64-bit r0/r1 return. The original frequency implementation
at `0x9d840` returns 19,200,000. Original-wrapper fixtures check values beyond
32 bits; the hardware-counter query is explicit fixture data. R4 records ticks
with damage/camera payloads and subtracts them before converting to float, so
mailbox delay does not extend a number's lifetime. Record size is now 248 bytes,
with copied player identity at `+236` and the tick at `+240`; the earlier fields
keep their offsets.

The renderer uses the pinned cimgui font/draw API through Forge's free-render
callback, which runs even when the Forge window is closed. The inspected ImGui
font cache rounds requested sizes, and Forge's NVN backend advertises dynamic
texture support. Outlined digits use eight black offset passes plus the color
pass; critical lines have an outline and reserved vertical lane space. Actual
font rasterization, GPU output and impact alignment subsequently passed the
user's r4 melee visual check at a logged 1920x1080 viewport/display.

The r4 log has 51 finalized hits: 18 local hunter hits and 33 local-palico hits.
Of the palico hits, 15 have tags `0x030403`, 15 have `0x030402` and three have
`0x030802`. The last two families were rejected by r4's direct-primary filter;
their total equals the panel's 18 rejected hits. The user reports missing
boomerang and thrown-barrel popups while melee works. Tags pack primary kind
`+0x10`, source subtype `+0x6d` and secondary owner kind `+0x6e`, so the rejected
families have primary kind 2 and secondary palico kind 3. Ownership was already
resolved through the secondary pointer and current palico master work.

R5 admits just those two observed families with `Owner::LocalPalico`, retaining
the producer ownership and healing gates. This fixes the known display exclusion
without generalizing to all projectile/object sources. Boomerang-versus-barrel
subtype labels are inferred from the observed actions and HZV/effect patterns;
the next manual check covers both attacks. Unknown physical HZV remains white.
Saved DTI `cOtWeaponBase` separately exposes boomerang attack, element and affinity
properties; no equipment or damage data was edited for this fix.

## Hitzone and bounce capture

The melee routine's tap at `0x16dd7c`, immediately before the selected-type
store, exposes cut/blunt selection in `r0` and adjusted cut/blunt HZVs in
`s20/s22`. Weighted cut/blunt ties select cut. The native fixture verifies the
tap and C++ extraction through 1,440 combinations of hitzones, attack weights,
monster state, player adjustment and part modifiers. Using its returned damage
factor times 100 would lose the unweighted HZV for mixed attacks.

The candidate scopes the tap within `0x16dc6c` and the matching monster impact.
The shot helper `0x178660` supplies its physical-factor output pointer as the
second stack argument; only attacks with native flag `+0x30 & 1` have a shot
HZV. The compiled capture fixture covers 44/45/80 and the no-physical-HZV case.

Getter `0x2a87a8` resolves weapon feedback from attacks at `player+0x3020` and
`player+0x30d8`. Each has active byte `+5`, raw effect `+6` and flags `+0x30`.
The getter selects the higher valid effect; it merges flags from the second
attack only when that effect is selected. It then applies Mind's Eye,
Steady Hand/Nightcloak variants, Blind Eye's native attack counter and status,
stance and move-specific exceptions. Native skill arguments match the imported
`/skills.h/SkillType` annotations. The read-only enum exporter does not establish
other imported annotations as correct.

The getter and `feedback.h` agree across 3,488 calls, with unchanged fixture
player memory and preserved callee-saved registers. Its aggregate return is
insufficient for assigning per-hit bounce. The candidate records the native
return, both attack identities, active/effect/critical bytes, flags and attack
counter for live correlation. No temporary player-buffer edits or repeat RNG
rolls are used.

## Capture candidate and ABI

`gateway.S` preserves all `r0-r12`, `d0-d31`, APSR flags and FPSCR in a
328-byte frame. Native stack arguments stay at their original address. Around
hooks redirect the saved caller LR through an owned native TLS scope; before-only
taps return directly to their trampoline. Leaf return LR is restored, while
LR values computed by native callees are retained. Native and compiled fixtures
cover nested scopes and bounded-capacity failures.

The candidate uses a private 4 KiB CodeMemory arena and one SDK TLS slot;
Forge's shared context slot and legacy arena are not reused. Six additional
exact displaced-instruction words extend the project's transaction whitelist.
The installer checks the original 64-byte windows, title ID and GNU build note,
then prepares every trampoline before publishing any hook. It runs only from
Forge's boot callback before `original_sApp_run`; no hot install/remove is
provided. Existing Forge, Better Input and modern-controls sites are excluded.

Recovery r2 installs six combat hooks by default. Viewport `0xb8909c` and camera
upload `0x885844` remain untouched unless a separate camera opt-in file is present.
This isolates renderer hooks after the first enabled runtime trial exited before
the main menu; it does not establish which hook caused the exit.

The fingerprinted SDK's `nnosSetTlsValue` wrapper branches through `0x43b7bc`
to native `_ZN2nn2os11SetTlsValueENS0_7TlsSlotEj` at `0x9d80c`. This function
returns void and leaves a computed thread/slot address in `r0`. Forge's supplied
header declares a `Result` return. The capture plugin now uses its own verified
void declaration and checks the stored value with `nnosGetTlsValue`. Executing
the original getter/setter bytes passed 256 checks across 32 slots and two
synthetic thread records. Allocation and the live scheduler were not executed.
The compiled NRO fixture also retains a nonzero setter return register so it
cannot mask the old declaration error by supplying zero.

Each of 16 TLS contexts has 64 scopes and a 256-record mailbox. Scope overflow
quarantines capture until the outermost accepted scope returns. Pool exhaustion
stores a per-thread bypass sentinel. Payload words and tickets are lock-free
32-bit atomics; producer and consumer never wait or retry. The render consumer
rejects a competing thread. External hook changes disable capture while retaining
callable originals; restart is required.
Shared diagnostic increments use one weak compare-and-swap attempt and are
best-effort under contention. Context allocation tries each of 16 slots once;
it never spins on a contested slot. Mailbox payloads and per-thread serials
remain independent of these approximate counters.

The compiled NRO fixture uses explicit SDK/kernel bridges and a synthetic
collision body. It executes original normalization, melee/shot HZV and feedback
code with the real compiled capture adapter, checks local ownership in all
four session slots, rejects healing/nonpositive contributions, preserves
individual hits and exercises overflow/bypass and patch monitoring. Camera
copying is tested with supplied scope/input values. These fixtures do not execute
the original collision traversal, SDK or renderer.

## Empty RomFS activation-file failure

The second startup log identifies Eden revision `d16735f5b6` and a fatal
`stbtt_GetFontOffsetForIndex` assertion before capture's ImGui initialization.
The installed font matches the maintained Forge source byte for byte. The
installed capture opt-in file was zero bytes.

In that Eden revision, the [RomFS builder](https://github.com/eden-emulator/mirror/blob/d16735f5b6/src/core/file_sys/fsmitm_romfsbuild.cpp)
aligns file data to 16 bytes, adds a file's length to the next offset, then uses
an unstable sort comparing offsets only. An empty file and the next file share
an offset. The [concatenation builder](https://github.com/eden-emulator/mirror/blob/d16735f5b6/src/core/file_sys/vfs/vfs_concat.cpp)
sets its running end to each entry's offset plus length. When sorting places the
empty file after the font, this moves the end backward and creates a padding
entry covering the font. The reader's upper-bound lookup selects that padding.

The offline test models these decisions with the dump and listed-package file
metadata. It reproduces the capture flag shadowing `Roboto-Medium.ttf` at data
offset `9203903232`. An absent flag preserves the font there. Giving the flag
1, 2 or 8 bytes places the font at `9203903248` and preserves its header.
Listed-package metadata is not proof of current activation; the full emulator
and guest filesystem were not executed by this fixture. The symptom matches the
retained startup log. Startup with a nonempty flag remains a manual gate.

The capture build supplies a separate nonempty activation template and rejects
empty candidate-package files. Both capture and future camera opt-in files must
contain text. No emulator patch, font replacement or further NRO change is
needed for this packaging correction.

## Open gates

- Actual collision-to-normalization correlation, distinct hit identity,
  lifecycle cleanup and extended monster-part behavior.
- HZV for the resolved physical type and current part state, including mixed
  damage-type ties and attacks without a physical HZV.
- Actual hunter bounce versus visual impact-effect codes, Mind's Eye and
  non-bouncing attacks.
- Placed/environmental bombs versus owned support bomb moves; monster attacks,
  recurring ticks and all-owner optional-source exceptions.
- Critical marker behavior for special affinity outcomes, particularly the
  positive native outcome produced by Bitter Affinity.
- Active gameplay camera, blended projection, viewport and render-scale mapping.
- ABI-preserving hooks, private arena transaction/rollback, thread ownership,
  imports, ARM32 build and runtime capture trial.

R3's local hunter/palico capture, logging and diagnostic retention passed the
user's trial. The Phase 1 binding gate remains open for the other cases above.
R4's melee projection, animation and size passed the user's visual trial. R5's
palico child projectile popups, optional sources and per-hit bounce remain open.

## Reproduce research

```bash
python3 scripts/damage_numbers_research.py inspect
python3 scripts/damage_numbers_research.py export 0x170968 0x9d6f8 0x97ddc
python3 scripts/damage_numbers_research.py scan 0x3ac4 0x3ad1
python3 scripts/damage_numbers_research.py enum /skills.h/SkillType
```

Exports preserve original bytes beside imported instruction bytes and caller
references, and decompile selected functions without reanalysis. They serialize
headless access to the preserved project. Generated inputs, database exports
and logs stay outside Git; no runtime dumper is rerun.
