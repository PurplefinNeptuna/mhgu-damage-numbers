# Damage Numbers: crop, projection and assets

Worked example from `mods/damage-numbers/`. Game facts it rests on are in `mhgu-understanding`.

## Visible crop

MHGU can display a smaller crop of an unchanged swapchain allocation. Forge's
ImGui DisplaySize describes that allocation, so it can report 1920×1080 bounds for a
visible 1280×720 region. The user reported correct docked placement and displaced
handheld placement on Switch and Android against r28, and later reported it fixed
(build and hashes not stated). Exact live dimensions were never measured.

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
address and reads 0 (see the `VtxBuffer` note under [Hook sites](damage-capture.md#hook-sites)).

An unavailable/invalid crop retains the previous Forge bounds to keep damage
visible and emits one lifetime-deduplicated warning once a player exists. That
fallback can retain the original displacement. No native pointer is cached or
dereferenced by deferred rendering.

## Projection and centre fallback

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

## Prepared assets and pixel parity

Only the numeric font scale and atlas rectangles remain in `src/assets.rs`. Font and bitmap bytes live in the package's external asset files. File sizes are checked before allocation; the exposed ImGui file APIs read directly into the owned texture buffer, and the local file guard closes every successful open. Short reads destroy the incomplete texture. Wilds loads eagerly and Rise once on its first critical effect. Failed assets are not retried in the same context; each new context retries. Missing textures omit their effects without disabling the model or digits.

A separate context-shutdown callback advances the generation even if no texture loaded. Texture callbacks only unregister/free their own allocations. This makes font/resource invalidation independent of successful artwork loading and prevents stale handles if a context address is reused. Warning flags survive reinitialization for the process lifetime, while resource pointers and load attempts reset.

The prepared font retains Open Sans Semibold naming. Its glyph outlines, hinting, metrics and weight match r2; only naming metadata/checksums changed. Artwork pixels and atlas geometry are unchanged.

Ordinary text, image, line and clip operations use forge-rs DrawList wrappers. Raw access remains for glyph ink bounds, Rise's exact vertex stretch, texture registration and quad drawing. Style selection is Wilds, Rise, Classic, with the existing internal numeric IDs retained for compatibility.
