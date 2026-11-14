# Damage Numbers: development and research

**Current state.** r47 DEVELOPMENT, candidate at `build/damage-numbers/candidates/r47/delivery/`, not run. It removes Show other damage, adds Spread, and puts the settings on a Purple-lib Mod Options page with help in seven languages and a live preview, anchored in the world in quests and flat on screen in the village. TESTING.md lists the run that would accept it. The last gameplay-accepted build is r42 (`build/damage-numbers/candidates/r42/`, release ZIP and SHA256SUMS there). DISTRIBUTION.json is authoritative for status and hashes; EVIDENCE.json holds every version's runs.

What the current build does at its compatibility edges:

- It zeroes the projection's m[8]/m[9] before projecting, where MHGURecomp's FSR writes its per-frame sub-pixel jitter (its `runtime/camera_jitter.c`: "zero as made").
- When the picture's shape differs from Forge's display (gu-native draws the overlay over its whole window), the projection's shape places the picture, letterboxed or stretched, inside the display (see *Projection and centre fallback*).
- When ImGui draws on a host surface (MHGURecomp), the visible crop is ignored (see *Visible crop*); otherwise a lowered render resolution draws numbers at about half size toward the top left.
- No plugin code uses i32 saturating arithmetic, which emits QADD/QSUB that stop the game on Ryubing 1.3.3; `verify.py` rejects every saturating instruction Ryubing lacks.
- Entry hooks accept another plugin's Forge hook on the same function (see *Hook sites*).
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

NRO hashes differ across build roots. The r3 standalone fingerprints are in EVIDENCE.json under `historical` → `docs/STANDALONE_RESULTS.json`.

## Code walkthrough

### From a native call to a visible number

`src/lib.rs` declares Forge's entry and rendering callbacks. Startup creates the stock `ThreadLocal<CaptureState>`, then validates the exact game windows and installs all eleven ordinary hooks. Capture becomes active only after the last hook succeeds.

`src/gateway.S` saves a 328-byte stack image before Rust runs: continuation scratch at 0, CPSR/FPSCR at 8/12, d16–d31 at 16, d0–d15 at 144, and r0–r12/LR at 272. Native stack arguments remain above the original SP. Around hooks replace the saved LR only after `CaptureState::enter` records the caller; the shared after gateway pops that scope and resumes its exact caller. Camera, melee HZV, blast and mount-finisher taps observe registers before the displaced instructions execute. `core/src/capture.rs` explains each copied payload and nested scope.

`src/runtime.rs` obtains a short mutable TLS borrow for each helper and releases it before assembly branches to the original trampoline. Nested original calls can therefore access the same thread-local state again without an overlapping Rust borrow. First access may allocate through the stock ThreadLocal helper. Capture then uses fixed storage: 64 scopes, 16 producer mailboxes with 256 records each, and copied numeric payloads.

Mailboxes belong to static plugin storage and are never recycled during the process. Each producer owns one slot; the consumer owns each Cursor. Thread exit drops its CaptureState while queued records remain valid. Atomic payload words and publication tickets reject torn snapshots, and overflow advances to the newest available records. Pool exhaustion passes through without capturing. This matches the r25 limit of 16 producer contexts.

The consumer copies records into `core/src/lib.rs`'s bounded Model: 256 pending events, 128 visible numbers and 512 recent identities. It applies player/time ownership, environmental-source rules, bounce feedback and duplicate suppression. Player or camera changes reset the display epoch. Game identities are used for comparisons only; the rendering code never follows captured object pointers.

`core/src/presentation.rs` projects copied matrices, reserves peak animation bounds, chooses stable placement, and produces DrawNumber values. It contains the three animation styles and r25 size constants. `src/render.rs` measures actual baked glyphs and submits those draw values to ImGui. Effects precede digit fills, preserving r25 ordering, colors, outlines, stars and Rise slash geometry.

### Forge ownership and ABI

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches Forge's native ARM32 declaration: 24 bytes, with context and its update cell. Forge shares trampoline storage across hooks. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the GE audit below.

`forge_onLoad` writes the 70-byte native metadata record: requested version 1.5.1 and the name `Damage Numbers rNN` at offset 6 (`src/lib.rs`; `tests/gateways.py` checks the label). Forge compares only major/minor, so this is not a strict loader-equality guard. Initialization takes no arguments, matching the loader callback. Game-code signatures and instruction checks remain in place.

