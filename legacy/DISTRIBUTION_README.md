# Damage Numbers

Displays a separate damage number at each impact, including elemental damage,
attacks from your hunter or Prowler and Palicoes, bombs and monster-to-monster
hits.

## Requirements

MHGU 1.4.0 for Nintendo Switch and Forge 1.4.1.

## Installation

1. Close the emulator.
2. Extract `damage-numbers-r22-config1.zip`.
3. Copy the `damage-number` folder into your MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
4. Enable `damage-number` in the emulator and launch the game.

To update, replace the existing `damage-number` folder with the extracted folder.
To remove the mod, disable or remove `damage-number` in the emulator's mod directory.

## Settings

Open **Plugin UI > Damage numbers** in Forge's overlay.

Choose Wilds, Classic or Rise style. Wilds uses outlined digits and glowing
critical stars; Classic uses straight critical-hit markers; Rise uses slanted
critical slashes. The default is Wilds, with number size **1.0** and duration
**0.9 seconds**. Adjust size and duration to suit your screen.

White digits indicate physical hitzone values below 45 or unknown values.
Gold digits indicate 45 or higher. Bounced attacks appear gray.

Select **Save settings** to keep your choices in `damage_numbers.ini` beside
Better Input's config in player save storage. **Restore defaults** changes the
current settings; select **Save settings** afterward to keep the defaults.
If saving fails, the current choices remain active for the session. Load a
player save and retry **Save settings**.

## Edit the INI directly

You can change settings without opening Forge's menu.

1. Stop the game. Use your emulator's save-folder option for MHGU to open the
   save folder for the user profile you play with.
2. Open `damage_numbers.ini` in a plain text editor. It belongs in the root of
   that save folder, beside Better Input's config. If it does not exist, create
   it there using the complete example below. Keep the filename exactly
   `damage_numbers.ini`, without an extra `.txt` extension.
3. Edit the values, save the file, then launch MHGU and load your player save.

```ini
[DamageNumbers]
version=2
enabled=1
style=wilds
scale=1.000000
lifetime=0.900000
```

| Setting | Supported values |
| --- | --- |
| `version` | Keep `2`. |
| `enabled` | `1` shows damage numbers; `0` hides them. |
| `style` | `wilds`, `classic` or `rise`, in lowercase. |
| `scale` | Number size from `0.5` to `3.0`; default `1.0`. |
| `lifetime` | Duration in seconds from `0.4` to `2.0`; default `0.9`. |

Keep the section name and all five settings, with each key appearing once.
Use a dot for decimals and no more than six decimal places. Save as plain
UTF-8 text without a byte-order mark. Invalid or incomplete files load the
mod's defaults. Changes are read when you next launch the game and load your
player save.

## Known issue

Opening a dropdown or displaying gameplay overlays can make Forge's entire
ImGui window blank. If this happens, click random places inside that ImGui
window until the menu reappears.
