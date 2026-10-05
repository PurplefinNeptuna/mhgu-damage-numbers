# Open-only damage numbers — 2026-10-05

The user selected the open-font edition as the sole supported version. Default
build, visual verification and packaging commands now use Open Sans Semibold
600 and independently authored procedural critical stars. There is no private
asset profile, extraction script or game-font conversion entry point.

The Wilds stamp/hold/exit animation is now the sole Wilds animation path.
Classic appearance and animation, capture hooks, settings and source filtering
remain unchanged. Historical combat results are retained in
[TEST_RESULTS.md](TEST_RESULTS.md); retirement requires no repeat of those tests.

## Current package

- Release: `build/damage-numbers/releases/r14/damage-numbers-r14.zip`.
- Candidate: `build/damage-numbers/open-assets/candidate-package/romfs/nativeNX/plugins/damage_numbers_capture.nro`.
- Installed plugin: `~/.local/share/eden/load/0100770008DD8000/damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro`.
- NRO SHA-256: `be263fbebf8fe2057514aabda07c12724bb0706aad2592ee1c3761100603f158`.

The installed plugin already matches the open r14 NRO. Retirement does not
replace installed files, launch Eden, alter preferences or touch saves. The
rebuild retained this hash; existing compiled fixtures cover these same
bytes. Fresh host checks, the asset audit and real-ImGui preview passed on
the default path.

## Retired material

Private payloads are quarantined outside active build/work directories at
`backups/damage-numbers/retired-game-assets-20261005/`. Its
`RETIREMENT_MANIFEST.json` records original paths, sizes and SHA-256 hashes for
all 160 archived files. This preserves rollback material under project rules;
the archive is not a supported build or release destination.

Archived roots: `build/damage-numbers/capture`,
`build/damage-numbers/releases/r12`, `work/damage-numbers/native-capture`,
`work/damage-numbers/wilds-assets`, `work/damage-numbers/host-visual`, and
`work/damage-numbers/font-source/frutiger-digits.ttf`.

Historical provenance and notices moved to [history/r12/](history/r12/README.md).
Removed scripts: `extract_wilds_damage_assets.py`,
`prepare_damage_numbers_wilds_source.py`, and the redundant
`package_damage_numbers_open.py`. Use `package_damage_numbers.py` instead.
`--assets local` is rejected; `--assets open` remains optional.

The original Wilds installation and shared research tools remain untouched.
Open source font candidates and old open-edition rollback packages are preserved.

## Reproduce

```bash
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_capture.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/test_damage_numbers_visual.py
python3 scripts/package_damage_numbers.py
```

The packager requires matching build, compiled-fixture and asset-audit hashes,
includes OFL/MIT notices, and reopens every ZIP member. It refuses to overwrite
a differing existing release. No installation is performed.