### Hook and asset lifetimes

`src/hooks.rs` checks the complete signature windows, instruction relocation whitelist, alignment, bounds and reserved/overlapping redirects before calling Forge. Disabled upstream Patch objects snapshot all originals. Forge creates the eleven ordinary hooks in its shared pool; the installer checks each returned ARM trampoline's displaced words, return jump and executable mapping before publishing its continuation. Capture activates only after every branch and synchronization passes.

Rollback checks each current word against the saved original and this plugin's branch. A pair containing foreign bytes is left untouched. For owned or partially published pairs, the Patch enable/disable operations restore the snapshot, followed by cache synchronization and readback. Snapshots remain disabled outside that restoration, so their destruction cannot overwrite foreign modifications. Valid continuations are retained even after a later failure, and incomplete rollback requires restart. There is no live installation, uninstall or trampoline release. The Forge pool owns all trampoline memory until process exit; retaining returned Hook structs is unnecessary.

Eleven ordinary ARM hooks use 16 bytes each, totaling 176 bytes of slots in Forge's shared pool of at most two 16 KiB chunks. The host fixture confirms sharing and failure behavior against the pinned native allocator. Actual emulator pool use remains a runtime check.

Font loading uses `ImFont::load_with` from RomFS OpenSans.ttf. Its configuration allows load errors to return a fallback instead of asserting. The atlas owns the loaded file data. Each file-loaded RGBA texture is registered with ImGui and has a shutdown callback to unregister and destroy it. Context epochs prevent initialization from keeping pointers after shutdown, including when an allocator reuses a context address. Repeated initialization of a live context does not load duplicate assets.

### Settings

`src/config.rs` derives upstream IniSerialize/IniDeserialize. Individual field wrappers define true, 1.0 and 0.9 defaults, and reject invalid or nonfinite values; Style defaults to Wilds. Spread (INI key `spread`, 0.0–2.0, default 0.8) multiplies the 17 placement offsets in `presentation::layout`; at 1.0 the products equal r42's unscaled offsets exactly, so the reference comparisons pin it to 1.0. An r42 INI has no `spread` key and loads with 0.8; its `other` key is ignored, because the derive reads only declared fields. The preview mode and random hit types are session-only `Settings` fields, never written to the INI, and have no ImGui widgets. Once save data and MtFile are available, stock `config::load` reads the file or creates it with defaults. Save uses stock ScopedMount and Writer. Reset updates session settings; Save persists them. The live settings are `runtime::SETTINGS` (`SharedSettings` in the core: atomics, no lock), because Purple-lib's get/set run in the game's update and may not lock. Every writer stores there; `consume` copies a changed value into the Model each frame, and `EDITED` stops a late INI load replacing a user edit. These are the explicitly requested forge-rs saving semantics, superseding the older custom persistence rules for this mod.

The unframed settings tree is closed by default. Its guard balances TreePop on every opened path. Free-render processing runs independently of whether that tree is open.

### Mod Options (Purple-lib)

Optional: Purple-lib r17 or later calls the exported `purplelib_on_register` (`link/exported.txt`) once, from its first update, and `src/purple_lib.rs` registers one page of eight rows plus seven page-help topics (`add_set_help`, one per setting row). Without Purple-lib nothing calls it and the Plugin UI panel and INI work alone. Records, row texts, row shapes and the value mapping are in `core/src/purple_lib.rs`, where host tests check them; the workspace check compares the records with Purple-lib's own.

- Rows: Show damage numbers and Random hit types use the game's Off/On (StartMenuMsg 175/176); Style is Wilds, Rise, Classic (row values 0..2, not `Style`'s numbering); Number size is 50%..300% and Spread 0%..200%, numbers rows in steps of 10; Duration is a text row, 0.4 s..2.0 s, with comma decimals for French, Spanish, German and Italian and 秒 for Chinese; Preview mode is Single, Burst, Multi; Restore defaults is an action row. A step value divided by 10 gives the decimal literal's f32, so the defaults compare equal; `get` shows the nearest step for a slider value between steps.
- `get`/`set` run in the game's update and touch only `runtime::SETTINGS`. A saved row (0..4) marks the page dirty; `closed` turns that into `purple_lib::SAVE`, and the render thread saves the INI on its next frame. Preview rows never save. Restore defaults sets `SAVE` itself, because Purple-lib calls `closed` only after a `set`.
- Text: English, French, Spanish, German, Italian, Traditional and Simplified Chinese, from the game's language byte; anything else is English. Help pages hold five lines of 40 columns, help-bar lines two, and values 11 columns, a CJK character counting two. The translations are agent-drafted and no native speaker has reviewed them.

