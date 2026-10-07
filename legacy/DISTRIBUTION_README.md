# Damage Numbers r18

Ready to distribute. The package is local and has not been published.

Open-font damage numbers for Monster Hunter Generations Ultimate 1.4.0
(Nintendo Switch), tested on Eden/Linux with the maintained patched Forge loader.
Forge is required and is not included. Its tested binary fingerprint is recorded
in `release-manifest.json` beside the ZIP; other loader/game versions have not been validated.

Shows individual damage numbers near each impact, including elemental damage.
Your hunter/Prowler and Palicoes are included; other hunters' and their Palicoes'
weapon attacks are hidden. Recognized monster-to-monster and bomb/object damage
is included. Unsupported recurring damage ticks remain excluded.

## Install or update

1. Close Eden.
2. Extract this ZIP. For a fresh install, copy its `damage-number` folder into
   Eden's mod directory for title `0100770008DD8000`, alongside the required Forge
   package, and enable it in Eden. For Ryujinx-based emulators, use the emulator's
   MHGU `mods/contents/0100770008dd8000/` directory alongside Forge.
3. For an update, back up your existing plugin outside Eden's mod directory,
   then replace `damage-number/romfs/nativeNX/plugins/damage_numbers_capture.nro`.
   Keep only one damage-number plugin installed.

There is no extra title-ID folder inside the package and no activation flag.

## Appearance and settings

Open Forge's overlay, then **Plugin UI > Damage numbers**. The compact row is
closed by default; damage rendering continues while it is closed.

Defaults: enabled, Wilds style, Number size **1.0**, Duration **0.9 seconds**.

| Number size | 720p | 1080p |
| --- | ---: | ---: |
| 1.0 (default) | 20 px | 30 px |
| 1.1 | 22 px | 33 px |
| 1.2 | 24 px | 36 px |

Wilds style uses Open Sans Semibold and glowing red/blue critical stars. Numbers
stamp at 1.2× size, settle over 0.2 seconds, hold, then rise and fade during the
last 0.2 seconds. Classic uses Forge's font, outlines and straight critical lines.

Physical HZV below 45 or unknown gives white digits; HZV 45+ gives gold. Confirmed
bounces give gray. Positive critical stars appear above, negative below, with
marker color independent of digit color. Every impact stays separate.

**Save settings** writes your choices beside Better Input in the active user's
save container (`damage_numbers.ini`, with `damage_numbers_backup.ini` recovery).
**Restore defaults** changes the session; save afterward to persist it.
Existing version-2 size values remain multipliers of the new 20px base. This
package does not edit saves or migrate old configurations automatically.

## Files and validation

The NRO uses an OFL-licensed Open Sans digit subset and independently authored
critical-star art. No extracted Monster Hunter Wilds font or texture is bundled.
The installable ZIP contains only the NRO. Full font, art and dependency
notices are combined in `NOTICES.txt` beside the ZIP. Keep this notice file
with the ZIP when redistributing. Font provenance and asset verification are
recorded in the adjacent `release-manifest.json`.

R18 keeps the appearance and behavior settings, replaces diagnostic panels with concise Forge logs, and emits no normal hit or camera dumps. Its 148KiB NRO retains the open artwork and audited NZCVQ-only gateways. The damage overlay continues with settings closed.

Fresh host, compiled ARM32, ABI, import, asset and rendering fixtures cover this payload. No new Eden, Ryujinx or Switch gameplay run was performed. R17's user-confirmed Eden/Ryujinx reports remain historical evidence for that version. The adjacent release manifest records this payload and its evidence; SHA256SUMS covers the delivery.

The tested Forge loader has a known visual corruption issue when its menu and damage popups are visible together. R18 leaves Forge unchanged. Close its menu during gameplay and use the included renderer handoff for the known limitation.
