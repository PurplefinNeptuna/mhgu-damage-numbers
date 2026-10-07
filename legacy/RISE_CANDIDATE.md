# Rise style candidate

[R21-rise1](RISE_CANDIDATE.json) adds Rise alongside Wilds and Classic. It is a DEVELOPMENT candidate,
with the user-selected three-quarter-height slash as the only Rise height.
Half-height frames remain comparison evidence; there is no height setting. Accepted r20, its distribution
record, ZIP, runtime and evidence stay intact. No game was launched and no
installation, activation, save/settings change, publication or push occurred.

## Appearance and placement

All three styles retain `tint()`: confirmed bounce wins and selects gray;
otherwise physical HZV at least 45 selects weak color, and lower/unknown HZV
selects white. Rise uses `#F0B41E`, `#FFFFFF` and `#808080`. Companion ownership
never selects gray. Affinity changes only the slash.

Rise uses the existing renamed OFL Open Sans Semibold subset, widened to 1.08
its glyph width. Eight opaque black outline passes surround the fill, with a
soft open glyph shadow underneath. Outline radius is 0.10 of font size, scaled
with resolution and the entrance. Normal/gray settle at 25 design units and
orange at 35. Wilds remains the default and retains r20's font, palette,
half-pixel outline and 1.5x stars. Classic retains its font, palette and motion.

The common Rise entrance grows from 0.64 to 1.20, shrinks to 0.88 and settles
at 1.00. Entrance takes the smaller of 0.30 seconds and 30 percent of Duration.
The number then holds. Exit takes the smaller of 10/30 seconds and 35 percent
of Duration. Its first tenth kicks downward to +4 design units; the rest rises
to -50 while alpha fades to zero. Duration remains the complete number
lifetime. These are UI timing choices, not measured Rise engine seconds.
There is no extra blue-white hit flash.

Positive crit uses an independently authored red beam with a pale core;
negative uses violet. Noncrit has none. A beam expands/brightens for 0.05
seconds, then thins/fades for 0.15 seconds. It expires while the digits remain
held. Its peak bounding width is 48 pixels at 720p; the Gaussian glow and pale
core occupy only part of that width. The registered hit centers the beam, with equal upward and downward reach.
There is no ground cutoff. This follows the user's 2026-10-08 correction to the
handoff's earlier short-tail proposal.
The effect's size and alpha do not inherit the digit pop, size slider, collision
placement or exit fade.

The origin is the current-camera projection of the saved `Event.position`.
No digit anchor correction, collision offset or subsequent monster position
is applied. A stable hash of all four `HitKey` fields selects slope within
[-0.25,+0.25], approximately +/-14.04 degrees, independently of affinity.
The same slope persists during expansion/fade without per-frame trigonometry.
All slashes draw before all digits, and clip to the gameplay viewport. They
are excluded from the digit collision solver. Terrain intersection and scene
occlusion remain unavailable in this UI approximation.

## Renderer comparisons

From the mod folder, `./build.sh test` generates real pinned ImGui draw lists,
textures and CPU composites under the versioned `work/` output. The compositor
retains r20's exact axis-aligned path and handles tilted geometry with indexed
triangle coverage, bilinear UV sampling, RGBA interpolation, a shared-edge
coverage rule and command clipping. Analytic fixtures cover those operations.

In the coordinating workspace, previews are at:

- `work/damage-numbers/candidates/r21-rise1/open-visual/rise-height-comparison-dark.png`
- `work/damage-numbers/candidates/r21-rise1/open-visual/rise-height-comparison-bright.png`
- `work/damage-numbers/candidates/r21-rise1/open-visual/rise-timeline-dark.png`
- `work/damage-numbers/candidates/r21-rise1/open-visual/rise-timeline-bright.png`

Half-height is on the left and three-quarter-height on the right. Columns show
white, orange and bounce gray. Rows show noncrit, positive and negative hits;
the bottom group contains crowded companion hits with both polarities. Slashes
from lower hits can pass behind other numbers. Forty frames cover entrance,
beam expansion/fade, hold, exit and expiry on bright/dark backgrounds, with
deterministic matching hits at both heights. The manifest records draw/preview
hashes and elapsed times. These are faithful plugin renderer previews, without
an MHGU GPU or gameplay result.

## Assets and lifecycle

[The open recipe](fonts/rise-open.json) and
[procedural generator](scripts/prepare_damage_numbers_rise.py) own the new
artwork. No Rise font, GUI, screenshot pixels or effect asset enters a build.
The asset audit blocks Rise/Wilds proprietary reads and compares the compiled
beam stream with every generated RGBA byte. Existing Wilds font/art remains
unchanged. The shared DN-LZ1 decoder runs once when a context first needs a Rise
slash. Failure is attempted once per context and logged once per process;
digits continue. Closed settings do not stop rendering.

Forge's actual pinned exports include `ImDrawList_AddImageQuad`, texture APIs
and `igAddContextHook`. A shutdown hook unregisters and frees Rise's CPU
texture data. ImGui owns the font atlas; embedded font data stays immutable.
The Forge backend's NVN shutdown is still a TODO and offers no complete GPU
teardown facility. This candidate does not claim to repair that loader issue,
or its separately disclosed menu/background rendering corruption.

## Build, validation and delivery

Run the mod-local entry point with configured pinned offline inputs:

```sh
./build.sh doctor
./build.sh prepare
./build.sh build
./build.sh test --host
./build.sh test
./build.sh verify
./build.sh package
```

`mod.json` selects `r21-rise1` and marks it as a candidate. Build, probe,
renderer and receipt paths all use `build/damage-numbers/candidates/r21-rise1/`
and `work/damage-numbers/candidates/r21-rise1/`; standalone exports use those
same relative paths inside their own checkout. Builds also reproduce the
normalized main image from the fingerprinted copied NSO. No downloads occur.
`package` produces a DEVELOPMENT candidate ZIP. `package --release` cannot
promote it through r20's accepted record.

Host fixtures cover palettes, ownership/affinity independence, settings,
entrance/exit at 30/60 fps, same-frame individual hits, stable tilt, raw origin,
camera movement, crowded cached digit placement and separate beam expiry.
Real ImGui exercises both heights, polarities and all digit colors, with
128 ten-digit labels below 16-bit vertex indexing capacity. Compiled ARM
fixtures cover the Style selector, version-2 save/restart, tilted-quad ABI,
raw hit origin, short independent lifetime, one-time decode, malformed texture
fallback and shutdown cleanup. Existing damage, ownership, gateway, FS/TLS,
compression and GE fixtures remain required.

Wilds draw JSON, atlas/artwork textures and preview must match the tracked
accepted r20 hashes. Gateway assembly and all eight native hook sites remain
unchanged; nine emitted NZCVQ-only restores must pass the existing GE audit.
No additional hook or CodeMemory arena is introduced. The full workspace
`python3 scripts/mod_distribution.py check` continues to verify r20 and the
other current deliveries. Candidate validation does not grant gameplay or
release acceptance.
