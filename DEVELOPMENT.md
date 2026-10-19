# r33 mount knockdown colour: DEVELOPMENT

User request (2026-10-20): the mount knockdown (hook 10, main+0x9978c) is read
from its own native subtraction, not HP watching, so it now uses the weak-point
colour like blast. `tint()` maps `Status` to Weak; nothing else changed from r32.
The unrun r32 candidate stays in its own build folder. Gameplay NOT RUN.

# r32 poison and blast colours (superseded candidate)

User request (2026-10-20): poison ticks use the bounce gray and blast uses the
weak-point colour, provided both come from their own native source rather than
HP watching. They do: poison comes from hook 8 (main+0xa2664) and blast from
hook 9 (main+0xaac54). r31's single Status family is split into `Poison` (wire
6), `Blast` (wire 7) and `Status` (mount finisher, wire 4); `tint()` maps them
to Gray and Weak, so Rise's weak-point enlargement also applies to blast. Kinsect
was already an ordinary Attack since r31, independent of Show other damage.
Hooks, signatures and gateways are unchanged from r31. The unrun r31 candidate
stays in its own build folder. Gameplay NOT RUN.

# r31 kinsect, status damage and opt-in other damage (superseded candidate)

The user reported that r30 shows no numbers for mount, Insect Glaive kinsect,
blast or poison damage. They asked for the native functions to be located
statically, because they cannot reliably produce Insect Glaive or mount runtime
evidence. They also allowed an opt-in "Show other damage" setting, off by default,
that shows monster HP loss the hooks did not explain, with a tooltip warning that
these numbers can be unknown damage. r31 is an unapproved candidate. Gameplay is
NOT RUN. The approved r30 release, candidate and receipts are untouched; r31
uses `build/damage-numbers/candidates/r31/` and the matching work path.

## How monster HP falls (static research)

Offsets are main-relative and were read from the verified original NSO; the
Ghidra database (image base `0x009c9000`) supplied decompilation and references.

- The monster work block is `*(monster+0x1428)`. The network sync record
  `nFestaNetwork::netEmMasterChange` (DTI `mParam.hp_vital`, s32) is filled by
  main+0xcba54 from `work+0x370`. The regeneration helpers main+0x9ce9c/0x9cf7c
  clamp `work+0x370` against the maximum at `work+0x374`. The same record maps
  the root position to `work+0x40/44/48`.
- HP is not reduced at hit time. main+0x9d6f8 (hook 1) adds each hit's
  contribution to the part counter `work+0x488+part*2`. The per-frame monster
  update main+0x97ddc sums the raw part counters (`local_4c += counter`) and
  subtracts the sum from HP. It then subtracts the s16 pending amount at
  `work+0x5d66` and clears it.
- All five hit handlers are reached from the collision function (hook 0):
  main+0x171350 by call and 0x172bf0/0x173a24/0x174c0c/0x176ad0 by tail branch
  from 0x170a20..0x170ab0. The collision scope therefore encloses every hit.

## Why each reported type was missed, and the r31 source

| Damage | Native path | r30 behaviour | r31 |
| --- | --- | --- | --- |
| Kinsect | Collision path. The w13 shell hit setup main+0x4816c0 calls the shared shell hit initialiser (vtable +0x140, main+0x4a2790) with subtype 1, then rewrites the entry: +0x10 = 6, +0xc = kinsect, +0x6e = 0, +0x70 = owner hunter (main+0x4a1b6c). | Owner resolved to the hunter, but `source_outcome` had no rule for primary kind 6. | Accept tags (6, 1, 0) with a known owner as Attack. Ownership rules are unchanged. |
| Poison | Poison tick main+0xa2664 clears `work+0x5d66`. While `work+0x5e08 & 2` is set it advances the timer at `work+0x5ca8`, and at each interval stores the poison table's s16 +0xc in `work+0x5d66`. | Never a hit; no hook saw it. | Around hook 8 at the function entry reads `work+0x5d66` after the native function returns. |
| Blast | Detonation main+0xaac54 obtains the blast table at `(*(monster+0x75ec)+100)+0x54` (the getter main+0x71078 is that expression) and returns early when it is null. Otherwise it resets the blast point and adds the table's s16 +0xc to the part counter directly. | Bypasses main+0x9d6f8. | Before-only tap 9 at the function entry reads the same table. |
| Mount | Hits on a ridden monster (`main+0x812a8`: ride action states or category 0xd) go to main+0x170adc, which adds the hit to the rider's mount gauge (main+0x2f6778), not to HP. In category 0xd the monster update reaches main+0x99764, loads a fixed amount from monster data +0x8c (×1.5 when the unidentified singleton query main+0x49938 returns ≥ 2), subtracts it from HP at 0x9979c and sets action 0x2b. | Not a hit and not `+0x5d66`. | Before-only tap 10 at 0x9978c reads r4 (the amount) and `[sp+60]` (monster+0x1428). |

