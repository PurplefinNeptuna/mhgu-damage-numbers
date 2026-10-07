# Rise damage-number feasibility

Inspected 2026-10-07 against the supplied Steam Rise installation and the
current MHGU Damage Numbers r20 source; updated 2026-10-08 with the user's
30 fps hit screenshots. A Rise style is feasible within the
existing Forge plugin. Its damage capture, ownership, camera projection and
individual-hit queue already supply the required data. The work belongs in
presentation, font/effects and settings; no additional native damage hooks are
needed for the proposed appearance.

This is a research result. No runtime code, accepted NRO, release package,
installed mod, emulator configuration or save was changed. Neither game was
launched. [Inspection record](research/rise-ui-inspection.json) retains the
resource locations, hashes, decoded curves and evidence limits. The
[screenshot record](research/rise-screenshot-inspection.json) adds hashes for
all 17 supplied PNGs, approximate pixel measurements and corrected design
directions.

## What Rise contains

Read both `re_chunk_000.pak` and `re_chunk_000.pak.patch_001.pak` under the supplied
`/mnt/storage2/SteamLibrary/steamapps/common/MonsterHunterRise/`. Selective
extraction followed patch precedence. The damage GUI and selected digit font
come from the base archive; the English/European font-slot table comes from
patch_001.

| Resource | Finding |
| --- | --- |
| `natives/stm/gui/04_quest/hud/damagecount.gui.430033` | GUI version 430033, embedded CLIP version 43 |
| `txt_DamageCount` | `via.gui.Text`, font `Slot1` |
| `natives/stm/gui/font_eul.fslt.2` | Slot1 selects `gui/01_Common/Font/GP-UDKakugoC80Pro-DB.oft` at 1.0 width/height multipliers |
| `natives/stm/gui/01_common/font/gp-udkakugoc80pro-db.oft.1` | Decrypted name: GP-UDKakugoC80 Pro, DB; weight 600, units-per-em 1000 |
| `crcl_DamageEfx` | `via.gui.Circle` with authored expanding ellipse and color/alpha curves |

The text's static font size is 26 by 26, raster size 30 by 30. Its black
`GlowColor` is enabled with blur size 7 by 7 and strength 12. `ShadowEnable` is
false. Rise therefore uses its glow facility for the dark text edge; the word
"glow" here does not imply a persistent colored halo. The animation briefly
changes the glow color before settling back to black.

At the same measured 48-pixel ink height, unoutlined FreeType advances for
`0123456789` are 303.4375 pixels for the extracted font and 365.9375 for r20,
a ratio of about 0.829. This measures isolated font geometry. The supplied
screenshots show broad rendered digits with thick black outlines. My earlier
recommendation to choose a narrower face from that preview was incorrect;
judge glyph shape and width with the complete outline at the target scale.
The native font-slot identity above remains an asset finding, while matching
FreeType, Forge/STB and RE Engine pixels remains unverified.

## Authored animation

The panel contains 13 clips, including `Normal`, `Critical`, `OTOMO`, `GATE`,
`TOPOGRAPHY`, `STRUGGLE` and their exit variants. Parsing recovered 235
properties and 567 leaf keys. The following values are native GUI design
units and authored clip frames, without an assumed runtime frame rate.

| Clip | Entrance font-size keys: frame:size | Settled color | Main clip length |
| --- | --- | --- | --- |
| `Normal` | 0:16, 2:30, 7:22, 9:25 | White `#FFFFFF` | 30 frames |
| `Critical` | 0:16, 3:40, 7:32, 9:35 | Orange `#F0B41E` | 32 frames |
| `OTOMO` | 0:16, 2:30, 7:22, 9:25 | Gray `#808080` | 30 frames |

Width and height follow the same keys. The orange clip settles at 35/25, or
1.4 times the normal size. It is a grow/shrink/settle bounce, unlike r20's
monotonic 1.2-to-1.0 stamp. Entrance opacity reaches full alpha at frame 7 for
Normal/OTOMO and frame 3 for Critical. Their text stays at local y=0 throughout
the main clip.

