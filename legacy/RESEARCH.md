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

Native attack fields: primary owner `+0xc`, kind `+0x10`;
secondary kind `+0x6e`, owner `+0x70`; impact
`+0x20..+0x28`; damage type `+0x3a`, effect `+6`,
crit `+0x54`, hurtbox `+0xac`. Hurtbox bytes
`+6`/`+8` select HZV/accumulation part. Reject
healing type `0x1d` and nonpositive contributions.

`0x9d6f8` adds its signed return to `monster parameter +0x488+2*part`. The consumer
sums eligible parts for HP before part-break modifiers; HP subtraction is at
`0x98b04`. Nearby clamps make HP-difference capture lose lethal values
and merge hits. Enemy vtable `+0x2f8` is an on-hit notification with a
no-op base; `0x9dd40` tracks threat. Neither applies HP damage.

Ordinary collisions combine physical/elemental damage before native integer
conversion/normalization. Capture the completed contribution with its impact
identity. Extended parts call vtable `+0x310` and return zero;
coverage remains unresolved.

## Ownership

`0x276278` reads local index via `main+0x188ca48`, then player table
`main+0x1885b58` at `+0x1c+4*index`. Valid indices: 0–3. `ownership.h`
matches original execution in all four slots.

Palico getter `0x285c6c` reads current work at `object+0x3ac4`,
master byte `work+1`, then player lookup `0x274ce4`.
Constructor work is inline; DTI reports `obw->mMasterId` at
`+0x3ad1`. Use the current pointer, which may differ. Serialized
`work+0x34d` is not the master ID.

Fixtures redirect work away from inline storage and compare local/remote
ownership with the native getter. Invalid indices, missing work/master are
hidden. Snapshots cover direct/secondary projectiles. Solo hunter/Palico and
multiplayer filtering passed; Prowler remains unconfirmed.

## Camera

Look-at `0x7c5e4c`, perspective `0x7c617c` and product
`0x29d48` use row vectors: `world * view * projection`, negative camera Z,
zero-to-one depth. `camera.h` transposes the product. C++ matches copied
native matrices across translation, rotation, FOV and aspect fixtures; tangent
is a float32 stub, not live camera selection.

`0xb8909c` reads camera `viewport+0x24`, rectangle
`+0x38..+0x44`, view/projection `+0xb0/+0xf0`, alternates
`+0x130/+0x170`. Interpolation/render-scale adjustments require final
uploaded matrices, gameplay-viewport identity and Forge-coordinate mapping;
FOV/unblended matrices alone are insufficient.

Upload `0x885844`: distance `s0`, context
`r0`, view/projection `r1/r2`, rectangle
`r3`. First eight bytes push registers/adjust frame pointer.
Capture copies only within viewport scope with `uQuestCamera` vtable
`main+0x1730d0c`; live selection/render-scale coverage was initially pending.

R4 maps zero-origin full-screen rectangles with matching aspect to Forge
`ImGuiIO::DisplaySize`. The inspected NVN backend uses swapchain texture
size/framebuffer scale one. Offset/split viewports are unsupported. Hide camera
data older than 200 ms; identity changes clear numbers. Rendering dereferences
no camera pointer.

SDK `nnosGetSystemTick` wrapper `0x9e640` returns 64 bits in r0/r1;
frequency `0x9d840`: 19,200,000 Hz. Original-wrapper fixtures cover
>32-bit values with a supplied hardware counter. R4 subtracts captured ticks
before float conversion, preserving lifetime despite mailbox delay. Records: 248
bytes, player `+236`, tick `+240`; other offsets
unchanged.

Forge free-render runs with its window closed. Pinned cimgui draws eight black
outline passes plus color, with outlined crit lines/reserved vertical space.
ImGui rounds font sizes; NVN supports dynamic textures. The r4 user trial
accepted rasterization, GPU output and melee alignment at logged 1920x1080.

R4: 51 hits (18 hunter, 33 Palico); Palico tags `0x030403`: 15,
`0x030402`: 15, `0x030802`: three. Direct-primary filtering
rejected the latter 18, matching the panel and missing boomerang/barrel report.
Tags pack `+0x10` primary kind, `+0x6d` subtype,
`+0x6e` secondary kind. Rejected kinds are 2/3; secondary
pointer/current master work already resolved ownership.

