# Damage numbers r14

The open-font edition is the sole supported build. It uses Open Sans Semibold
600 under SIL OFL 1.1, renamed Damage Digits Open, and original procedural
red/blue crit stars. The game-derived edition is retired.

## Package

`build/damage-numbers/releases/r14/damage-numbers-r14.zip` contains one
`damage-number/` mod folder, a plugin at
`romfs/nativeNX/plugins/damage_numbers_capture.nro`, font/art notices, source
provenance, manifest and checksums. Forge is required and not bundled.

NRO SHA-256:
`be263fbebf8fe2057514aabda07c12724bb0706aad2592ee1c3761100603f158`.
The installed open r14 matches this hash. Retirement requires no reinstall.

For a fresh setup, close Eden, extract the ZIP and copy `damage-number/` into
Eden's mod directory for title `0100770008DD8000`. Keep one copy alongside the
tested Forge package and enable it in Eden. For upgrades, preserve the previous
open NRO outside Eden and replace the existing plugin.

Defaults are enabled, Wilds style, size 1.0 and total duration 0.9 seconds.
Settings remain under Forge's compact **Plugin UI > Damage numbers** row and
use `damage_numbers.ini` beside Better Input in the active user's game save.
Existing valid version-2 settings are retained. No activation flags are needed.

## Reproduce

```bash
python3 scripts/build_damage_numbers_capture.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/package_damage_numbers.py
```

Only open assets are supported. The package requires matching build, complete
compiled-fixture and asset-audit NRO fingerprints. It creates a deterministic
ZIP, then reopens and verifies every member. No installation or launch occurs.
The retirement rebuild matches the previously tested 321-check r14 binary;
historical combat coverage is retained in [TEST_RESULTS.md](TEST_RESULTS.md).
See [RETIREMENT.md](RETIREMENT.md) for the archive and support decision.