Identifying the 0x2b branch as the mount finisher is an inference. It depends on
the ride-state gate, the rider-gauge path and the mount point `work+0x5d9a`
(DTI `mParam.nori_dmg_point`, updated by main+0xa3528 from the same update).
No string names the branch. If gameplay shows a different event, only the
label changes; the amount is still real HP damage.

Status records carry no owner and are shown for every player, like monster and
object damage. Their position is the monster root, not an impact point.

## Opt-in other damage

`Settings::other` (INI key `other`, default false) enables the per-frame HP
check. `sEnemy::Instance` (main+0x1851fe0) lists up to ten monsters at +0x1c;
main+0x5ceb4 (`getMonsterInMap`) reads the same slots. Every published damage
record, including remote and unclassified hits that stay hidden, repays HP loss
in `Vitals`. Loss nothing explains waits 0.15 s for late mailbox records, then
becomes one ownerless number at the monster root. Captured damage the game never
subtracts (HP floors, capture limits) expires after 1 s. A new monster pointer
or maximum resets the baseline; healing is ignored. Remote hunters' damage in
online play, traps, cannons and the environment appear here as unknown damage.

Capture now publishes all finalized hit amounts, not only shown families. The
consumer's visibility rules are unchanged, and bounce association still uses
only the previously published local/ownerless subset.

## Hook sites added

| Hook | Main offset | Kind | Displaced words |
| --- | --- | --- | --- |
| 8 | `0xa2664` | Poison tick entry, around | `push {r4-r7,r11,lr}`, `vpush {d8}` |
| 9 | `0xaac54` | Blast detonation entry, before-only | `push {r4-r8,lr}`, `sub sp,sp,#8` |
| 10 | `0x9978c` | Mount finisher, interior before-only | `ldr r7,[sp,#60]`, `ldr r0,[r7]` |

The relocation whitelist adds only the two loads at hook 10. Neither reads PC.
A scan of all ARM branches found no branch to 0x99790, the second displaced
word. 0x9978c is reached by fall-through and from the `blt` at 0x99770, where r4
already holds the unscaled amount. The poison function's tail branch at 0xa2740
returns through the substituted LR, like the collision function. No workspace
mod patches these offsets, and none is in the reserved list.

The functions containing hooks 8–10 (0xa2664, 240 bytes; 0xaac54, 332 bytes;
0x97ddc, 13,472 bytes), and the poison tick's caller 0xc8ddc (2,056 bytes),
contain no SEL, MRS, MSR or GE-setting parallel add/subtract encoding. The
reduced NZCVQ-only restore therefore covers the new sites too. The gateway
source hash in `dependencies.lock.json` was updated for the three new ENTRY lines.

## r31 validation (static/host/build only)

Host: 21 core tests, including kinsect ownership/policy, poison/blast/mount
records, remote-hit publication without bounce association, visibility, the
monster list and HP bookkeeping; 5 config tests with the new key; 8 installer
tests over all eleven sites; the native pool with eleven hooks (176 bytes); 243
parity scenarios and 2,800 policy/projection cases (the r25 oracle covers
primary kinds 0–4 only, so the kinsect rule does not alter it); 24 renderer
scenes with zero pixel differences. ARM: 352 gateway calls across twelve
gateways and `verify` with 15 game windows matched against the original NSO,
twelve NZCVQ restores and 69 resolved imports, including `igSetItemTooltip`.
None of this establishes gameplay behaviour.

Runtime checks still needed after an authorized install: kinsect hits (local
and online), a poison tick, a blast detonation and a mount finisher each show
one number of the expected size. With "Show other damage" off nothing extra
appears; with it on, solo hits and status damage add no extra numbers, while trap or
cannon damage does.

# r30 visible-crop correction: READY_TO_DISTRIBUTE (preserved)