R5 admits those two tags only with `Owner::LocalPalico`, retaining
ownership/healing gates. Boomerang/barrel labels are inferred from actions and
HZV/effect patterns; both displays passed manually. Unknown physical HZV stays
white. DTI `cOtWeaponBase` exposes boomerang attack/element/affinity;
equipment/damage data stayed unchanged.

## Hitzone and bounce capture

Tap `0x16dd7c` precedes selected-type storage: `r0`
cut/blunt, `s20/s22` adjusted HZVs; weighted ties choose cut.
Native/C++ fixtures pass 1,440 HZV/weight/state/player/part combinations.
Returned damage factor times 100 would lose mixed attacks' unweighted HZV.

Scope the tap within `0x16dc6c` and its impact. Shot helper
`0x178660` returns physical factor through its second stack argument;
only `+0x30 & 1` attacks have shot HZV. Compiled fixtures cover 44/45/80
and no physical HZV.

`0x2a87a8` compares attacks `player+0x3020`/`player+0x30d8`:
active `+5`, effect `+6`, flags `+0x30`.
It chooses the higher valid effect, merging second-attack flags only if
selected. Mind's Eye, Steady Hand/Nightcloak, Blind Eye counter,
status/stance/move exceptions follow. Skill arguments match `/skills.h/SkillType`;
the enum exporter verifies no other annotations.

`feedback.h` matches 3,488 native calls with player memory/callee-saved
registers preserved. Aggregate return cannot assign per-hit bounce. Capture
records both identities, active/effect/crit bytes, flags and counter for
correlation; no player edits or repeat RNG.

## Capture candidate and ABI

`gateway.S` preserves `r0-r12`, `d0-d31`, NZCVQ/FPSCR
in 328 bytes; native stack arguments stay put. Around hooks redirect caller LR
via owned TLS; before-taps return to trampolines. Leaf LR is restored;
native-calculated LR retained. Fixtures cover nesting/capacity failures.
R17 leaves GE as set by the helper, based on the eight-site
[GE liveness audit](GE_PRESERVATION_AUDIT.md); this is not a general gateway
contract for arbitrary new interior hooks.

Capture owns one 4 KiB CodeMemory arena/SDK TLS slot, separate from Forge's
shared context/legacy arena. Six exact displaced words extend the transaction
whitelist. Verify original 64-byte windows, title/GNU build note; prepare all
trampolines before publishing hooks. Boot-only before `original_sApp_run`, no hot
install/remove. Exclude Forge/Better Input/modern-control sites.

R2 enabled six combat hooks; `0xb8909c`/`0x885844` required
camera opt-in after the first enabled trial exited before main menu. This
isolation does not identify the faulty hook.

`nnosSetTlsValue`: wrapper `0x43b7bc`, native `_ZN2nn2os11SetTlsValueENS0_7TlsSlotEj` at
`0x9d80c`, void return with thread/slot address left in
`r0`. Forge declares `Result`. Capture uses verified
void ABI/readback via `nnosGetTlsValue`. Original bytes passed 256 checks
across 32 slots/two synthetic threads; no allocation/live scheduling. NRO
fixtures retain nonzero r0 to expose the old ABI error.

Sixteen TLS contexts each hold 64 scopes/256 mailbox records. Overflow
quarantines until the outer accepted scope returns; pool exhaustion installs a
bypass sentinel. Lock-free 32-bit payload/ticket atomics never wait/retry;
competing consumers are rejected. External hook changes disable capture but
retain originals until restart. Diagnostics use one weak CAS (best-effort);
allocation tries each slot once without spinning. Records/serials are
independent of counters.

Compiled fixtures run the real adapter with original normalization, HZV and
feedback routines, synthetic collision/SDK/kernel bridges and supplied camera
inputs. They cover four local slots, healing/nonpositive rejection, separate
hits, overflow/bypass and patch monitoring; not original collision traversal,
SDK or GPU renderer.

## Empty RomFS activation-file failure

Second startup: Eden `d16735f5b6`, fatal `stbtt_GetFontOffsetForIndex` before ImGui
initialization. Font matched Forge byte-for-byte; opt-in file was empty.

