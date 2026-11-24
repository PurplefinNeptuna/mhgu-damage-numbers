# Damage Numbers: capturing hits

Worked example from `mods/damage-numbers/`. Game facts it rests on are in `mhgu-understanding`.

## From a native call to a visible number

`src/lib.rs` declares Forge's entry and rendering callbacks. Startup creates the stock `ThreadLocal<CaptureState>`, then validates the exact game windows and installs all eleven ordinary hooks. Capture becomes active only after the last hook succeeds.

`src/gateway.S` saves a 328-byte stack image before Rust runs: continuation scratch at 0, CPSR/FPSCR at 8/12, d16–d31 at 16, d0–d15 at 144, and r0–r12/LR at 272. Native stack arguments remain above the original SP. Around hooks replace the saved LR only after `CaptureState::enter` records the caller; the shared after gateway pops that scope and resumes its exact caller. Camera, melee HZV, blast and mount-finisher taps observe registers before the displaced instructions execute. `core/src/capture.rs` explains each copied payload and nested scope.

`src/runtime.rs` obtains a short mutable TLS borrow for each helper and releases it before assembly branches to the original trampoline. Nested original calls can therefore access the same thread-local state again without an overlapping Rust borrow. First access may allocate through the stock ThreadLocal helper. Capture then uses fixed storage: 64 scopes, 16 producer mailboxes with 256 records each, and copied numeric payloads.

Mailboxes belong to static plugin storage and are never recycled during the process. Each producer owns one slot; the consumer owns each Cursor. Thread exit drops its CaptureState while queued records remain valid. Atomic payload words and publication tickets reject torn snapshots, and overflow advances to the newest available records. Pool exhaustion passes through without capturing. This matches the r25 limit of 16 producer contexts.

The consumer copies records into `core/src/lib.rs`'s bounded Model: 256 pending events, 128 visible numbers and 512 recent identities. It applies player/time ownership, environmental-source rules, bounce feedback and duplicate suppression. Player or camera changes reset the display epoch. Game identities are used for comparisons only; the rendering code never follows captured object pointers.

`core/src/presentation.rs` projects copied matrices, reserves peak animation bounds, chooses stable placement, and produces DrawNumber values. It contains the three animation styles and r25 size constants. `src/render.rs` measures actual baked glyphs and submits those draw values to ImGui. Effects precede digit fills, preserving r25 ordering, colors, outlines, stars and Rise slash geometry.

## Forge ownership and ABI

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches Forge's native ARM32 declaration: 24 bytes, with context and its update cell. Forge shares trampoline storage across hooks. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the GE audit below.

`forge_onLoad` writes the 70-byte native metadata record: requested version 1.5.1 and the name `Damage Numbers rNN` at offset 6 (`src/lib.rs`; `tests/gateways.py` checks the label). Forge compares only major/minor, so this is not a strict loader-equality guard. Initialization takes no arguments, matching the loader callback. Game-code signatures and instruction checks remain in place.

## Hook and asset lifetimes

`src/hooks.rs` checks the complete signature windows, instruction relocation whitelist, alignment, bounds and reserved/overlapping redirects before calling Forge. Disabled upstream Patch objects snapshot all originals. Forge creates the eleven ordinary hooks in its shared pool; the installer checks each returned ARM trampoline's displaced words, return jump and executable mapping before publishing its continuation. Capture activates only after every branch and synchronization passes.

Rollback checks each current word against the saved original and this plugin's branch. A pair containing foreign bytes is left untouched. For owned or partially published pairs, the Patch enable/disable operations restore the snapshot, followed by cache synchronization and readback. Snapshots remain disabled outside that restoration, so their destruction cannot overwrite foreign modifications. Valid continuations are retained even after a later failure, and incomplete rollback requires restart. There is no live installation, uninstall or trampoline release. The Forge pool owns all trampoline memory until process exit; retaining returned Hook structs is unnecessary.

Eleven ordinary ARM hooks use 16 bytes each, totaling 176 bytes of slots in Forge's shared pool of at most two 16 KiB chunks. The host fixture confirms sharing and failure behavior against the pinned native allocator. Actual emulator pool use remains a runtime check.

Font loading uses `ImFont::load_with` from RomFS OpenSans.ttf. Its configuration allows load errors to return a fallback instead of asserting. The atlas owns the loaded file data. Each file-loaded RGBA texture is registered with ImGui and has a shutdown callback to unregister and destroy it. Context epochs prevent initialization from keeping pointers after shutdown, including when an allocator reuses a context address. Repeated initialization of a live context does not load duplicate assets.

## Unsafe boundaries

Native address access checks mappings and copies values into owned numeric structures. FFI and assembly requirements are documented beside each unsafe block. Compiler-derived layout data stays in the NRO for verification against the pinned native headers. Raw ImGui operations require an active context, current fonts/draw lists and registered texture lifetime. The mailbox representation contains no padding, references, booleans or Rust enums: every copied word has a valid integer/float representation.

## Hook sites

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

MHGU DPS Meter v1.3.1 chains onto an already patched site after the first update, so it lies over this mod's branch rather than meeting the load check.
On Eden it logged `main+0x00170968 already patched (0xE51FF004); chaining`, the first word being this mod's branch, so it stacks on these hooks.

On MHGURecomp, `DrawList` is a guest mirror of a host object, and `VtxBuffer.Data` (guest +32, `MF_DATA` in its `forge_imgui_thunks.inc`) points at host memory with no guest address, so it reads 0 while `Size` is real. The vertex edits in `Renderer::number` then land on the null page (`null page` warnings at +0x475C..+0x490C) instead of the vertices. Wilds outline copies stay under the fill, which the user confirmed in play, Rise loses its outline and width stretch by the same reading, and Classic is unaffected because it passes offsets to the text call. Not fixed. Writing through `VtxBuffer.Data` works on Forge only.

A twelfth signature window checks the crop call and applied-field stores (below)
before any hook is installed.

## What capture publishes

Capture publishes every finalized hit amount, not only shown families; the
consumer's visibility rules decide what appears, and bounce association uses only
the local/ownerless subset. The `Source` byte in each record is fixed (Poison 6,
Blast 7) because `runtime.rs` decodes those values.
