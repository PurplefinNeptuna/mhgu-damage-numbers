# Damage Numbers

Displays a separate damage number at each impact, including elemental damage,
attacks from your hunter or Prowler and Palicoes, bombs, and monster-to-monster
hits.

## Requirements

MHGU 1.4.0 for Nintendo Switch and Forge.

## Installation

1. Close the emulator.
2. Extract `damage-numbers-r20.zip`.
3. Copy the `damage-number` folder into your MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
4. Enable `damage-number` in the emulator and launch the game.

## Settings

Open **Plugin UI > Damage numbers** in Forge's overlay.

The default appearance is Wilds style, with number size **1.0** and duration
**0.9 seconds**. Adjust the size and duration to suit your screen, or select
Classic style for outlined numbers with straight critical-hit markers.

Wilds style uses outlined digits and glowing critical stars. Numbers settle
into place, then rise and fade. Positive critical stars appear above the number;
negative stars appear below it.

White digits indicate physical hitzone values below 45 or unknown values.
Gold digits indicate 45 or higher. Bounced attacks appear gray.

Select **Save settings** to keep your choices. **Restore defaults** resets the
current settings; select **Save settings** afterward to keep the defaults.

## Known issue

Forge's menu can show visual corruption while damage popups are visible.
Close the overlay menu when playing.
