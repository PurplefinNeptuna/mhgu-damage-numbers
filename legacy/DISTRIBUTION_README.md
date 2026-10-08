# Damage Numbers

Displays a separate damage number at each impact, including elemental damage,
attacks from your hunter or Prowler and Palicoes, bombs and monster-to-monster
hits.

## Requirements

MHGU 1.4.0 for Nintendo Switch and Forge 1.4.1. Use Forge 1.4.1-render1 for
visible dropdowns and menus while gameplay overlays are active.

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

## Compatibility

Older Forge loaders can blank or corrupt the menu while dropdowns or damage
popups are visible. Use Forge 1.4.1-render1 for these menus and overlays together.
