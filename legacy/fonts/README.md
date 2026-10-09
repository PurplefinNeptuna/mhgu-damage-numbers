# Open damage digits and effects

The only supported font is **Open Sans Semibold 600**, instantiated at width
100 and weight 600 from pinned Google Fonts. [open-source.json](open-source.json)
records source, license, axes and FontTools hashes/version. The digit subset is
renamed **Damage Digits Open**, with the full [SIL OFL notice](OFL.txt) embedded
in the NRO and included in adjacent release notices. Digits, space and question mark are retained.
Visible height is normalized against Forge's existing Roboto Medium.

The 512x256 effect atlas uses this licensed font for soft glyph shading and
original geometric four-point stars, Gaussian bloom and pale highlights.
The broad red/blue halo is baked into the existing sprites. Their render size
is independent of font normalization; no additional runtime draw calls occur.
No Wilds installation, extracted texture, OFT or proprietary font is used.

```bash
python3 scripts/prepare_damage_numbers_font.py
python3 scripts/prepare_damage_numbers_sprites.py
python3 scripts/test_damage_numbers_visual.py
```

Work outputs remain under `work/damage-numbers/open-assets/`; real-ImGui preview
is `work/damage-numbers/open-visual/preview.png`. `--assets open` is optional;
`--assets local` is rejected. The old extraction/conversion entry points have
been removed. R12 provenance remains in Git history.

## Forge registration

`forge_onImGuiInit` registers the embedded font through Forge's exported ImGui
API. Static font bytes and glyph ranges live for the context, with
`FontDataOwnedByAtlas=false`; the temporary font config is destroyed. Registration
failure logs once and falls back to Forge's font.

The RGBA texture is registered once per context through `igRegisterUserTexture`,
using `ImTextureData` and `ImDrawList_AddImage`. Forge's renderer advertises
`RendererHasTextures` and handles upload. Pixels and metadata remain alive
while the context uses them. Missing texture support logs and leaves crisp
digits; capture and Classic rendering continue.