The user approved distribution on 2026-10-18. The validated candidate NRO and
three assets were promoted unchanged, without rebuilding. This approval does
not add an affected-device gameplay result. The local, unpublished release was
archived unchanged for r31 development to
`backups/accepted/damage-numbers/r30-before-status-damage/release/damage-numbers-r30.zip`.
The candidate and pre-promotion records are preserved under
`backups/accepted/damage-numbers/r30-distribution-approved/`.
Keep r30 build outputs and receipts immutable. Further runtime edits require a
new version and output directories.

MHGU can display a smaller crop of an unchanged swapchain allocation. Forge's
ImGui DisplaySize describes that allocation, so the previous mapping could use
1920×1080 bounds for a visible 1280×720 region. The user reports correct docked
placement and displaced handheld placement on Switch and Android. Exact live
dimensions and the r30 correction remain unconfirmed on those devices.

The render callback now obtains the actual sFestaRender singleton through
Forge's exact-type ID lookup (0x300a6cd8). The sRender base ID is not registered
for that object. Checked copies of inherited mDisplaySize at +0xb8/+0xbc provide
the applied crop width/height in backing-texture pixels. Native main+0xbc0874
calls main+0x86a24c to SetCrop(0,0,mScreenSize), updates that pair, then presents.
The fields at +0xb0/+0xb4 are requested dimensions and may still be pending.
Saved object-kind DTI and original-byte research agree on these offsets.
See research/visible-crop.json for the native contract and provenance.

The size is re-read each render callback. It must be positive and fit inside
Forge's finite backing dimensions. The same size reaches camera projection,
number/effect scaling, placement bounds, clipping and screen-centre fallback.
Layout's existing viewport comparison discards cached offsets after a size change.
The native matrix multiplication and camera-admission policy are unchanged;
same-aspect rendering at another internal resolution remains supported. No
hardcoded 720p/1080p multiplier or additional hook is used. Stock Forge is unchanged.

An unavailable/invalid crop retains the previous Forge bounds to keep damage
visible and emits one lifetime-deduplicated warning once a player exists. That
fallback can retain the original displacement. No native pointer is cached or
dereferenced by deferred rendering. A twelfth signature window checks the crop
call and applied-field stores before any of the original eight hooks are installed.
The gateway assembly and its NZCVQ/GE contract are unchanged.

r29's diagnostic module, screen crosses and per-hit logs are retired in Git;
its exact candidate remains available. r28/r29 delivery hashes and copied records
are preserved under backups/accepted/damage-numbers/r29-before-visible-crop/.
New outputs use build/damage-numbers/candidates/r30/ and the matching work path.

New tests cover crop reads, invalid dimensions/mappings and pointer overflow,
applied versus requested size, repeated mode changes, all-style geometry and
centre fallback. tests/viewport.py executes the original ARM SetCrop wrapper
and applied-size stores with an NVN double. Existing host/renderer/gateway tests
remain required. These checks do not establish GPU or affected-device acceptance.

# r29-diagnostics (preserved)

Temporary diagnostic candidate for the user-reported misplaced numbers on
real Switch and Android. r28 starts and shows damage according to the user;
the precise loader/plugin hashes have not been supplied. The current candidate
adds evidence collection, not a placement fix. No installation or launch is implied.

The camera record retains its 248-byte ABI. Camera-only unused words carry
checked copies of sCamera::Viewport metadata: scene texture +0x2c, test camera
+0x28, visible +0x30, and packed subscene/number/mode/display at +4/+0x31/+0x33/+0x34.
The original class/matrix admission rule remains intact when metadata is unreadable.
These fields are ignored by the damage-record decoder. Native pointers are logged
as numeric identities and are never dereferenced by diagnostics after capture.

Only the render consumer formats logs. Limits are 16 distinct camera/viewport
metadata snapshots, 32 first-visible hit keys and 4 display changes per process.
Budgets do not reset when the player, camera or ImGui context changes. Each hit
includes its raw impact, camera matrices, rectangle, camera freshness/admission
result, projected point or fallback, and optional laid-out text coordinates.
ERROR severity is deliberate because stock Forge filters at ERROR by default.
Lines stay below its fixed 512-byte message buffer and go to svcOutputDebugString,
which the emulator captures. No save-data/SD log writer or settings change is added.

