# Damage numbers r12

Complete and accepted in-game on 2026-10-05. MHGU 1.4.0 Switch, Eden/Linux,
using the tested `forge-patched` baseline. The installed r12 is already the
same binary as this package; no reinstall is needed on the current setup.

## Package

`build/damage-numbers/releases/r12/damage-numbers-r12.zip` contains the
`damage-number/` mod folder, with one plugin under
`romfs/nativeNX/plugins/damage_numbers_capture.nro`, notes and checksums.
Forge is a dependency and is not bundled. No activation flags or config files
are supplied. This is the personal local build using the original Wilds digit
outlines and crit sprites; its asset notice and provenance are retained.

For a fresh setup, close Eden, extract the ZIP and copy `damage-number/` into
Eden's mod directory for title `0100770008DD8000`. Keep one copy of the damage
plugin, alongside the tested Forge package. Enable the package in Eden.
For an upgrade, preserve the old plugin outside Eden, then replace its existing
NRO with the packaged NRO. Do not add a second differently named mod folder.

Defaults are enabled, Wilds style, size 1.0 and duration 0.9 seconds. Settings
are in Forge's compact **Plugin UI → Damage numbers** row. Save settings writes
`damage_numbers.ini` beside Better Input in the active user's game save.
Existing valid version-2 settings are retained; old versions are ignored.
No save or emulator-setting changes are needed for installation.

Tested binary SHA-256:
`1a851eb2947a88ec3bb06685c5b852d2324b742ff0420d56fee13a1e31bf7b3f`.

## Recreate the package

```bash
python3 scripts/package_damage_numbers.py
```

The recipe freezes the accepted candidate after checking its hash against the
ARM32 fixture report and release record. It produces a deterministic ZIP,
reopens it and verifies every member. It does not rebuild or install anything.
Completed coverage: [TEST_RESULTS.md](TEST_RESULTS.md),
[R12_VISUAL_CHECK.md](R12_VISUAL_CHECK.md) and [RELEASE.json](RELEASE.json).
