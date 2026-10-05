# Damage numbers r15 — finished, ready for distribution

Open Sans Semibold 600 under SIL OFL 1.1, renamed Damage Digits Open, and
independently authored procedural red/blue critical stars. The game-derived
edition remains retired.

## Package and installation

`build/damage-numbers/releases/r15/damage-numbers-r15.zip` contains one
`damage-number/` mod folder with only
`romfs/nativeNX/plugins/damage_numbers_capture.nro`, matching modern-control's
runtime-only archive layout. README, combined `NOTICES.txt`, source/audit release
manifest and SHA-256 checksums remain beside the ZIP. Keep `NOTICES.txt` with
the archive when redistributing. The tested patched
Forge loader is required and is not bundled. Target: MHGU 1.4.0, Switch, Eden/Linux.

Close Eden before installing. For a fresh setup, extract the ZIP and copy
`damage-number/` into Eden's title `0100770008DD8000` mod directory, alongside
Forge, and enable it in Eden. For upgrades, preserve the previous open NRO
outside Eden's load directory, then replace the existing NRO. Keep one copy.

## Final defaults

Enabled, Wilds style, Number size **1.0 = 20px at 720p**, total duration 0.9s.
This scales to **30px at 1080p**, 40px at 1440p and 60px at 4K. Wilds stamps at
1.2×, settles over 0.2s, holds, then rises and fades during the final 0.2s.
The crit stars, colors, shading and compact placement use the approved open style.

Preferences remain version 2 beside Better Input in the active user's game save.
Existing size values multiply the new base; size 1.2 yields the former 24px/36px.
No migration, save edit or activation flag is needed. The compact Forge panel
retains enable, Style, Number size, Duration, Save settings and Restore defaults.

## Validation and status

The user approved the open appearance and selected this final size from the
real-renderer comparison. Combat, source filtering, persistence and Prowler/bomb
confirmations remain recorded in [TEST_RESULTS.md](TEST_RESULTS.md). The final
size update is covered by host, compiled ARM32, real-ImGui and provenance checks.
This release makes no claim of a fresh r15 gameplay run. Installation has not
been performed by the packager; rollback material is preserved. A fresh read
now finds the installed plugin matching r15.

The packager requires matching build/fixture/audit hashes, checks the new
20px/30px size assertions, then reopens and verifies every deterministic ZIP
member. NRO and archive hashes are in the generated release manifest/checksums.

```bash
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_capture.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/test_damage_numbers_visual.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
python3 scripts/package_damage_numbers.py
```

Only open font/art is supported. [Retirement record](RETIREMENT.md) preserves
the private edition's history and quarantine locations.