Cyan marks the calculated screen centre. Magenta marks projected impacts and
Yellow marks fallback. Crosses use the existing background draw list and bypass
digit placement. The original eight hooks and pinned assembly remain unchanged.
Read-only original-byte research is under `work/damage-numbers/placement-r28/`.
The scene-texture field is verified by native main+0xb89900 and saved DTI;
an affected runtime sample is still needed to establish the cause.

r28 build/work/delivery directories remain intact. Prior distribution/support
records and hashes are copied to
`backups/accepted/damage-numbers/r28-before-projection-diagnostics/`.
New output uses `build/damage-numbers/candidates/r29-diagnostics/` and the
matching work directory. EVIDENCE.json records exact validation and hashes.

## r28 hardware-test rebuild (preserved)

PIC remains enabled. The target adds `-nostartfiles` and `-Wl,-z,text` to
exclude GCC startup objects and reject dynamic relocations in read-only sections.
Gameplay, hooks, settings and assets are unchanged apart from build identification.
The final NRO verifier also rejects DT_TEXTREL/DF_TEXTREL.

The accepted r27 release and its records are preserved at
`backups/accepted/damage-numbers/r27-before-hardware-link/`.
New output uses separate r28 candidate and work directories. No installation,
launch or hardware acceptance is implied. The updated loader is not bundled.
See EVIDENCE.json for the exact build and validation results.

# Development and research

Previous accepted Switch build: r27 READY_TO_DISTRIBUTE with upstream Forge 1.5.1.
Earlier version claims below retain their recorded scope.

## r27 projection and centre fallback

A failed impact-position copy previously left `(0,0,0)`, which could be projected
as a real world position. Capture now marks the position unavailable while keeping
the collision scope and native return chain intact. The consumer represents a
missing position with `None`. A genuine captured world origin remains valid.

Missing positions, missing/stale/invalid cameras and clipped projections now use
the display centre, with normal animation and overlap spacing. This includes hits
behind or outside the camera. Invalid display dimensions still prevent drawing.
Normal projected positions retain the existing matrix, scaling and placement rules.

The viewport guard checks origin and aspect ratio, not presentation-target
ownership. A same-aspect offscreen view can still pass. Requiring equal native
and display dimensions would reject legitimate resolution scaling, so r27 retains
the guard. Read-only native exports and original-byte comparisons are retained in
`work/damage-numbers/projection-r27/native/`. They do not identify the reporter's camera.

Regression tests cover failed/non-finite position reads inside nested collision
scopes, valid world origin, matrix order and 720p/900p/1080p/1440p mapping. Fallback
tests cover all three styles, absent/invalid cameras and clipped points.
The user reported r27 working well and approved distribution on 2026-10-15.
The tested runtime bytes were promoted unchanged. The r3 third-party report
remains unreproduced; individual fallback cases and other platforms were not
separately confirmed. Exact hashes and log limits are in EVIDENCE.json.

After an authorized manual installation, compare the same stationary hit with a
stationary camera at 1.0× and 1.25×, then rotate the camera and repeat with each style.
Use the matched plugin/loader hashes, an actual launch log and video showing both
the impact and number to establish whether the reporter's displacement persists.
Host tests prove the fallback branches, not the live camera selection.

## Damage Numbers in Rust

`r3` uses Forge 1.5.0 and forge-rs 1.5.0 for the Rust port of Damage Numbers r25. It retains the eight capture sites, ownership and damage rules, duplicate suppression, projection and placement, and Wilds, Rise and Classic presentation. Defaults are enabled, Wilds, size 1.0 and lifetime 0.9 seconds. The unchanged r3 payload is READY_TO_DISTRIBUTE after user acceptance on 2026-10-15. Coverage and log limits are recorded in [runtime evidence](EVIDENCE.json).

[Player guide](README.md), [build instructions](DEVELOPMENT.md), [code walkthrough](DEVELOPMENT.md), [validation scope](DEVELOPMENT.md) and [r3 results](EVIDENCE.json) and [dependency provenance](THIRD_PARTY.md).

The standalone workflow is `./build.sh doctor`, `prepare`, `build`, `test`, `verify`, `package`, and `export --output DEST`. `test --host` omits ARM checks. Preparation, builds and packaging do not install or activate the plugin. The runtime package contains `damage_numbers.nro`, `OpenSans.ttf`, and two external RGBA artwork files. Ordinary builds copy prepared assets; the runtime contains no artwork payloads or compression decoder. Existing C++ source and r25 packages remain preserved.

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches Forge 1.5.0's native ARM32 declaration: 24 bytes, with context and its update cell. Forge shares trampoline storage across hooks. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the recorded GE audit of all eleven sites.

