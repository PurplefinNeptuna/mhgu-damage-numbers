# Damage numbers r16

R16 compresses the approved open artwork losslessly, reducing the NRO from
256KiB to **152KiB**. The Open Sans Semibold font, texture pixels, rendering,
20px default, animation and settings remain identical to r15. The decoder runs
once per ImGui context directly into the existing texture buffer; it adds no
per-hit work or persistent allocation. [Compression evidence](COMPRESSION.md)
records bounds checks, exact comparisons and host initialization timings.

## Package and installation

`build/damage-numbers/releases/r16/damage-numbers-r16.zip` contains only
`damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro`. README,
`NOTICES.txt`, release provenance and `SHA256SUMS` stay beside the ZIP, matching
the clean modern-control layout. Redistribute the archive with its notices.
The tested patched Forge loader is required and not bundled. Target: MHGU
1.4.0, Switch, Eden/Linux.

Close Eden before installing. Preserve the current NRO outside Eden's load
directory, then replace it with the r16 NRO. Keep one damage-number plugin.
For a fresh setup, copy the extracted `damage-number/` folder into Eden's title
`0100770008DD8000` mod directory alongside Forge, and enable it in Eden.
The packager performs no installation. A fresh read during release cleanup
finds the installed NRO matching r16; this is a file-hash check, not new
gameplay confirmation.

## Defaults and evidence

Enabled, Wilds style, Number size **1.0 = 20px at 720p / 30px at 1080p**,
Duration 0.9s. Wilds stamps at 1.2×, settles over 0.2s, holds, then rises and
fades during the final 0.2s. Existing version-2 preferences remain usable.
No configuration or save migration is needed.

Passed: 919 host core/presentation checks, mailbox stress, 23 transaction
scenarios, **338 compiled ARM32 checks**, bounded codec tests under ASan/UBSan,
real-ImGui byte equivalence and open-asset provenance audit. The deterministic
ZIP is reopened and checked member by member. Prior user-confirmed combat and
appearance remain the gameplay evidence; no fresh r16 in-game run or live FPS
measurement is claimed.

The tested Forge loader's corruption when its menu and damage popups appear
together remains unresolved. This asset compression update leaves that loader
unchanged. [Repair handoff](../../docs/FORGE_IMGUI_DAMAGE_CORRUPTION_HANDOFF.md).

Only open font/art is supported. [Retirement record](RETIREMENT.md) preserves
the private edition's history and quarantine locations. NRO and ZIP hashes,
font provenance and verification reports are in the adjacent release manifest.