All three exit clips last 10 authored frames. They retain settled font size,
move local y from +4 to -50, and fade alpha from 255 to zero. The inspected
scalar curves use Linear (2) and terminal Discrete (1) interpolation. The
complete color and size keys are in the inspection record.

Clip names alone do not prove which damage condition selects them. In
particular, `Critical` must not automatically be mapped to the plugin's
positive-affinity `Critical` enum. The inspected GUI contains text and circle
effects, with no separate Wilds-style crit-star element. Exact runtime
selection, any dynamic overrides, the hold state's duration and seconds per
clip frame remain unverified.

## Local Rise customization

The local `Adjustable_Damage_Number_Size/damage_numbers.lua` accesses
`snow.gui.GuiDamageDisp.NumDisp`, `_DamageText`, `DispType` and `FontSize`.
It hooks `setRoutine`, `routineIn`, `routineWait`, `routineOut` and
`stateFinishEvent`. Its six labels are WHITE, ORANGE, BIG_ORANGE, GRAY, RED
and BIG_GRAY. Those labels describe mod-side handling; they do not establish
the native conditions for each display type.

The saved configuration says enabled and sets all width/height multipliers to
0.6. Loading/activation was not verified. Consequently, a later comparison
with this installation must distinguish stock asset values from the user's
potentially reduced display size. The new hit screenshots establish rendered
appearance for their capture, without establishing whether this size mod was
active.

## User's 30 fps hit screenshots

Inspected all 17 supplied 1919 by 1080 PNGs in `hit_screenshots`: four
`norm_orange` samples (38), seven `positive_crit_orange` samples (48) and six
`negative_crit_orange` samples (58). The older 78 beside the negative crit is
a separate number already rising/fading; it must not be used to measure 58's
entrance, scale or anchor. Numbered filenames identify samples; `before_go`
is the user's exit-transition marker and has no timestamp or guaranteed
adjacency to the previous numbered file.

| Samples | Observed effect |
| --- | --- |
| 1 and 2, all three groups | Shared blue-white impact burst/horizontal flare; digits are strongly washed out, with a transient orange/red edge in sample 2 |
| Noncrit 3 | Orange digits with thick black outline, no long crit slash |
| Positive 3 and 4 | Long red glow slash with bright pale core behind the orange digits; sample 4 reaches beyond the top screen edge |
| Negative 3 and 4 | Long violet/purple glow slash with bright pale core behind the orange digits; sample 4 reaches beyond the top screen edge |
| Both crits 5 | Thin residual slash and sparks as the beam fades |
| Both crits 6 | Long beam gone; orange digits remain |

The crit effect stretches through the impact, with considerably more length
above it than below. It expands/brightens and then thins/fades. Its short
lifetime is independent of the number's later hold/exit. Assuming consecutive
30 fps numbered samples, the three intervals from 3 to 6 cover approximately
100 ms; this does not establish a complete native effect duration. These
screenshots locate the slash in the rendered scene but do not identify which
native GUI/effect resource creates it. The user identifies it as a world-space
hit effect whose endpoint is constrained by the scene/ground. The MHGU proposal
below adapts its appearance to the existing UI renderer. A single slash direction per polarity
does not show a polarity-specific lean rule.

Colored beam fits above the impact estimate the positive sample at roughly
13 degrees from vertical and the negative sample at roughly 8 degrees, leaning
opposite ways. Bloom makes the positive fit less precise. The two hits are
consistent with the user's revised approximate +/-15 degree range, but do
not establish its endpoints, distribution or whether the game randomizes it.

At `before_go`, the masked orange top moves down 4 pixels in both the normal
and positive samples; the bottom moves down 5 and 3 pixels respectively.
Antialiasing and reticle overlap limit precision. This agrees with the decoded
exit transition from main y=0 to exit y=+4, followed by upward motion toward
-50 with fading alpha. Reproduce the brief downward kick before the rise;
use elapsed time so it resembles one 30 fps sample without changing speed
with the emulator's frame rate. The supplied set has no complete exit tail
and does not measure the hold duration or native animation tick rate.