The load callback leaves the loader requirement record untouched, so Forge version checks do not gate startup. The build targets the 1.5.0 native ABI; other loader layouts can be incompatible. Initialization takes no arguments, matching the loader callback. Game-code signatures and instruction checks remain in place. The preserved [r1 results](EVIDENCE.json) describe the older candidate.

The prepared font retains Open Sans Semibold naming. Its glyph outlines, hinting, metrics and weight match r2; only naming metadata/checksums changed. Artwork pixels and atlas geometry are unchanged. The selected styles appear as Wilds, Rise, Classic; stored style IDs remain stable. Missing artwork omits its effects while readable damage numbers continue. Failure messages are deduplicated across context replacements.

The approved r30 release ZIP is archived at `backups/accepted/damage-numbers/r30-before-status-damage/release/damage-numbers-r30.zip`; r31 is a candidate.
Package with `python3 scripts/mod_distribution.py package damage-numbers` and run
the full `check`. The approved r26 delivery is preserved under
`backups/accepted/damage-numbers/r26-before-centre-fallback/`.


## Build and validate

Copy `build.local.example.json` to ignored `build.local.json` and supply the tool paths and copied native inputs. Use Rust nightly-2026-10-02 with rust-src, the pinned cargo-forge 1.4.1 converter built from revision `7f572ed8d0e0a6cfc562571fdbc1ad57441a8438` (retained independently of the 1.5.0 runtime crates), devkitARM GCC 15.2.0, and the Python dependencies recorded in requirements files. The Rust compiler and cargo-forge binary fingerprints are in `dependencies.lock.json`. No installed emulator package is an input.

`inputs` points to a directory with `main`, `sdk`, and the mutex-corrected Forge 1.5.0 NSO named `forge`. Expected hashes are recorded in the lock. `main` must be the MHGU 1.4.0 executable with build ID `FB08F1D20FD1204FE7070672777F869B`. Preserve supplied originals. Configure `arm_python` separately if Unicorn and the visual-test Python packages are installed in different environments.

| Command | Purpose |
| --- | --- |
| `./build.sh doctor` | Validate the compiler, converter and owned dependency fingerprints. |
| `./build.sh prepare` | Validate the prepared assets without generating or compressing data. |
| `./build.sh prepare --fetch` | Explicitly fetch checksum-pinned dependencies when refreshing inputs. |
| `./build.sh build` | Build ARM32 ELF/NRO and stage the NRO, OpenSans.ttf and both RGBA files. |
| `./build.sh test --host` | Run portable behavior, pinned config, production installer, native pooled allocator, C++ parity and real ImGui comparison tests. |
| `./build.sh test` | Add execution of the production ARM gateways and native verification. Requires a completed build. |
| `./build.sh verify` | Check exact native imports, callback exports, game windows, layouts, asset hashes and NZCVQ restore instructions. |
| `./build.sh package` | Require complete tests for the exact NRO, then verify the candidate ZIP and adjacent guides/notices/checksums. |
| `./build.sh export --output DEST` | Export the explicit source allowlist to an unused directory. |

Workspace outputs are under `build/damage-numbers/candidates/r33/` and `work/damage-numbers/candidates/r33/`. Cargo caches, the unstripped ELF and converter output stay under `work/damage-numbers/candidates/r33/cargo-target/`. The pinned converter requires a temporary `target` symlink and loose NRO in the source root while building; the wrapper removes both when the converter exits, including on failure. An independent source export uses its own ignored build/work folders. Registry sources are vendored, including metadata needed by this nightly's build-std resolver. Host tests explicitly load the source-only Cargo configuration so they do not inherit the ARM target or build-std settings. Default builds and tests are offline.

After adding source files, refresh the allowlist with `python3 scripts/export_source.py --refresh`. Exports omit native game inputs, outputs, private configurations, caches and Git metadata. They retain dependency bytes exactly so Cargo checksum validation remains meaningful. Supply your toolchain/input paths in the exported repo's own configuration. No sibling mod import is needed.

The root distribution packager validates the r33 candidate. Run
`python3 scripts/mod_distribution.py package damage-numbers` and the full
`python3 scripts/mod_distribution.py check`. The accepted r26 delivery and tested
r27 candidate remain in `backups/accepted/damage-numbers/` with exact hashes.
Packaging does not install, activate, publish or push.

