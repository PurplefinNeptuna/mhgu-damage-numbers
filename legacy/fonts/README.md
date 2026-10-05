# Wilds digit font and effects

R12 uses the default Wilds damage font: **Frutiger Neue LT W1G Medium**, resolved
from the UI020020 font slot to `gui/ui_font/font_efigs0.oft.1`. It replaces the
Roboto approximation used in r10/r11. The user's reference is the default game
UI, not Enhanced Damage Numbers, which happens to be present in their Wilds
installation.

[Source manifest](source.json) pins the original OFT and converted input hashes.
[Asset provenance](../WILDS_ASSETS.json) records the selected texture/UV/layout
resources and tool revisions. Extraction reads the local installation into
ignored workspace copies; it never modifies or launches Wilds.

The OFT wraps a CFF font. Forge's STB rasterizer requires quadratic outlines,
so the recipe converts only digits, space and question mark with FontTools
4.66.1, at a maximum curve error of 0.25 font units out of 1,000 units per em.
The embedded subset is 2,352 bytes, SHA-256
`b2a79e1f114f576f4cf47f68547cab3202f1e48a1b2864e7557984ad5e061b2c`.
Visible digit height is normalized against Forge's existing Roboto Medium:
`0.8621484555`. This retains the accepted on-screen number height.

The original crit sprites come from UVS patterns 2 and 3 in the 512x256 damage
texture. Their original RGBA color, pale core and bloom are retained. Smooth
glyph-effect masks replace repeated hard-outline text passes. The native GUI
records shadow blur 6x6/strength 5 and foreground glow blur 4x4; Gaussian mask
conversion is an approximation of the RE Engine shader. Forge rasterization
and the blur conversion still need an in-game comparison; using the original
assets does not establish a pixel-perfect renderer match.

## Reproduce this local build

Prerequisites: the user's Wilds installation, the pinned PAK path list and
cipher source under `tools/wilds-reference/`, FontTools 4.66.1, Pillow/NumPy,
and the local GDeflate wrapper. Tool revisions and source hashes are recorded
in [WILDS_ASSETS.json](../WILDS_ASSETS.json). Build the GDeflate wrapper from its
pinned checkout with CMake `-DCMAKE_POSITION_INDEPENDENT_CODE=ON`.

```bash
python3 scripts/extract_wilds_damage_assets.py
python3 scripts/prepare_damage_numbers_wilds_source.py
python3 scripts/prepare_damage_numbers_font.py
python3 scripts/prepare_damage_numbers_sprites.py
python3 scripts/test_damage_numbers_visual.py
```

PAK Zstandard entries require Python's `compression.zstd` (available in the
local Python 3.14). Extracted originals, converted font, generated headers,
textures and preview stay in ignored `work/damage-numbers/` paths. The recipes
and fingerprints are tracked, without the game-derived payload. This personal
candidate contains local game assets; `LOCAL_ASSETS.txt` accompanies the package.
The old OFL file remains as historical attribution for r10/r11 only.

## Forge registration

`forge_onImGuiInit` registers the embedded font through Forge's exported ImGui
API. Static font bytes and glyph ranges live for the context, with
`FontDataOwnedByAtlas=false`; the temporary font config is destroyed. Font
registration failure logs once and falls back to Forge's font.

The 512x256 RGBA effects texture is registered once per context through
`igRegisterUserTexture`, using `ImTextureData` and `ImDrawList_AddImage`. Forge's
installed renderer advertises `RendererHasTextures` and handles its GPU upload.
Texture pixels and metadata remain alive while the context uses them. The
baseline loader has no plugin unload callback. Missing texture support logs
and leaves crisp digits; capture and Classic rendering continue.

## Alternate open-asset edition

`--assets open` selects [open-source.json](open-source.json): pinned Google
Fonts Roboto, width 100, weight 600, renamed to Damage Digits Open, under the
[full SIL OFL notice](OFL.txt). It does not read the extracted Wilds font or
texture. Sprite generation uses independently authored geometric artwork.
The open preparation/build/preview outputs are isolated from the accepted
personal r12. See [OPEN_ASSETS.md](../OPEN_ASSETS.md).
