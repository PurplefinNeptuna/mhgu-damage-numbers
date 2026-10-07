# Rise damage-number feasibility

Inspected 2026-10-07 against the supplied Steam Rise installation and the
current MHGU Damage Numbers r20 source. A Rise style is feasible within the
existing Forge plugin. Its damage capture, ownership, camera projection and
individual-hit queue already supply the required data. The work belongs in
presentation, font/effects and settings; no additional native damage hooks are
needed for the proposed appearance.

This is a research result. No runtime code, accepted NRO, release package,
installed mod, emulator configuration or save was changed. Neither game was
launched. [Inspection record](research/rise-ui-inspection.json) retains the
resource locations, hashes, decoded curves and evidence limits.

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

The native font is narrower than r20's Open Sans Semibold. At the same measured
48-pixel ink height, FreeType advances for `0123456789` are 303.4375 pixels for
Rise and 365.9375 for r20, a ratio of about 0.829. This measures font geometry;
it does not establish matching pixels between FreeType, Forge/STB and RE Engine.

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
potentially reduced display size. Existing screenshot samples inspected were
cinematic/scenery views and did not verify damage-number rendering.

## Proposed MHGU implementation

Add `Rise` as a third style while retaining the accepted Wilds and Classic
choices. Keep the current capture and finalized totals. Use a narrower open
digit face or independently authored digits, the measured white/orange
palette, black soft edge, brief entrance flash, bounce/hold/rise/fade sequence
and style-specific maximum bounds for collision placement.

The proposed initial rule is to retain MHGU's physical HZV >=45 selection for
orange and confirmed-bounce selection for gray. This is an MHGU design choice,
not a verified reconstruction of Rise's native classifier. `OTOMO` supplies a
separate gray presentation reference if companion coloring is desired. Avoid
using positive affinity to select the larger orange clip merely because of its
name. Decide affinity indicators separately using MHGU's existing captured
polarity.

Implementation points:

- `src/model.h`: new style value and palette; distinguish HZV from affinity.
- `src/presentation.h`: Rise motion, larger orange bounds, anchor alignment and
  maximum entrance dimensions; retain fixed per-hit placement.
- `src/cosmetics.cpp`: digit font, black edge and flash. Existing Forge/ImGui
  font, texture and draw facilities cover these operations. An independently
  generated alpha texture can approximate the blur; matching RE Engine's blur
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
and `font-reference.png`. The PNG shows extracted glyphs and the settled
palette, without simulating engine effects. They are not export dependencies.

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