forge-rs source is pinned at `d1865b68e9b194eb3eacc8bd71bbf601a5fac63e`; the native hook fixture is pinned to Forge `1671ad1526f6df5ee8d6e61a03518e0a427e202e`. The copied loader input has SHA-256 `1e452243cc85108a2271dda99167f9728d051887075766b38e36a1ce94a29b42`. The native fixture compiles the actual allocator with host JIT/TLS/cache doubles, omits ARM barriers, and uses ASan/UBSan. LeakSanitizer is disabled under sandbox tracing; the fixture counts creation and closure of every mock JIT chunk. These checks do not measure emulator allocation or establish gameplay acceptance.

The owned `assets/` folder contains the exact prepared runtime files and their hash/provenance manifest. Ordinary builds need no FontTools or image-generation dependencies. `requirements.txt` supplies Pillow/NumPy only for visual tests. Deleted authoring tools remain recoverable from Git; they are not needed by standalone builds.

`tests/r2-pixels/` holds 24 reference scenes captured from untouched r2 rendering before this refactor, at 720p/1080p and the minimum/maximum scale. Atlas hashes and composited RGB pixels must match exactly. The frozen C++ oracle headers and decoder remain test-only under `tests/oracle/`; no reference artwork or decoder is linked into the production NRO.

External RGBA files are opened through Forge's exposed ImGui APIs, length-checked before texture allocation, read directly into the pixel buffer, and closed on every return. Wilds loads during initialization; Rise loads on its first critical effect. Failed assets are not retried in the same context. Each new context retries loading, while failure logs remain deduplicated for the resource owner's process lifetime. Independent context invalidation prevents stale font handles even when no artwork texture could be registered.

The r3 standalone build, full tests and packaging passed offline with an empty Cargo home. NRO hashes differ across build roots; both exact artifacts passed complete validation. See `docs/STANDALONE_RESULTS.json` for exact fingerprints.


## Code walkthrough

### From a native call to a visible number

`src/lib.rs` declares Forge's entry and rendering callbacks. Startup creates the stock `ThreadLocal<CaptureState>`, then validates the exact game windows and installs all eleven ordinary hooks. Capture becomes active only after the last hook succeeds.

`src/gateway.S` saves a 328-byte stack image before Rust runs: continuation scratch at 0, CPSR/FPSCR at 8/12, d16–d31 at 16, d0–d15 at 144, and r0–r12/LR at 272. Native stack arguments remain above the original SP. Around hooks replace the saved LR only after `CaptureState::enter` records the caller; the shared after gateway pops that scope and resumes its exact caller. Camera, melee HZV, blast and mount-finisher taps observe registers before the displaced instructions execute. `core/src/capture.rs` explains each copied payload and nested scope.

`src/runtime.rs` obtains a short mutable TLS borrow for each helper and releases it before assembly branches to the original trampoline. Nested original calls can therefore access the same thread-local state again without an overlapping Rust borrow. First access may allocate through the stock ThreadLocal helper. Capture then uses fixed storage: 64 scopes, 16 producer mailboxes with 256 records each, and copied numeric payloads.

Mailboxes belong to static plugin storage and are never recycled during the process. Each producer owns one slot; the consumer owns each Cursor. Thread exit drops its CaptureState while queued records remain valid. Atomic payload words and publication tickets reject torn snapshots, and overflow advances to the newest available records. Pool exhaustion passes through without capturing. This matches the r25 limit of 16 producer contexts.

The consumer copies records into `core/src/lib.rs`'s bounded Model: 256 pending events, 128 visible numbers and 512 recent identities. It applies player/time ownership, environmental-source rules, bounce feedback and duplicate suppression. Player or camera changes reset the display epoch. Game identities are used for comparisons only; the rendering code never follows captured object pointers.

`core/src/presentation.rs` projects copied matrices, reserves peak animation bounds, chooses stable placement, and produces DrawNumber values. It contains the three animation styles and r25 size constants. `src/render.rs` measures actual baked glyphs and submits those draw values to ImGui. Effects precede digit fills, preserving r25 ordering, colors, outlines, stars and Rise slash geometry.

### Hook and asset lifetimes

`src/hooks.rs` checks the complete signature windows, instruction relocation whitelist, alignment, bounds and reserved/overlapping redirects before calling Forge. Disabled upstream Patch objects snapshot all originals. Forge creates the eleven ordinary hooks in its shared pool; the installer checks each returned ARM trampoline's displaced words, return jump and executable mapping before publishing its continuation. Capture activates only after every branch and synchronization passes.

