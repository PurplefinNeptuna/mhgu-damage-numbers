# Wilds digit font

Full-width Roboto Bold is instantiated at `wdth=100`, `wght=700` and subset to digits
0–9 plus space and a fallback question mark. It approximates the reference font;
it is not extracted from Monster Hunter Wilds.

Source: [Google Fonts metadata](https://github.com/google/fonts/blob/main/ofl/roboto/METADATA.pb).
Pinned revision, source URL/hash, OFL hash and FontTools version:
[source.json](source.json). Full license: [OFL.txt](OFL.txt).

For a fresh checkout, obtain the pinned source URL from `source.json` and save it
at its `original_path`. Install FontTools 4.66.1 in the build's Python environment,
then run `python3 scripts/prepare_damage_numbers_font.py`. The recipe verifies
input and license hashes before instancing/subsetting. Generated files live in
`work/damage-numbers/native-capture/`; they are not committed.

Subset SHA-256: `495d951b5c118e5c1169928eb8a606d4bf79f945f6f654e2dc34515eb96d0b1b`
(10,440 bytes). The preparation manifest also records the existing Forge font's
height-reference hash. Visible digit height is normalized against that font;
the current factor is 1.0.

`forge_onImGuiInit` registers the embedded TTF using Forge's exported ImGui APIs.
The static font buffer and glyph ranges stay alive for the context, with
`FontDataOwnedByAtlas=false`; only the temporary config is destroyed. A failed
registration logs once and uses Forge's font with Wilds colors/stars. Ownership
reference: [ImGui font documentation](https://github.com/ocornut/imgui/blob/master/docs/FONTS.md).

The NRO embeds the OFL notice; the staged package includes
`Roboto-OFL.txt`. Only digits/fallback glyphs are added to the atlas.

R11 replaces the r10 condensed weight-600 subset after the user's side-by-side
screenshots showed thin, narrow digits. The new zero's ink width/height ratio is
0.658 versus 0.554 for r10; height normalization remains 1.0. Original r10 source
and license provenance remain in Git history and its NRO rollback.
