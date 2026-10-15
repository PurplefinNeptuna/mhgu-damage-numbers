# Damage Numbers

Shows damage numbers at each impact, combining physical and elemental damage.
Works with hunter, Prowler and Palico attacks, bombs and monster-to-monster hits.
Choose Wilds, Rise or Classic style.

## Requirements

MHGU 1.4.0 for Nintendo Switch and a compatible Forge 1.5.1 loader.

## Installation

1. Close the emulator. Replace the previous Damage Numbers package when updating.
2. Extract `damage-numbers-r26.zip`.
3. Copy `damage-numbers` into the MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
4. Enable `damage-numbers` and launch MHGU.

Keep `romfs/nativeNX/plugins/damage_numbers.nro` and all three accompanying files in `romfs/nativeNX/plugins/damage-numbers/`: `OpenSans.ttf`, `wilds-effects.rgba` and `rise-slash.rgba`. To remove the mod, disable or remove its package in the emulator.

## Settings

Open **Plugin UI > Damage Numbers** in Forge's overlay. Closing the settings row leaves numbers running.

| Control | Default and range |
| --- | --- |
| Show damage numbers | Enabled. |
| Style | Wilds by default. Rise and Classic are also available. |
| Number size | 1.0, adjustable from 0.5 to 3.0. |
| Duration | 0.9 seconds, adjustable from 0.4 to 2.0 seconds. |
| Restore defaults | Reset the current settings. |
| Save settings | Save the current settings to MHGU's save folder. |

Wilds uses outlined digits and glowing stars for critical hits. Rise has slanted
critical slashes, while Classic has straight critical markers. Rise makes
weak-point numbers larger. All sizes scale with screen height.

Numbers are gold when the physical hitzone value is 45 or higher, and white
when it is below 45 or unknown. Bounced attacks appear gray.

Forge creates `damage_numbers.ini` with defaults once it can access MHGU's save
folder. Use **Save settings** to keep your changes for the next session.
**Restore defaults** resets the current session's settings. Press Save settings
afterward to keep those defaults. If saving fails, load a player save and retry.

## INI settings

Stop the game before editing `damage_numbers.ini` at the root of MHGU's save folder for the active user profile.

```ini
[DamageNumbers]
enabled = true
style = wilds
scale = 1
lifetime = 0.9
```

Set `enabled` to `true` or `false` and `style` to lowercase `wilds`, `classic`
or `rise`. Size and duration use the ranges above. Missing or invalid settings
use their individual defaults. Restart the game to load your edits.

## Known issues

Rise slashes are drawn over the screen and may appear over foreground scenery.

If an artwork file is missing or unreadable, numbers continue without that file's effects. If OpenSans.ttf is unavailable, numbers use Forge's font. Restore the packaged files and restart the game to restore the full styling.