Rollback checks each current word against the saved original and this plugin's branch. A pair containing foreign bytes is left untouched. For owned or partially published pairs, the Patch enable/disable operations restore the snapshot, followed by cache synchronization and readback. Snapshots remain disabled outside that restoration, so their destruction cannot overwrite foreign modifications. Valid continuations are retained even after a later failure, and incomplete rollback requires restart. There is no live installation, uninstall or trampoline release. The Forge pool owns all trampoline memory until process exit; retaining returned Hook structs is unnecessary.

Eleven ordinary ARM hooks use 16 bytes each, totaling 176 bytes of slots in Forge's shared pool of at most two 16 KiB chunks. The host fixture confirms sharing and failure behavior against the pinned native allocator. Actual emulator pool use remains a runtime check. The load callback is a no-op, leaving the loader's requirement record zero; initialization uses the loader's zero-argument ABI.

Font loading uses `ImFont::load_with` from RomFS OpenSans.ttf. Its configuration allows load errors to return a fallback instead of asserting. The atlas owns the loaded file data. Each file-loaded RGBA texture is registered with ImGui and has a shutdown callback to unregister and destroy it. Context epochs prevent initialization from keeping pointers after shutdown, including when an allocator reuses a context address. Repeated initialization of a live context does not load duplicate assets.

### Settings

`src/config.rs` derives upstream IniSerialize/IniDeserialize. Individual field wrappers define true, 1.0 and 0.9 defaults, and reject invalid or nonfinite values; Style defaults to Wilds. The r31 `other` key is a plain bool, so a missing or invalid value means off. Once save data and MtFile are available, stock `config::load` reads the file or creates it with defaults. Save uses stock ScopedMount and Writer. Reset updates session settings; Save persists them. These are the explicitly requested forge-rs saving semantics, superseding the older custom persistence rules for this mod.

The unframed settings tree is closed by default. Its guard balances TreePop on every opened path. Free-render processing runs independently of whether that tree is open.

### Unsafe boundaries

Native address access checks mappings and copies values into owned numeric structures. FFI and assembly requirements are documented beside each unsafe block. Compiler-derived layout data stays in the NRO for verification against the pinned native headers. Raw ImGui operations require an active context, current fonts/draw lists and registered texture lifetime. The mailbox representation contains no padding, references, booleans or Rust enums: every copied word has a valid integer/float representation.

### Prepared assets and pixel parity

Only the numeric font scale and atlas rectangles remain in `src/assets.rs`. Font and bitmap bytes live in the package's external asset files. File sizes are checked before allocation; the exposed ImGui file APIs read directly into the owned texture buffer, and the local file guard closes every successful open. Short reads destroy the incomplete texture. Wilds loads eagerly and Rise once on first use. Missing textures omit their effects without disabling the model or digits.

A separate context-shutdown callback advances the generation even if no texture loaded. Texture callbacks only unregister/free their own allocations. This makes font/resource invalidation independent of successful artwork loading and prevents stale handles if a context address is reused. Warning flags survive reinitialization, while resource pointers and load attempts reset.

Ordinary text, image, line and clip operations use forge-rs DrawList wrappers. Raw access remains for glyph ink bounds, Rise's exact vertex stretch, texture registration and quad drawing. No animation/layout formula changed. Style selection is Wilds, Rise, Classic, with the existing internal numeric IDs retained for compatibility.


## Validation scope

Host tests exercise copied-value capture nesting, scope overflow/quarantine, ownership/source policy, bounce association, bounded model/mailbox overflow, duplicate suppression, disabled settings, concurrent publication and queued records surviving thread exit.

The config harness compiles the pinned upstream config/INI modules and derives, plus production src/config.rs, against test MtFile and counted-mount I/O. It checks automatic default-file creation, individual defaults and invalid parsing, all style round trips, Save and Reset.

The hook harness compiles production src/hooks.rs and the stock Patch wrapper against a native API double. It rejects invalid recipes, reserved/overlapping spans and changed game signatures; injects failures before, during and after branch publication at all eleven sites; checks invalid/non-executable trampoline and returned-record evidence; and exercises cache, restoration and foreign-change failures. Owned changes must roll back and capture must remain disabled on failure. Foreign changes remain untouched, valid continuations remain published and incomplete restoration requires restart. Success activates all eleven.

