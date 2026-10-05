# Damage numbers r16: open-only release

The sole supported build uses **Open Sans Semibold 600**, a renamed digit subset under
SIL Open Font License 1.1, and independently authored procedural red/blue crit
stars. It bundles no extracted Wilds font or texture. The artwork is generated
from a generic four-point polygon, Gaussian bloom and a pale highlight; it is
not traced from the Wilds sprite. Its shape and the font differ from the
retired original-asset r12 edition. The user selected Open Sans after comparing twelve
licensed font families at equal visible digit height.

The red/blue stars now have a stronger broad colored halo and horizontal bloom
streak. Their solid polygon, texture dimensions and render size remain unchanged.
Star size is independent of the new font's height normalization. The bloom is
baked into the existing texture without extra draw calls.

Soft number shading, gold halo, positive/negative marker placement, individual
hit capture, compact spacing, source filtering, Classic mode,
version-2 settings and the compact Forge panel use the existing implementation.
The Style selector keeps its saved Wilds/Classic values; here Wilds selects the
open-asset appearance. The game-derived r12 edition is retired.

Wilds style uses the accepted timing: a 1.2x stamp settles
with quadratic easing over 0.2 seconds, stays at its projected hit position,
then rises 30 pixels at 720p during the final 0.2-second fade. Critical stars
move/fade with their numbers. Duration remains the total lifetime: 0.9 seconds
holds for 0.7 seconds; 1.2 seconds holds for 1 second. At the minimum 0.4 seconds,
stamp and exit follow each other. Camera projection still updates naturally.
Classic retains its original 1.25x/0.1-second pop, continuous drift and fade.

## Download and installation

Archive: `build/damage-numbers/releases/r16/damage-numbers-r16.zip`.
Plugin: `build/damage-numbers/open-assets/candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro`.

Close Eden, preserve the current NRO outside its load directory, then replace
`damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro` with this
build. Keep one damage-number plugin installed; existing valid version-2 preferences remain usable. No
save edits or activation flags are needed. Forge is required and not bundled.

The user approved the open-font appearance and selected **20px at 720p as
Number size 1.0** from the comparison. This is 30px at 1080p, 40px at 1440p
and 60px at 4K. Size 1.2 restores the former 24px/36px pair. Critical marker
gaps and compact collision margins scale with the selected preview; the
stamp/hold/exit timing and 30px exit travel remain unchanged.

R16 compresses the accepted RGBA atlas losslessly with DN-LZ1. The artwork,
font, visible size and animation remain identical to r15. The NRO is 152KiB,
down from 256KiB. Decoding writes directly into Forge's existing texture buffer
once per ImGui context. It adds no per-hit work or persistent allocation.
[Compression evidence](COMPRESSION.md) includes bounded host/ARM decoding,
byte comparisons and initialization timing. Release cleanup made no installed
file changes; a fresh read now finds the installed NRO matching r16. No fresh
r16 gameplay run is claimed.

## Asset provenance and notices

The upstream Google Fonts revision is pinned to
`9710da1eacb3be272583c3224dcb70f9da6eadbb`. Open Sans source font SHA-256:
`36643644f318a812aab2d2ed3bb98f8cf0872527f835fe9398d95fe6b9adb878`.
The modified subset is named **Damage Digits Open** and contains digits, space
and a question mark. [Source recipe](fonts/open-source.json) records the exact
axes, font source, license hash and FontTools version. The full
[OFL notice](fonts/OFL.txt) is embedded in the NRO and included in the adjacent
`NOTICES.txt`, together with art and dependency notices. The ZIP contains only
the runtime NRO; redistribute it with `NOTICES.txt`.

The independently authored crit artwork has a separate
[notice](OPEN_ART_LICENSE.txt). The generator operates on the licensed font
and its own geometric parameters without opening Wilds assets. The asset audit
blocks proprietary visual input paths, checks font/license provenance and
compares the actual compiled font, license and sprite buffers with the open
payloads. This documents what is shipped; it is not a blanket legal conclusion
about modding or every dependency.

## Reproduce

Use FontTools 4.66.1 and the pinned upstream font recorded above. The existing
Forge/devkitARM baseline remains required. Wilds installation/extraction is not
needed for this profile.

```bash
python3 scripts/test_damage_numbers_sprite_codec.py
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_capture.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/test_damage_numbers_visual.py
python3 scripts/verify_damage_numbers_compression.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
python3 scripts/package_damage_numbers.py
```

Build and work outputs are isolated under `open-assets/`. Host preview:
`work/damage-numbers/open-visual/preview.png`. Open assets are the default and only profile. The retired `--assets local`
choice is rejected. The packaging script reopens the deterministic
ZIP and verifies every member; it neither installs nor launches anything.