### Mod Options preview

While `purple_lib::shown()` (Purple-lib's `options_shown` for this page) is true, `runtime::render` draws a preview after the real numbers; otherwise it stops it. `core/src/preview.rs` holds the logic and `PreviewState`, which has its own `Model` and `Placement`, so preview hits never mix with real ones. The production `layout` and `Resources::draw` draw them, so style, size, duration and spread apply as for real hits, and Show damage numbers off does not hide the preview.

- Frame: fractions of the visible view (left 2.5%, 57% wide, 38% tall, vertically centred), so it ends before the Options window. `Resources::preview_frame` draws a light border over a dark one, a filled label tab "Damage number preview" in English in Forge's default font, and a dot per spot; all sizes scale with view height / 720.
- Spots at 1/6, 1/2 and 5/6 of the frame's width. When the page opens, a spot becomes the world point at clip depth `preview::DEPTH` (600, unverified world units) that projects to it, and keeps it across loops, so moving or turning the camera moves the spots and their hits like real ones. They are anchored in the frame again when one leaves the screen. The preview uses the same live camera as real numbers (a sample under 0.2 s old); the capture accepts only `uQuestCamera` (main+0x1730d0c), so in the village the preview uses flat screen positions. Without a live camera the spots are anchored in a flat identity view and stay put; a camera appearing or vanishing anchors them again. A diagnostic logs which kind of anchor the spots got, at most four times a run, until the anchoring is accepted.
- Hidden while the page's own help is open, when Purple-lib provides API version 4 (`help_shown`, r19 and later). The table is read field by field through a raw pointer, because a version 3 table is 28 bytes and ends before that entry.
- Loop: Single one hit per spot at 0 s, Burst five at 0.15 s intervals, Multi five at once; the loop lasts the last spawn plus the duration plus 0.5 s, and a mode change starts a new one. Normal types: left weak point (hitzone 60) with positive critical, middle hitzone 20 with negative critical, right a bounce. Random hit types pick weak/normal/bounce and the critical evenly. Damage is 1..150 (xorshift32).

### Unsafe boundaries

Native address access checks mappings and copies values into owned numeric structures. FFI and assembly requirements are documented beside each unsafe block. Compiler-derived layout data stays in the NRO for verification against the pinned native headers. Raw ImGui operations require an active context, current fonts/draw lists and registered texture lifetime. The mailbox representation contains no padding, references, booleans or Rust enums: every copied word has a valid integer/float representation.

### Prepared assets and pixel parity

Only the numeric font scale and atlas rectangles remain in `src/assets.rs`. Font and bitmap bytes live in the package's external asset files. File sizes are checked before allocation; the exposed ImGui file APIs read directly into the owned texture buffer, and the local file guard closes every successful open. Short reads destroy the incomplete texture. Wilds loads eagerly and Rise once on its first critical effect. Failed assets are not retried in the same context; each new context retries. Missing textures omit their effects without disabling the model or digits.

A separate context-shutdown callback advances the generation even if no texture loaded. Texture callbacks only unregister/free their own allocations. This makes font/resource invalidation independent of successful artwork loading and prevents stale handles if a context address is reused. Warning flags survive reinitialization for the process lifetime, while resource pointers and load attempts reset.

The prepared font retains Open Sans Semibold naming. Its glyph outlines, hinting, metrics and weight match r2; only naming metadata/checksums changed. Artwork pixels and atlas geometry are unchanged.

Ordinary text, image, line and clip operations use forge-rs DrawList wrappers. Raw access remains for glyph ink bounds, Rise's exact vertex stretch, texture registration and quad drawing. Style selection is Wilds, Rise, Classic, with the existing internal numeric IDs retained for compatibility.

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

### Hook sites

| Hook | Main offset | Location | Enclosing function bytes |
| --- | --- | --- | --- |
| 0 | `0x170968` | Collision function entry, around | 15,868 |
| 1 | `0x9d6f8` | Damage function entry, around | 432 |
| 2 | `0x2a87a8` | Feedback function entry, around | 680 |
| 3 | `0xb8909c` | Viewport function entry, around | 2,128 |
| 4 | `0x885844` | Camera function entry, before-only | 4,424 |
| 5 | `0x16dd7c` | Melee hitzone function tail, before-only | 284 |
| 6 | `0x16dc6c` | Melee hitzone function entry, around | 284 |
| 7 | `0x178660` | Shot hitzone function entry, around | 500 |
| 8 | `0xa2664` | Poison tick entry, around | 240 |
| 9 | `0xaac54` | Blast detonation entry, before-only | 332 |
| 10 | `0x9978c` | Mount finisher, interior before-only (inside 0x97ddc) | 13,472 |

Hooks 8–10 displace `push {r4-r7,r11,lr}` / `vpush {d8}`, `push {r4-r8,lr}` /
`sub sp,sp,#8`, and `ldr r7,[sp,#60]` / `ldr r0,[r7]`. The relocation whitelist
adds only the two loads at hook 10; neither reads PC. A scan of all ARM branches
found no branch to 0x99790, the second displaced word. 0x9978c is reached by
fall-through and from the `blt` at 0x99770, where r4 already holds the unscaled
amount. The poison function's tail branch at 0xa2740 returns through the
substituted LR, like the collision function. No workspace mod patches these
offsets, and none is in the reserved list. The gateway source hash in
`dependencies.lock.json` covers the three ENTRY lines for hooks 8–10.

Hooks 0-4 and 6-9 run at a function entry and only observe around the original, so their windows accept
another plugin's Forge hook over the first eight bytes (see "Chaining" in the forge-modding hooks reference); supporting
windows do too, since they are only compared. `window_matches` in `src/hooks.rs` is the check. Hook 5 (melee
hitzone tail) and hook 10 (mount finisher) sit inside a function and stay exact. The around hooks keep the
caller and SP only to return through, and classify nothing by caller, so a foreign hook above or below them
changes no capture.

MHGU DPS Meter v1.2.0 holds the constants of hooks 0 and 1 and supporting window `0x276278`. Its binary says it chains onto an
already patched site after the first update, so it lies over this mod's branch rather than meeting the load check.
On MHGURecomp it logged `main+0x00170968 already patched (0xE51FF004); chaining`, the second word being this mod's gateway, then `chained hooks installed after first update`, so it stacks on these hooks there.

On MHGURecomp, `DrawList` is a guest mirror of a host object, and `VtxBuffer.Data` (guest +32, `MF_DATA` in its `forge_imgui_thunks.inc`) points at host memory with no guest address, so it reads 0 while `Size` is real. The vertex edits in `Renderer::number` then land on the null page (`null page` warnings at +0x475C..+0x490C) instead of the vertices. Wilds outline copies stay under the fill, which the user confirmed in play, Rise loses its outline and width stretch by the same reading, and Classic is unaffected because it passes offsets to the text call. Not fixed. Writing through `VtxBuffer.Data` works on Forge only.

A twelfth signature window checks the crop call and applied-field stores (below)
before any hook is installed.

### How monster HP falls, and damage outside ordinary hits

The native routes (part counters, the pending amount, the collision scope, kinsect, poison, blast, mount, map-fired shells, ballista, siege weapons) are in `.agents/skills/mhgu-understanding/references/damage-and-cameras.md`, sections How monster HP actually falls and Shells and siege weapons. Each route here has its own hook: poison is hook 8, blast hook 9, the mount finisher hook 10; kinsect, map-fired shells and ballista arrive through the collision path. The mount finisher label is an inference; if gameplay shows a different event, only the label changes, the amount is still real HP damage. Hits with unrecognised tags stay hidden.

Status and stage records carry no owner and are shown for every player, like
monster and object damage. Status positions are the monster root, not an
impact point. Poison numbers are gray; blast and mount use the weak-point colour
because each comes from its own native subtraction, not HP watching.

### What capture publishes

Capture publishes every finalized hit amount, not only shown families; the
consumer's visibility rules decide what appears, and bounce association uses only
the local/ownerless subset. The `Source` byte in each record is fixed (Poison 6,
Blast 7) because `runtime.rs` decodes those values.

### Visible crop

MHGU can display a smaller crop of an unchanged swapchain allocation. Forge's
ImGui DisplaySize describes that allocation, so it can report 1920×1080 bounds for a
visible 1280×720 region. The user reported correct docked placement and displaced
handheld placement on Switch and Android against an older build. Exact live
dimensions and this correction remain unconfirmed on those devices.

The render callback obtains the actual sFestaRender singleton through Forge's
exact-type ID lookup (0x300a6cd8). The sRender base ID is not registered for that
object. Checked copies of inherited mDisplaySize at +0xb8/+0xbc provide the
applied crop width/height in backing-texture pixels. Native main+0xbc0874 calls
main+0x86a24c to SetCrop(0,0,mScreenSize), updates that pair, then presents. The
fields at +0xb0/+0xb4 are requested dimensions and may still be pending. Saved
object-kind DTI and original-byte research agree on these offsets. See
research/visible-crop.json for the native contract and provenance.

The size is re-read each render callback. It must be positive and fit inside
Forge's finite backing dimensions. The same size reaches camera projection,
number/effect scaling, placement bounds, clipping and screen-centre fallback.
Layout's viewport comparison discards cached offsets after a size change. The
native matrix multiplication and camera-admission policy are unchanged;
same-aspect rendering at another internal resolution remains supported. No
hardcoded 720p/1080p multiplier is used.

On MHGURecomp, ImGui draws on a host surface scaled over the whole window, and
+0xb8/+0xbc holds the internal render size, so `platform::visible_size` returns
Forge's display size there. It recognises Recomp by the guest DrawList mirror:
`CmdBuffer` always holds a command after NewFrame, but its `Data` has no guest
address and reads 0 (see the `VtxBuffer` note under *Hook sites*).

An unavailable/invalid crop retains the previous Forge bounds to keep damage
visible and emits one lifetime-deduplicated warning once a player exists. That
fallback can retain the original displacement. No native pointer is cached or
dereferenced by deferred rendering.

### Projection and centre fallback

A failed impact-position copy marks the position unavailable while keeping the
collision scope and native return chain intact; the consumer represents it with
`None`. A genuine captured world origin remains valid. Missing positions and
missing/stale/invalid cameras use the display centre, with normal animation and
overlap spacing. Since r36, a position that a usable (finite, non-zero) camera
clips, including hits behind the camera, is skipped for that frame only. Layout
re-projects every live number each frame, so the hit appears at its world
position if the camera turns to it before its lifetime ends; its placement
offset is chosen when it first becomes visible. Invalid display dimensions
still prevent drawing.

The viewport guard checks origin and aspect ratio, not presentation-target
ownership. A same-aspect offscreen view can still pass. Requiring equal native
and display dimensions would reject legitimate resolution scaling, so the guard
stays. When the viewport's shape differs from the display's (more than 2%), the
projection decides: the perspective builder main+0x7c617c stores
`cot(fov/2)/aspect` at `[0]` and `cot(fov/2)` at `[5]`, so `[5]/[0]` is the
picture's shape. It must match the viewport (bars around the picture) or the
display (a window resized after the 3D picture was made), and the view is the
largest rectangle of that shape that fits, centred. The projection is used with m[8] and m[9] set to 0:
the game's perspective leaves them 0, and a temporal upscaler's jitter there made the numbers shake. gu-native (an Android and
Windows port with its own Forge) needs both: its overlay covers the whole
window, and its picture is fitted inside with the screen's shape
(`device.c` `backend_present`, source only). Read-only native exports and original-byte comparisons are in
`work/damage-numbers/projection-r27/native/`. They do not identify the r3
third-party reporter's camera; that report remains unreproduced.

To investigate a displacement report after an authorized install, compare the
same stationary hit with a stationary camera at 1.0× and 1.25×, then rotate the
camera and repeat with each style. Use the matched plugin/loader hashes, an
actual launch log and video showing both the impact and number.

## Validation scope

Host tests exercise copied-value capture nesting, scope overflow/quarantine, ownership/source policy (including kinsect, stage-owned shells, poison/blast/mount records and remote-hit publication without bounce association), bounded model/mailbox overflow, duplicate suppression, the monster list and HP bookkeeping, disabled settings, concurrent publication and queued records surviving thread exit. Failed/non-finite position reads inside nested scopes, valid world origin, matrix order, 720p/900p/1080p/1440p mapping, crop reads, invalid dimensions/mappings, pointer overflow, applied versus requested size, repeated mode changes and centre fallback for all three styles are covered. `tests/viewport.py` executes the original ARM SetCrop wrapper and applied-size stores with an NVN double.

The config harness compiles the pinned upstream config/INI modules and derives, plus production src/config.rs, against test MtFile and counted-mount I/O. It checks automatic default-file creation, individual defaults and invalid parsing, all style round trips, spread limits, an r42 file with `other` and no `spread`, preview fields never saved, Save and Reset.

The hook harness compiles production src/hooks.rs and the stock Patch wrapper against a native API double. It rejects invalid recipes, reserved/overlapping spans and changed game signatures; injects failures before, during and after branch publication at all eleven sites; checks invalid/non-executable trampoline and returned-record evidence; and exercises cache, restoration and foreign-change failures. Owned changes must roll back and capture must remain disabled on failure. Foreign changes remain untouched, valid continuations remain published and incomplete restoration requires restart. Success activates all eleven.

The native pool fixture compiles pinned Forge hook.c with JIT/TLS/cache doubles and omitted ARM barriers. Eleven ordinary hooks share one chunk and use 176 bytes of slots. Allocation failure, rejected fallback and full two-chunk capacity preserve target bytes. ASan/UBSan run; mocked JIT allocations are counted and closed because LeakSanitizer cannot run under sandbox tracing. This is host evidence, not measured emulator allocation.

The C++ r25 oracle comparison covers 243 combinations of style, critical outcome, HZV and age, producing 1,944 draw records. Exact identities/colors/text and bounded float differences are checked. Another 2,800 cases compare source/owner policy, finalized damage/critical outcomes and native matrix projection against r25, including invalid matrices, perspective, translation and clipped points. The r25 oracle covers primary kinds 0–4 only, so the kinsect rule does not alter it. The real renderer harness compiles production src/render.rs with pinned Forge font/draw wrappers and actual ImGui 1.92.8. All three styles are compared against 24 frozen r2 scenes at 720p/1080p and minimum/maximum scales, including textures, glyph atlas bytes, draw commands, indices, vertices and exact composited pixels. Repeated initialization is checked against duplicate resource creation. File-operation doubles inject short reads and count handles; missing/wrong-size assets, font fallback, context replacement and shutdown cleanup are exercised.

The ARM fixture executes all twelve gateways from the production NRO in Unicorn 2.1.4. Its helper doubles deliberately clobber registers, floating-point state and flags; it checks all GPRs, d0–d31, FPSCR, NZCVQ, LR/SP, native stack arguments, around-return paths, and all 16 GE patterns with preserved and changed GE. Helpers are doubles; this is not a native gameplay run.

`verify` additionally compares compiler-derived Rust/C++ ARM32 Hook size/alignment/pointer offsets, saved frame, ImGui/font/texture/draw layouts; checks callback exports, all strong imports against actual main/SDK/Forge exports (including `igSetItemTooltip`), exact game signatures, asset receipt hashes and twelve NZCVQ-only restoration instructions.

Gameplay, config persistence across a real restart, loader allocation limits, GPU output and emulator lifecycle remain manual checks. No installation, activation or game launch is performed by these commands. Exact counts for each candidate are in EVIDENCE.json.

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

## Do not

- **Replace hooks 3 and 4 with main+0x18a48 from `forge_onUpdate`.** The A/B gate failed: 6 of 64 hits within 2 px, median 7.2 px, max 102.4 px, growing with camera motion. See `work/damage-numbers/projection-call/RESEARCH.md`. The user closed this idea on 2026-10-30.
- **Move an off-screen hit to screen centre.** A position a usable camera clips stays hidden until the camera turns to it. Centre is only for missing positions and missing or invalid cameras.
- **Require native and display dimensions to match in the viewport guard.** That rejects legitimate resolution scaling.
- **Log per frame or above budget in a diagnostic.** Stock Forge filters below ERROR and its message buffer is 512 bytes, so cap lines per process and keep them short. A hit cap that ends the run early loses the later segments.
- **Treat the handheld placement report as fixed.** It was against r28, on real Switch and Android. No affected device has run r29-diagnostics or later. Ask for matched plugin and loader hashes, a launch log and a video.
- **Use `i32::saturating_add`/`saturating_sub` in plugin code.** They emit QADD/QSUB, which stop the game on Ryubing 1.3.3; `-dsp` does not remove them in ARM mode.
- **Expect an NRO hash to match across build roots.** Hashes differ by root.
- **Accept `uVillageCamera` (main+0x1730e10) in the camera capture.** Its samples reached the mod in r46 but lagged the view (user report on MHGURecomp); the cause is unknown, so the village preview stays flat.
