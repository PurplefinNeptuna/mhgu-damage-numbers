# Damage Numbers r16

Finished open-font release for Monster Hunter Generations Ultimate 1.4.0
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
   package, and enable it in Eden.
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

R16 losslessly compresses the same artwork, shrinking the NRO from 256KiB to
152KiB. It decodes once per UI context directly into the existing texture.
Appearance, settings, hit rendering and GPU texture dimensions remain unchanged.
Host, compiled ARM32 and real-ImGui fixtures verify the decoded bytes and
rendering. Prior user-confirmed gameplay remains the manual evidence; no fresh
r16 in-game run is claimed. `release-manifest.json` records the exact NRO hash and
compiled check count. The adjacent `SHA256SUMS` covers the ZIP, plugin, README,
notices and release manifest. Only the `damage-number` folder belongs in Eden.

The tested Forge loader has a known visual corruption issue when its menu and
damage popups are visible together. This compression update leaves that issue
unchanged.
