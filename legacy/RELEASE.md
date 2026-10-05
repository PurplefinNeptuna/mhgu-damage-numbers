# Damage numbers r17 compatibility candidate

R17 applies the audited NZCVQ-only gateway restoration for Ryujinx-based
emulators. Nine restore instructions stop writing GE, avoiding the
`APSR_g` translation path in the recorded Ryujinx-Nextendo reference.
The existing eight MHGU 1.4.0 hook sites do not consume GE. NZCVQ, integer/VFP
registers, FPSCR, native stack arguments and return state remain preserved.
[Compatibility notes](RYUJINX_COMPATIBILITY.md) and the
[GE audit](GE_PRESERVATION_AUDIT.md) record the evidence and its limits.

The NRO remains **152KiB**. Every other executable byte matches the audited
r16 build. Open Sans Semibold, the compressed artwork, colors, animation,
spacing, settings and capture behavior remain unchanged. R17 is a validated
candidate awaiting fresh emulator gameplay confirmation.

## Package and installation

`build/damage-numbers/releases/r17/damage-numbers-r17.zip` contains only
`damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro`. README,
`NOTICES.txt`, provenance and `SHA256SUMS` remain beside it. Redistribute the
archive with its notices. The tested patched Forge loader is required and
not bundled; this package does not replace an emulator.

Close the emulator before installing. Preserve the existing NRO outside its
mods directory, then replace it with the r17 NRO. Keep one damage-number
plugin. For a fresh setup, place the extracted `damage-number/` folder in
MHGU's per-title mods directory alongside Forge. On Ryujinx-based emulators,
use the emulator's `mods/contents/0100770008dd8000/` directory. On Eden, use
its title `0100770008DD8000` mod directory. Listing and activation are separate.
The recipes perform no installation, activation, save edit or cache deletion.

## Verification and brief manual check

Passed: 919 host checks, mailbox stress, 23 transaction scenarios, **9,287
ARM gateway checks across 2,025 calls**, **347 compiled capture checks**,
production instruction comparison, real-ImGui equivalence and open-asset audit.
All 16 GE patterns were tested across all eight entries with preserving and
clobbering helpers. No fresh Ryujinx or Eden gameplay result is claimed.

After installing, restart completely, load the save, enter one quest, land a
few hunter/Palico hits and return to the village. Confirm normal controls,
visible numbers and no `APSR_g` failure. Prior combat tests need no full repeat.

The separate Forge menu/popups corruption remains unresolved and documented
in the [renderer handoff](../../docs/FORGE_IMGUI_DAMAGE_CORRUPTION_HANDOFF.md).
R16 rollback and test artifacts remain under
`backups/damage-numbers/r16-before-r17-ge-20261006/`. Only open font/art is
supported; the game-derived edition remains retired.