The native pool fixture compiles pinned Forge hook.c with JIT/TLS/cache doubles and omitted ARM barriers. Eleven ordinary hooks share one chunk and use 176 bytes of slots. Allocation failure, rejected fallback and full two-chunk capacity preserve target bytes. ASan/UBSan run; mocked JIT allocations are counted and closed because LeakSanitizer cannot run under sandbox tracing. This is host evidence, not measured emulator allocation.

The C++ r25 oracle comparison covers 243 combinations of style, critical outcome, HZV and age, producing 1,944 draw records. Exact identities/colors/text and bounded float differences are checked. Another 2,800 cases compare source/owner policy, finalized damage/critical outcomes and native matrix projection against r25, including invalid matrices, perspective, translation and clipped points. The real renderer harness compiles production src/render.rs with pinned Forge font/draw wrappers and actual ImGui 1.92.8. All three styles are compared against 24 frozen r2 scenes at 720p/1080p and minimum/maximum scales, including textures, glyph atlas bytes, draw commands, indices, vertices and exact composited pixels. The unchanged C++ oracle comparison retains its historical tolerance. Repeated initialization is checked against duplicate resource creation. File-operation doubles inject short reads and count handles; missing/wrong-size assets, font fallback, context replacement and shutdown cleanup are exercised.

The ARM fixture executes all twelve gateways from the production NRO in Unicorn 2.1.4. Its helper doubles deliberately clobber registers, floating-point state and flags; it checks all GPRs, d0–d31, FPSCR, NZCVQ, LR/SP, native stack arguments, around-return paths, and all 16 GE patterns with preserved and changed GE. Helpers are doubles; this is not a native gameplay run. The GE liveness audit (eight original sites, plus the r31 sites above) applies to the pinned game.

`verify` additionally compares compiler-derived Rust/C++ ARM32 Hook size/alignment/pointer offsets, saved frame, ImGui/font/texture/draw layouts; checks callback exports, all strong imports against actual main/SDK/Forge exports, exact game signatures, asset receipt hashes and twelve NZCVQ-only restoration instructions.

Gameplay, config persistence across a real restart, loader allocation limits and emulator lifecycle remain manual checks. No installation, activation or game launch is performed by these commands.


## GE preservation audit

Checked 2026-10-06 for the eight current MHGU 1.4.0 capture hooks and the
released r16 plugin. GE restoration is unnecessary for these hook locations.
The gateway's general transparent-state contract originally included GE, but
the captured game computations do not consume it.

The bounded compatibility change is `msr APSR_nzcvqg, r2` to
`msr APSR_nzcvq, r2` in the shared RESTORE macro. This preserves NZCVQ and
leaves GE untouched by that instruction. It does not zero GE. This audit did
not change the production source, release package or installed plugin.

### Binary identity

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

### Original game instructions

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

### Capture helper instructions

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

### Scratch execution checks

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

### R17 implementation — 2026-10-06

The audited change is now applied in the production `gateway.S` restore macro.
The original r16/scratch evidence above remains a historical record.
[R17 compatibility notes](DEVELOPMENT.md) document the reduced
contract, production binary comparison, fresh fixtures and user-confirmed
Eden/Ryujinx gameplay. The stale entries-6/7 comment is corrected; all eight entries
remain production hooks. No hook location or captured frame layout changes.

The original Ryujinx-Nextendo translator at reference commit
`05577ab543f4322cb7534c1f5caca0e0079791bf` handles MSR mask 8 (NZCVQ) and throws
`APSR_g` for mask 4 (GE). The proposed instruction therefore avoids this
specific unsupported translation path. Its MRS also omits GE, but the capture
helpers do not consume the saved GE bits.

### Limits

This result applies to the recorded game build, r16 helper binary and current
eight hook sites. Recheck new hooks, especially a tap between GE-setting
parallel arithmetic and a `SEL`, or code that reads GE through `MRS`.
A generic transparent gateway should retain GE preservation when such
liveness has not been ruled out. Gameplay compatibility of a changed NRO
still requires a user-run emulator test.

R18 revalidation (2026-10-07): gateway source is byte-identical to the audited r17 assembly. All nine emitted NZCVQ restores and all original/compiled gateway state fixtures passed. No hook site or continuation changed; r18 carries this liveness conclusion. This is static/compiled evidence without a fresh gameplay run.
