# Damage Numbers r20

Shows damage numbers near each impact in Monster Hunter Generations Ultimate
1.4.0 for Nintendo Switch. Includes elemental damage, your hunter or Prowler,
your Palicoes, recognized monster-to-monster damage and bomb/object damage.
Other hunters' and their Palicoes' weapon attacks are hidden. Unsupported
recurring damage ticks are excluded.

## Requirements

Forge is required and supplied separately. Use MHGU 1.4.0. Earlier versions
worked on the tested Eden/Linux and Ryujinx installations; r20's updated
appearance has not had a separate gameplay test. Other loader/game versions
and native Switch hardware are unverified.

## Install or update

1. Close the emulator.
2. Extract `damage-numbers-r20.zip`. Copy its `damage-number` folder into the
   emulator's MHGU mod directory, alongside Forge. For Eden, use
   `~/.local/share/eden/load/0100770008DD8000/`; for Ryujinx-based emulators,
   use `mods/contents/0100770008dd8000/` inside your emulator's data directory.
3. When updating, back up the old package outside the emulator's mod directory
   and replace it. Keep only one damage-number plugin installed.
4. Enable the package in the emulator and restart the game.

To remove it, close the emulator, disable or remove `damage-number`, then restart.

## Appearance and settings

Open Forge's overlay, then **Plugin UI > Damage numbers**. Damage rendering
continues while the settings row is closed.

Defaults: enabled, Wilds style, Number size **1.0**, Duration **0.9 seconds**.

| Number size | 720p | 1080p |
| --- | ---: | ---: |
| 1.0 (default) | 20 px | 30 px |
| 1.1 | 22 px | 33 px |
| 1.2 | 24 px | 36 px |

Wilds style uses Open Sans Semibold, a thin near-black outline and glowing
red/blue critical stars, enlarged in r20. Numbers appear slightly larger,
settle, then rise and fade. Classic uses Forge's font, outlines and straight
critical lines. Each impact has its own number.

Physical hitzone values below 45 or unknown give white digits; 45+ gives gold.
Confirmed bounces give gray. Positive critical stars appear above the digits,
negative stars below. Star color is independent of digit color.

**Save settings** keeps your choices for the active player in
`damage_numbers.ini`, with `damage_numbers_backup.ini` for recovery.
**Restore defaults** changes the current session; save afterward to keep them.
Existing version-2 number-size settings remain multipliers of the 20 px base.

## Known issue

Forge's menu can show visual corruption while damage popups are visible.
Close the Forge menu during gameplay.

## Sharing

Keep `NOTICES.txt` with the ZIP when sharing it. The plugin includes an
OFL-licensed Open Sans digit subset and original critical-star art.