The normal exit sample has roughly 30 pixels of orange fill height. A scan
through the right edge of its 8 at y=566 has gold/antialiased pixels at x996-998,
a near-black band at x999-1003, then the background transition. This gives an
approximate 4-5 pixel dark edge at that row, around 13-17% of fill height as a
starting proportion for previews. It is a thick nearly solid outline with a
lightly softened outside edge; r20's 0.5 px outline is insufficient as a Rise
reference. This local scan is not an exact global stroke radius or proof of
stock scaling. Font preview candidates must include the outline, counters,
spacing and overall outlined width, including multi-digit numbers.

Orange survives unchanged across noncrit, positive and negative examples;
affinity is expressed by the separate slash. Only orange was supplied.
Shared motion and outline behavior for the other colors is the user's design
direction, not a measurement of additional color samples.

## Proposed MHGU implementation

Add `Rise` as a third style while retaining the accepted Wilds and Classic
choices. Keep the current capture and finalized totals. Use an open digit face
or independently authored digits calibrated to the screenshots' broad shapes
and thick black outlines, the measured white/orange palette, entrance size
bounce, hold, downward exit kick and upward fade. Reserve style-specific
maximum outlined dimensions for collision placement.

Per the user's direction, omit the blue-white impact flash visible in the
first two samples: MHGU already has its own hit effects. Keep the digit
entrance animation and use a black outline; suppressing the hit flash must
not hide or delay the number. Positive/negative affinity can select an
independently authored red/purple tapered slash with a pale core and fading
colored glow behind the digits. Noncrit gets no slash. The short effect
expires separately from the held number; do not carry it into the exit fade.

Treat the slash as a UI approximation of Rise's world effect. Start previews
with a total vertical extent of half the gameplay viewport height; compare
three-quarters height if the shorter version loses the intended appearance.
At 1080p these are 540 and 810 pixels, at 720p 360 and 540. Half height is the
initial candidate, not a measured Rise dimension or a final accepted size.
Keep the core/glow narrow and short-lived so repeated crits remain readable.

The existing plugin projects each event's saved world hit position every
frame, then applies digit motion and number-placement offsets. A separate
slash anchor can use that raw projection before those offsets, following
camera movement without following the digit bounce/rise. Draw the slash behind
the digits and clip to the gameplay viewport. Its large bounds should not
enter the digit collision solver or force numbers away from the hit. This
requires separate effect geometry/age in presentation; current `DrawNumber`
markers are attached to the moving number and share its alpha.

The inspected path renders with ImGui draw lists. It supplies no ground-plane
intersection or scene depth test for this overlay, so it cannot reproduce the
native effect's ground stop or monster/terrain occlusion. Keep the lower tail
short as an appearance choice; do not infer a ground boundary from the number's
screen y. A camera-projected UI streak remains an overlay.

For the user's revised approximate +/-15 degree range, choose slope `s` in `[-1/4,+1/4]`
once per hit and retain it for the effect's lifetime. An upward vector `(s,-1)`
has a maximum tilt `atan(1/4)`, approximately 14.04 degrees. Build endpoints
as `(x+s*dy, y-dy)`, and a perpendicular along `(1,s)`; normalize once per hit
if consistent physical width/length is needed. This avoids per-frame sine,
cosine or degree conversion. Radians alone do not remove that work. An
existing stable hit identity can seed a small hash for varied tilts, separate
from polarity. This is a proposed approximation, not a recovered Rise RNG.

Use the existing shared `tint()` classifier for every style. A confirmed bounce
selects gray first; otherwise physical HZV >=45 selects orange/yellow, while
HZV <45 or unavailable selects white. Style chooses palette/appearance only.
Rise borrows these native GUI colors:

| Shared MHGU tint | Rise palette reference |
| --- | --- |
| Weak: physical HZV >=45, no confirmed bounce | Orange/yellow `#F0B41E` |
| White: physical HZV <45 or unknown, no confirmed bounce | White `#FFFFFF` |
| Gray: confirmed bounce, regardless of HZV | OTOMO gray `#808080` |

