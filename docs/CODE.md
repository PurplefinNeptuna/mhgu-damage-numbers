# Code walkthrough

## From a native call to a visible number

`src/lib.rs` declares Forge's entry and rendering callbacks. Startup creates the stock `ThreadLocal<CaptureState>`, then validates the exact game windows and installs all eight ordinary hooks. Capture becomes active only after the last hook succeeds.

`src/gateway.S` saves a 328-byte stack image before Rust runs: continuation scratch at 0, CPSR/FPSCR at 8/12, d16–d31 at 16, d0–d15 at 144, and r0–r12/LR at 272. Native stack arguments remain above the original SP. Around hooks replace the saved LR only after `CaptureState::enter` records the caller; the shared after gateway pops that scope and resumes its exact caller. Camera and melee HZV taps observe registers before the displaced instructions execute. `core/src/capture.rs` explains each copied payload and nested scope.

`src/runtime.rs` obtains a short mutable TLS borrow for each helper and releases it before assembly branches to the original trampoline. Nested original calls can therefore access the same thread-local state again without an overlapping Rust borrow. First access may allocate through the stock ThreadLocal helper. Capture then uses fixed storage: 64 scopes, 16 producer mailboxes with 256 records each, and copied numeric payloads.

Mailboxes belong to static plugin storage and are never recycled during the process. Each producer owns one slot; the consumer owns each Cursor. Thread exit drops its CaptureState while queued records remain valid. Atomic payload words and publication tickets reject torn snapshots, and overflow advances to the newest available records. Pool exhaustion passes through without capturing. This matches the r25 limit of 16 producer contexts.

The consumer copies records into `core/src/lib.rs`'s bounded Model: 256 pending events, 128 visible numbers and 512 recent identities. It applies player/time ownership, environmental-source rules, bounce feedback and duplicate suppression. Player or camera changes reset the display epoch. Game identities are used for comparisons only; the rendering code never follows captured object pointers.

`core/src/presentation.rs` projects copied matrices, reserves peak animation bounds, chooses stable placement, and produces DrawNumber values. It contains the three animation styles and r25 size constants. `src/render.rs` measures actual baked glyphs and submits those draw values to ImGui. Effects precede digit fills, preserving r25 ordering, colors, outlines, stars and Rise slash geometry.

## Hook and asset lifetimes

`src/hooks.rs` checks the complete signature windows, instruction relocation whitelist, alignment, bounds and reserved/overlapping redirects before calling Forge. Disabled upstream Patch objects snapshot all originals. Forge creates the eight ordinary hooks in its shared pool; the installer checks each returned ARM trampoline's displaced words, return jump and executable mapping before publishing its continuation. Capture activates only after every branch and synchronization passes.

Rollback checks each current word against the saved original and this plugin's branch. A pair containing foreign bytes is left untouched. For owned or partially published pairs, the Patch enable/disable operations restore the snapshot, followed by cache synchronization and readback. Snapshots remain disabled outside that restoration, so their destruction cannot overwrite foreign modifications. Valid continuations are retained even after a later failure, and incomplete rollback requires restart. There is no live installation, uninstall or trampoline release. The Forge pool owns all trampoline memory until process exit; retaining returned Hook structs is unnecessary.

Eight ordinary ARM hooks use 16 bytes each, totaling 128 bytes of slots in Forge's shared pool of at most two 16 KiB chunks. The host fixture confirms sharing and failure behavior against the pinned native allocator. Actual emulator pool use remains a runtime check. The load callback is a no-op, leaving the loader's requirement record zero; initialization uses the loader's zero-argument ABI.

Font loading uses `ImFont::load_with` from RomFS OpenSans.ttf. Its configuration allows load errors to return a fallback instead of asserting. The atlas owns the loaded file data. Each file-loaded RGBA texture is registered with ImGui and has a shutdown callback to unregister and destroy it. Context epochs prevent initialization from keeping pointers after shutdown, including when an allocator reuses a context address. Repeated initialization of a live context does not load duplicate assets.

## Settings

`src/config.rs` derives upstream IniSerialize/IniDeserialize. Individual field wrappers define true, 1.0 and 0.9 defaults, and reject invalid or nonfinite values; Style defaults to Wilds. Once save data and MtFile are available, stock `config::load` reads the file or creates it with defaults. Save uses stock ScopedMount and Writer. Reset updates session settings; Save persists them. These are the explicitly requested forge-rs saving semantics, superseding the older custom persistence rules for this mod.

The unframed settings tree is closed by default. Its guard balances TreePop on every opened path. Free-render processing runs independently of whether that tree is open.

## Unsafe boundaries

Native address access checks mappings and copies values into owned numeric structures. FFI and assembly requirements are documented beside each unsafe block. Compiler-derived layout data stays in the NRO for verification against the pinned native headers. Raw ImGui operations require an active context, current fonts/draw lists and registered texture lifetime. The mailbox representation contains no padding, references, booleans or Rust enums: every copied word has a valid integer/float representation.

## Prepared assets and pixel parity

Only the numeric font scale and atlas rectangles remain in `src/assets.rs`. Font and bitmap bytes live in the package's external asset files. File sizes are checked before allocation; the exposed ImGui file APIs read directly into the owned texture buffer, and the local file guard closes every successful open. Short reads destroy the incomplete texture. Wilds loads eagerly and Rise once on first use. Missing textures omit their effects without disabling the model or digits.

A separate context-shutdown callback advances the generation even if no texture loaded. Texture callbacks only unregister/free their own allocations. This makes font/resource invalidation independent of successful artwork loading and prevents stale handles if a context address is reused. Warning flags survive reinitialization, while resource pointers and load attempts reset.

Ordinary text, image, line and clip operations use forge-rs DrawList wrappers. Raw access remains for glyph ink bounds, Rise's exact vertex stretch, texture registration and quad drawing. No animation/layout formula changed. Style selection is Wilds, Rise, Classic, with the existing internal numeric IDs retained for compatibility.