Eden's [RomFS builder](https://github.com/eden-emulator/mirror/blob/d16735f5b6/src/core/file_sys/fsmitm_romfsbuild.cpp) aligns data to 16 bytes, advances by file length and
unstable-sorts offsets. Empty/next files share an offset. Its [concatenation builder](https://github.com/eden-emulator/mirror/blob/d16735f5b6/src/core/file_sys/vfs/vfs_concat.cpp)
resets running end to each offset+length. If the empty file follows the font,
end moves backward, padding covers the font and upper-bound lookup selects
padding.

Offline dump/listed-package metadata reproduces `Roboto-Medium.ttf` shadowing at
`9203903232`. Absent flag preserves it; 1/2/8-byte flags move it to
`9203903248`, preserving the header. This is not activation/full-emulator
proof. It matches the log; nonempty startup passed r3 ([RUNTIME.md](RUNTIME.md)).

R7 removes flag checks/templates. Eight hooks start at boot after
title/signature, clock, TLS and transaction gates; old flags are ignored.
Packaging still rejects empty RomFS files. No emulator/font patch.

## Open gates

Manual acceptance covers hunter/Palico capture, logging/history, visuals,
projectiles/white HZV, LS/bowgun crits and separate impacts, bounce/Mind's Eye,
multiplayer exclusions, persistence and monster-to-monster damage.

Remaining coverage:

- Independent collision/normalization accuracy, lifecycle edge cases and extended
  monster parts.
- Mixed-type HZV ties and broader monster-state/no-physical-HZV cases.
- Placed/environmental objects, recurring ticks and all-owner exceptions beyond
  the implemented source families; Prowler-specific attacks.
- Special affinity outcomes, particularly Bitter Affinity's positive native result.
- Additional blended cameras, viewports and render scales.
- Broader hook/rollback/thread/import behavior beyond fixture and accepted trials.

## R6 source, bounce and storage bindings

`0x170968` dispatch `+0x6d`: 1 hunter, 2 monster, 4 Palico;
others use generic `0x171350`. Monster kind 1 resolves via primary
`+0xc` or secondary `+0x70` with `+0x6e == 1`. R6
optional sources admit subtype 2 and known generic subtype-8 hunter-owned
children. `0x030802` stays owned Palico support. Unknown families are
hidden; bindings use saved exports/DTI, no dumper.

Keep latest contribution per thread until next collision/feedback. Bounce needs
one active hunter attack, native feedback/raw effect zero, unchanged
pointer/flags/crit/counter, matching epoch/serial/target/thread within 100 ms.
Earlier multi-hits, other threads, two active attacks and prevention results
keep HZV color. Recoil/Mind's Eye passed manually; damage/RNG is never rerun or
edited.

Verified ARM32 FS ABI: 4-byte handles/aligned 64-bit offsets; generic nnheaders'
8-byte handle is incompatible. R6 used two SD binaries; r8 ignores them and
imports Forge's counted `savedata` mount like Better Input. Helper
resolves `sSavedata`, calls main `0x878da0`, releases with normal
commit. Saved DTI: 4,000-byte `cSystem` descendant. Builder checks
`CommitSaveData`; no new dump.

Current storage opens only `savedata:/damage_numbers.ini`, range-checked text
within 255 bytes. Required fields are `enabled`, `style`, `scale` and `lifetime`.
No version marker is written/required; old `version=2` remains readable until
an explicit save drops it. Missing/invalid primary uses defaults. Backup files
are ignored and untouched. Validate, write, flush, read-verify and explicitly
commit through Forge's counted mount. Balance handles/mounts on every failure;
keep session settings, without promising previous disk contents survive.

Load at initialization, retry unavailable storage at most once/second on the
single render consumer; manual edits cancel retries. Only Save settings writes,
outside combat detours.

SDK FS errors may auto-abort. Scoped disabler `SDK+0x4d004` saves one-word
effective context/installing `g_AllReturnContext` in thread TLS;
`SetCurrentThreadFsContext` (`0x4d080`) restores every exit. Handler
`0x4cffc` returns AbortSpecifier 2; decision `0x51b64` returns
false. 36 initialized GOT/context/TLS fixtures with prior Abort callback
verify restoration, not live mounting/storage.

## Reproduce research

```bash
python3 scripts/damage_numbers_research.py inspect
python3 scripts/damage_numbers_research.py export 0x170968 0x9d6f8 0x97ddc
python3 scripts/damage_numbers_research.py scan 0x3ac4 0x3ad1
python3 scripts/damage_numbers_research.py enum /skills.h/SkillType
```

Serialized headless exports retain original/imported bytes, callers and selected
decompilation without reanalysis. Generated inputs/exports/logs stay outside
Git; no runtime dump.

## R10 cosmetics: Forge APIs and bounds

Rechecked the installed patched Forge export table and pinned cimgui 1.92.8
headers. Font config/atlas registration, baked-font glyph lookup, filled
triangles and the string-array combo are exported. `Graphics::initialize()`
calls plugin ImGui initialization after adding its font and before NewFrame.
The NVN backend supports atlas texture creation/updates. This adds no native
capture hooks or private rendering backend.

Font registration uses the same context/allocator setup as the existing plugin.
ARM32 uses 16-bit ImWchar; glyph/config/baked-font offsets come from the pinned
headers. Fixtures check compiled argument passing and disable atlas ownership
of the module's static font data. Native measurement sums actual glyph advances
and visible bounds. The star is concave, so it is drawn as eight triangles per
layer instead of passing it to the convex-polygon API.

The 17 fixed candidates minimize overlap then squared displacement, keeping
input order for equal scores. Reservations include maximum-pop glyphs, solid
markers and small shadows, excluding glow. New hits retain bounded offsets;
existing hits retain theirs until expiry or context/style/size/viewport reset.
Classic collision bounds include its existing straight outlined markers.

## Current presentation and Rise implementation

Wilds uses Open Sans Semibold digits, a centered soft shadow and a near-black
outline of 0.5 px at the default 24 px size. Its gold halo is removed. Procedural
positive/negative stars scale to 1.5× their R19 dimensions, with pixel-rounded
solid bounds and unchanged digit gap. Wilds stamps from 1.2× over 0.2 seconds,
holds, then rises 30 px at 720p during its final 0.2-second fade. Classic retains
Forge's font, outlined digits, crit lines and original pop/drift.

R25's 720p bases are Wilds/Classic 24 px and Rise 20 px. Rise weak points use a
1.4× boost; all bases scale with resolution and saved size. Stable placement
reserves maximum entrance bounds, checks 17 nearby positions within 1.6 font
sizes horizontally and one vertically, and caches offsets until expiry. Crowded
bursts may overlap. Camera/context/viewport/style/size changes reset caches.

Rise preserves `tint()`: confirmed bounce wins, then physical HZV at least 45
selects weak color, otherwise white. Its colors are `#808080`, `#F0B41E` and
`#FFFFFF`. Companion ownership does not select gray; affinity affects only
the slash. Open Sans glyphs widen by 1.08, with eight opaque black outline
passes at radius 0.10 of font size and the open glyph shadow underneath.

Rise entrance follows 0.64 -> 1.20 -> 0.88 -> 1.00 over the smaller of 0.30 seconds
and 30 percent of Duration. Exit uses the smaller of 10/30 seconds and 35 percent
of Duration: its first tenth kicks down +4 design units, then rises to -50 while
fading. These are authored UI timings. Duration remains the complete lifetime.

Positive crit uses an original red beam/pale core; negative uses violet. It
expands for 0.05 seconds, thins/fades for 0.15 seconds, then expires independently
of digits. Peak bounding width is 48 px at 720p. The sole selected slash is
three-quarter height, centered on the projected raw `Event.position`, with equal
upward/downward reach and no ground cutoff. It inherits no digit pop, slider,
collision offset or exit fade. All four `HitKey` fields determine a stable slope
within [-0.25,+0.25], approximately +/-14.04 degrees. Slashes draw before digits,
clip to the viewport and remain outside the collision solver. Terrain/scene
occlusion is unavailable.

`fonts/rise-open.json` and `scripts/prepare_damage_numbers_rise.py` own the beam
recipe. No Rise/Wilds screenshot pixels, proprietary font or effect asset enters
the build. DN-LZ1 decode runs once per ImGui context; failure is attempted once
per context and logged once per process while digits continue. An ImGui shutdown
hook unregisters/frees CPU texture data. Forge's NVN shutdown still has no complete
GPU teardown facility. No new native hook, CodeMemory arena or TLS slot was added.

Real pinned ImGui previews exercise tilted indexed triangles, bilinear sampling,
RGBA interpolation, shared-edge coverage and command clipping. Dark/bright height
comparisons and 40-frame timelines remain under the versioned R21 work output.
They cover polarity, color, crowded hits and independent beam expiry without an
MHGU GPU/gameplay result. Native GE/signature audits remain required.
