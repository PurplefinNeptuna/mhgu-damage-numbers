# Damage Numbers: development and research

**Current state.** r48 READY_TO_DISTRIBUTE (2026-11-14), unpublished, release at `build/damage-numbers/releases/r48/`. It removes Show other damage, adds Spread, and puts the settings on a Purple-lib Mod Options page with help in seven languages and a live preview, anchored in the world in quests and flat on screen in the village. r48 is r47 without the preview's diagnostic log; the user ran it on MHGURecomp with Purple-lib r20, so it is the last gameplay-accepted build. Eden, Ryubing and Switch hardware were not run with r47 or r48. DISTRIBUTION.json is authoritative for status and hashes; EVIDENCE.json holds every version's runs.

What the current build does at its compatibility edges:

- It zeroes the projection's m[8]/m[9] before projecting, where MHGURecomp's FSR writes its per-frame sub-pixel jitter (its `runtime/camera_jitter.c`: "zero as made").
- When the picture's shape differs from Forge's display (gu-native draws the overlay over its whole window), the projection's shape places the picture, letterboxed or stretched, inside the display (see [Projection and centre fallback](docs/worked/damage-view.md#projection-and-centre-fallback)).
- When ImGui draws on a host surface (MHGURecomp), the visible crop is ignored (see [Visible crop](docs/worked/damage-view.md#visible-crop)); otherwise a lowered render resolution draws numbers at about half size toward the top left.
- No plugin code uses i32 saturating arithmetic, which emits QADD/QSUB that stop the game on Ryubing 1.3.3; `verify.py` rejects every saturating instruction Ryubing lacks.
- Entry hooks accept another plugin's Forge hook on the same function (see [Hook sites](docs/worked/damage-capture.md#hook-sites)).
- Hits outside a usable camera view stay hidden at their world position and appear if the camera turns to them before they expire.
- Nothing logs at Forge's error level.

Runtime checks still needed after an authorized install: a hit landing outside
the camera view shows no number at screen centre, and turning the camera to it
within the lifetime shows it at the impact. Missing positions still show at
centre. Also open: kinsect hits
(local and online), a poison tick, a blast detonation, a mount finisher and a
map-fired siege shell each show one number of the expected size and colour.
Spread 0 stacks numbers on their hit, 2.0 scatters them further than r42; an
r42 INI loads with spread 0.8.

## Build and validate

Copy `build.local.example.json` to ignored `build.local.json` and supply the tool paths and copied native inputs. Use Rust nightly-2026-10-02 with rust-src, the pinned cargo-forge 1.4.1 converter built from revision `7f572ed8d0e0a6cfc562571fdbc1ad57441a8438` (retained independently of the 1.5.0 runtime crates), devkitARM GCC 15.2.0, and the Python dependencies recorded in requirements files. The Rust compiler and cargo-forge binary fingerprints are in `dependencies.lock.json`. No installed emulator package is an input.

`inputs` points to a directory with `main`, `sdk`, and the upstream Forge 1.5.1 NSO named `forge`. Expected hashes are recorded in the lock. `main` must be the MHGU 1.4.0 executable with build ID `FB08F1D20FD1204FE7070672777F869B`. Preserve supplied originals. Configure `arm_python` separately if Unicorn and the visual-test Python packages are installed in different environments.

| Command | Purpose |
| --- | --- |
| `./build.sh doctor` | Validate the compiler, converter and owned dependency fingerprints. |
| `./build.sh prepare` | Validate the prepared assets without generating or compressing data. |
| `./build.sh fetch` | Download pinned forge-rs and crates into ignored `vendor/`; the only network step. |
| `./build.sh build` | Build ARM32 ELF/NRO and stage the NRO, OpenSans.ttf and both RGBA files. |
| `./build.sh test --host` | Run portable behavior, pinned config, production installer, native pooled allocator, C++ parity and real ImGui comparison tests. |
| `./build.sh test` | Add execution of the production ARM gateways and native verification. Requires a completed build. |
| `./build.sh verify` | Check exact native imports, callback exports, game windows, layouts, asset hashes and NZCVQ restore instructions. |
| `./build.sh package` | Require complete tests for the exact NRO, then verify the candidate ZIP and adjacent guides/notices/checksums. |
| `./build.sh export --output DEST` | Export the explicit source allowlist to an unused directory. |

Preparation, builds and packaging do not install, activate, publish or push. The runtime package contains `damage_numbers.nro`, `OpenSans.ttf` and two external RGBA artwork files. The workspace coordinator packages the registered candidate with `python3 scripts/mod_distribution.py package damage-numbers`; run the full `python3 scripts/mod_distribution.py check` afterwards.

Workspace outputs are under `build/damage-numbers/candidates/VERSION/` and `work/damage-numbers/candidates/VERSION/`. Cargo caches, the unstripped ELF and converter output stay under `work/damage-numbers/candidates/VERSION/cargo-target/`. The pinned converter requires a temporary `target` symlink and loose NRO in the source root while building; the wrapper removes both when the converter exits, including on failure. An independent source export uses its own ignored build/work folders. `fetch` vendors registry sources, including the packages this nightly's build-std resolver reads. Host tests explicitly load the source-only Cargo configuration so they do not inherit the ARM target or build-std settings. Default builds and tests are offline.

After adding source files, refresh the allowlist with `python3 scripts/export_source.py --refresh`. Exports omit native game inputs, outputs, private configurations, caches and Git metadata. They retain dependency bytes exactly so Cargo checksum validation remains meaningful. Supply your toolchain/input paths in the exported repo's own configuration. No sibling mod import is needed.

forge-rs source is pinned at `d1865b68e9b194eb3eacc8bd71bbf601a5fac63e`; the native hook fixture is pinned to Forge `1671ad1526f6df5ee8d6e61a03518e0a427e202e`. The native fixture compiles the actual allocator with host JIT/TLS/cache doubles, omits ARM barriers, and uses ASan/UBSan. LeakSanitizer is disabled under sandbox tracing; the fixture counts creation and closure of every mock JIT chunk. These checks do not measure emulator allocation or establish gameplay acceptance.

The owned `assets/` folder contains the exact prepared runtime files and their hash/provenance manifest. Ordinary builds need no FontTools or image-generation dependencies, and the runtime contains no artwork payloads or compression decoder. `requirements.txt` supplies Pillow/NumPy only for visual tests. Deleted authoring tools remain recoverable from Git.

`tests/r2-pixels/` holds 24 reference scenes captured from untouched r2 rendering, at 720p/1080p and the minimum/maximum scale. Atlas hashes and composited RGB pixels must match exactly. The frozen C++ oracle headers and decoder remain test-only under `tests/oracle/`; no reference artwork or decoder is linked into the production NRO.

NRO hashes differ across build roots.

## How it works

Design, hooks and native detail are in `docs/worked/`: [Damage Numbers: capturing hits](docs/worked/damage-capture.md), [Damage Numbers: crop, projection and assets](docs/worked/damage-view.md), [Damage Numbers: GE preservation audit](docs/worked/ge-audit.md), [Damage Numbers: Mod Options and its preview](docs/worked/mod-options.md).

### Settings

`src/config.rs` derives upstream IniSerialize/IniDeserialize. Individual field wrappers define true, 1.0 and 0.9 defaults, and reject invalid or nonfinite values; Style defaults to Wilds. Spread (INI key `spread`, 0.0–2.0, default 0.8) multiplies the 17 placement offsets in `presentation::layout`; at 1.0 the products equal r42's unscaled offsets exactly, so the reference comparisons pin it to 1.0. An r42 INI has no `spread` key and loads with 0.8; its `other` key is ignored, because the derive reads only declared fields. The preview mode and random hit types are session-only `Settings` fields, never written to the INI, and have no ImGui widgets. Once save data and MtFile are available, stock `config::load` reads the file or creates it with defaults. Save uses stock ScopedMount and Writer. Reset updates session settings; Save persists them. The live settings are `runtime::SETTINGS` (`SharedSettings` in the core: atomics, no lock), because Purple-lib's get/set run in the game's update and may not lock. Every writer stores there; `consume` copies a changed value into the Model each frame, and `EDITED` stops a late INI load replacing a user edit. These are the explicitly requested forge-rs saving semantics, superseding the older custom persistence rules for this mod.

The unframed settings tree is closed by default. Its guard balances TreePop on every opened path. Free-render processing runs independently of whether that tree is open.

## Native research

Offsets are main-relative and were read from the verified original NSO; the
Ghidra database (image base `0x009c9000`) supplied decompilation and references.
The imported database contains one changed text byte, so instruction bytes come
from the NSO.

- Copied original main NSO: `work/damage-numbers/inputs/main`, SHA-256
  `b3935415499549adaf4c0e89822831daa175cfd456a7990791c79be3997645ef`.
- Main build ID: `FB08F1D20FD1204FE7070672777F869B`.
- Verified decompressed main image: SHA-256
  `24c208c395c65727b21b5fa91af8381396e70cf421b0fcfee0345d8743298110`.

### How monster HP falls, and damage outside ordinary hits

The native routes (part counters, the pending amount, the collision scope, kinsect, poison, blast, mount, map-fired shells, ballista, siege weapons) are in `.agents/skills/mhgu-understanding/references/damage.md`, sections How monster HP actually falls and Shells and siege weapons. Each route here has its own hook: poison is hook 8, blast hook 9, the mount finisher hook 10; kinsect, map-fired shells and ballista arrive through the collision path. The mount finisher label is an inference; if gameplay shows a different event, only the label changes, the amount is still real HP damage. Hits with unrecognised tags stay hidden.

Status and stage records carry no owner and are shown for every player, like
monster and object damage. Status positions are the monster root, not an
impact point. Poison numbers are gray; blast and mount use the weak-point colour
because each comes from its own native subtraction, not HP watching.

## Validation scope

Host tests exercise copied-value capture nesting, scope overflow/quarantine, ownership/source policy (including kinsect, stage-owned shells, poison/blast/mount records and remote-hit publication without bounce association), bounded model/mailbox overflow, duplicate suppression, the monster list and HP bookkeeping, disabled settings, concurrent publication and queued records surviving thread exit. Failed/non-finite position reads inside nested scopes, valid world origin, matrix order, 720p/900p/1080p/1440p mapping, crop reads, invalid dimensions/mappings, pointer overflow, applied versus requested size, repeated mode changes and centre fallback for all three styles are covered. `tests/viewport.py` executes the original ARM SetCrop wrapper and applied-size stores with an NVN double.

The config harness compiles the pinned upstream config/INI modules and derives, plus production src/config.rs, against test MtFile and counted-mount I/O. It checks automatic default-file creation, individual defaults and invalid parsing, all style round trips, spread limits, an r42 file with `other` and no `spread`, preview fields never saved, Save and Reset.

The hook harness compiles production src/hooks.rs and the stock Patch wrapper against a native API double. It rejects invalid recipes, reserved/overlapping spans and changed game signatures; injects failures before, during and after branch publication at all eleven sites; checks invalid/non-executable trampoline and returned-record evidence; and exercises cache, restoration and foreign-change failures. Owned changes must roll back and capture must remain disabled on failure. Foreign changes remain untouched, valid continuations remain published and incomplete restoration requires restart. Success activates all eleven.

The native pool fixture compiles pinned Forge hook.c with JIT/TLS/cache doubles and omitted ARM barriers. Eleven ordinary hooks share one chunk and use 176 bytes of slots. Allocation failure, rejected fallback and full two-chunk capacity preserve target bytes. ASan/UBSan run; mocked JIT allocations are counted and closed because LeakSanitizer cannot run under sandbox tracing. This is host evidence, not measured emulator allocation.

The C++ r25 oracle comparison covers 243 combinations of style, critical outcome, HZV and age, producing 1,944 draw records. Exact identities/colors/text and bounded float differences are checked. Another 2,800 cases compare source/owner policy, finalized damage/critical outcomes and native matrix projection against r25, including invalid matrices, perspective, translation and clipped points. The r25 oracle covers primary kinds 0–4 only, so the kinsect rule does not alter it. The real renderer harness compiles production src/render.rs with pinned Forge font/draw wrappers and actual ImGui 1.92.8. All three styles are compared against 24 frozen r2 scenes at 720p/1080p and minimum/maximum scales, including textures, glyph atlas bytes, draw commands, indices, vertices and exact composited pixels. Repeated initialization is checked against duplicate resource creation. File-operation doubles inject short reads and count handles; missing/wrong-size assets, font fallback, context replacement and shutdown cleanup are exercised.

The ARM fixture executes all twelve gateways from the production NRO in Unicorn 2.1.4. Its helper doubles deliberately clobber registers, floating-point state and flags; it checks all GPRs, d0–d31, FPSCR, NZCVQ, LR/SP, native stack arguments, around-return paths, and all 16 GE patterns with preserved and changed GE. Helpers are doubles; this is not a native gameplay run.

`verify` additionally compares compiler-derived Rust/C++ ARM32 Hook size/alignment/pointer offsets, saved frame, ImGui/font/texture/draw layouts; checks callback exports, all strong imports against actual main/SDK/Forge exports (including `igSetItemTooltip`), exact game signatures, asset receipt hashes and twelve NZCVQ-only restoration instructions.

Gameplay, config persistence across a real restart, loader allocation limits, GPU output and emulator lifecycle remain manual checks. No installation, activation or game launch is performed by these commands. Exact counts for each candidate are in EVIDENCE.json.

## Do not

- **Replace hooks 3 and 4 with main+0x18a48 from `forge_onUpdate`.** The A/B gate failed: 6 of 64 hits within 2 px, median 7.2 px, max 102.4 px, growing with camera motion. See `work/damage-numbers/projection-call/RESEARCH.md`. The user closed this idea on 2026-10-30.
- **Move an off-screen hit to screen centre.** A position a usable camera clips stays hidden until the camera turns to it. Centre is only for missing positions and missing or invalid cameras.
- **Require native and display dimensions to match in the viewport guard.** That rejects legitimate resolution scaling.
- **Log per frame or above budget in a diagnostic.** Stock Forge filters below ERROR and its message buffer is 512 bytes, so cap lines per process and keep them short. A hit cap that ends the run early loses the later segments.
- **Use `i32::saturating_add`/`saturating_sub` in plugin code.** They emit QADD/QSUB, which stop the game on Ryubing 1.3.3; `-dsp` does not remove them in ARM mode.
- **Expect an NRO hash to match across build roots.** Hashes differ by root.
- **Accept `uVillageCamera` (main+0x1730e10) in the camera capture.** Its samples reached the mod in r46 but lagged the view (user report on MHGURecomp); the cause is unknown, so the village preview stays flat.