`OTOMO` supplies the gray color only; companion ownership does not select it.
Do not add Rise's display categories, companion-specific coloring, affinity
color selection or separate style classifiers. Existing Wilds/Classic palettes
keep their own colors with the same classification. The native clip names
remain asset evidence; MHGU affinity selects only the separate crit effect.

Implementation points:

- `src/model.h`: new style value and Rise palette; retain shared `tint()` and
  captured affinity independently.
- `src/presentation.h`: Rise motion, larger orange bounds, anchor alignment and
  maximum entrance dimensions; retain fixed per-hit placement. Supply a separate
  raw projected hit anchor and short effect lifetime for the UI slash, outside
  digit motion and collision bounds.
- `src/cosmetics.cpp`: digit font, thick black outline and red/purple slash. Existing Forge/ImGui
  font, texture and draw facilities cover these operations. An independently
  generated alpha texture can provide the softened edge/glow; matching RE Engine's blur
  and blend math requires renderer comparison.
- `src/settings.h` and `src/capture.cpp`: encode/decode `style=rise`, extend the
  style selector, and retain existing version-2 preferences.

Do not feed the extracted font or GUI into the distributable open-asset build.
They remain local research references. Build the new appearance through an
open font recipe and independently generated effects, then extend provenance
validation to cover it. No new hooks or CodeMemory arenas are expected, and
the current eight-site NZCVQ/GE audit remains applicable if gateways are
unchanged.

A candidate needs host/real-ImGui previews, meaningful compiled ARM checks for
colors, motion, placement and settings, the GE instruction audit, a new runtime
version and the normal distribution gate. Fresh Eden/Ryujinx gameplay should
check dense bowgun bursts, bright backgrounds, companion/environmental hits,
camera movement and saved-style restart. Existing r20 evidence does not prove
the proposed renderer works in-game. Forge's recorded menu/overlay renderer
limitation still needs to be considered when assessing a live visual defect.

## Recheck the evidence

Ignored local research files are under `work/damage-numbers/rise-research/` in
the coordinating workspace: extraction script and path list, source copies,
GUI/CLIP inspection scripts, decoded attributes/keys, compressed-range hashes
and `font-reference.png`. That PNG shows unoutlined extracted glyphs and the
settled palette; use the user screenshots to judge actual presentation.
Inputs and generated crop sheets remain local research references, outside
source exports.

Reproduce the screenshot measurements from the independently exportable mod
folder, supplying a copy of the user's screenshot directory:

```sh
python3 scripts/inspect_rise_hit_screenshots.py /path/to/hit_screenshots /tmp/rise-screenshot-inspection.json
```

The script uses sample-specific ROIs and color masks, records input hashes,
and leaves PNGs untouched. It is an analysis helper, not a general HUD detector
or an automated gameplay test. Outline thickness and phase descriptions still
require visual interpretation.

The parser checks GUI/CLIP magic/version, table dimensions, property/key bounds,
and decoded sizes. The final three relevant copies were rehashed against their
provenance, and their original compressed ranges were independently hashed.
Full archive hashes were not computed; archive lengths, headers and inspected
range hashes are recorded explicitly. Production payloads were untouched.

Format sources: [Ekey's PAK resource lists](https://github.com/Ekey/REE.PAK.Tool),
[ree-pak-rs](https://github.com/eigeen/ree-pak-rs),
[GuiFile.cs](https://github.com/kagenocookie/RE-Engine-Lib/blob/ceb0d2fbe9d76f29c552e4c56f4fd8cd87a165ef/REE-Lib/RszFile/GuiFile.cs),
[ClipFile.cs](https://github.com/kagenocookie/RE-Engine-Lib/blob/ceb0d2fbe9d76f29c552e4c56f4fd8cd87a165ef/REE-Lib/RszFile/ClipFile.cs),
and [the Rise size mod](https://github.com/GreenComfyTea/MHR-Adjustable-Damage-Number-Size).
