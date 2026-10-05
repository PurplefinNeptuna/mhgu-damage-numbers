# Damage numbers r12: alternate open assets

This alternate build uses **Roboto Semibold 600**, a renamed digit subset under
SIL Open Font License 1.1, and independently authored procedural red/blue crit
stars. It bundles no extracted Wilds font or texture. The artwork is generated
from a generic four-point polygon, Gaussian bloom and a pale highlight; it is
not traced from the Wilds sprite. Its shape and the font differ from the
original-asset r12 edition.

Soft number shading, gold halo, positive/negative marker placement, individual
hit capture, compact spacing, animation, source filtering, Classic mode,
version-2 settings and the compact Forge panel use the existing implementation.
The Style selector keeps its saved Wilds/Classic values; here Wilds selects the
open-asset appearance. The accepted personal r12 remains unchanged.

## Download and switch editions

Archive: `build/damage-numbers/releases/r12-open-assets/damage-numbers-r12-open-assets.zip`.
Plugin: `build/damage-numbers/open-assets/candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro`.

Close Eden, preserve the current NRO outside its load directory, then replace
`damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro` with this
variant. Keep one damage-number plugin installed. Both editions share the same
configuration paths; existing valid version-2 preferences remain usable. No
save edits or activation flags are needed. Forge is required and not bundled.

The original r12 passed the full gameplay checks. This alternate changes only
embedded visual payloads. Build, compiled-fixture, provenance and real-ImGui
preview results are recorded in the generated manifests. Its new font and
stars still need a brief in-game look; no full combat or persistence test repeat
is requested.

## Asset provenance and notices

The upstream Roboto revision is pinned to
`69d0549e8fc7ad72e1531734c81237be401ec4fd`. Source font SHA-256:
`d7598e12c5dbef095ff8272cfc55da0250bd07fbdecbac8a530b9b277872a134`.
The modified subset is named **Damage Digits Open** and contains digits, space
and a question mark. [Source recipe](fonts/open-source.json) records the exact
axes, font source, license hash and FontTools version. The full
[OFL notice](fonts/OFL.txt) is embedded and included in the ZIP.

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
python3 scripts/build_damage_numbers_capture.py --assets open
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/test_damage_numbers_visual.py --assets open
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py --assets open
python3 scripts/package_damage_numbers_open.py
```

Build and work outputs are isolated under `open-assets/`. Host preview:
`work/damage-numbers/open-visual/preview.png`. The default `--assets local`
retains the personal r12 recipe. The packaging script reopens the deterministic
ZIP and verifies every member; it neither installs nor launches anything.
