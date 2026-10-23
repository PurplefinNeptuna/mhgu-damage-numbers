# Damage Numbers

Shows damage numbers at each impact, combining physical and elemental damage.
Choose Wilds, Rise or Classic style.

## Requirements

The Forge loader is supplied separately.

- MHGU 1.4.0 for Nintendo Switch.
- [Fexty's Forge 1.5.1](https://github.com/Fexty12573/forge/releases/tag/v1.5.1).

## Installation

1. Close MHGU.
2. Extract `damage-numbers-r35.zip`.
3. Copy the whole `damage-numbers` folder into your emulator's MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
   - Eden/suyu on Android: import or copy the extracted `damage-numbers` folder
     into the emulator's MHGU mod directory. The folder contains `romfs/`.
4. Enable `damage-numbers` and launch MHGU.

For an existing Switch Forge installation, merge the extracted
`damage-numbers/romfs/` contents into
`sd:/atmosphere/contents/0100770008DD8000/romfs/`, then launch MHGU.

Keep `romfs/nativeNX/plugins/damage_numbers.nro` and its accompanying files:
`OpenSans.ttf`, `wilds-effects.rgba` and `rise-slash.rgba` in
`romfs/nativeNX/plugins/damage-numbers/`.

## Usage

Number placement and size follow the game's visible viewport automatically.
There is no handheld/docked setting to change in the mod.

Numbers appear for hunter, Prowler and Palico attacks, Insect Glaive kinsect
hits, bombs and monster-to-monster hits. Poison ticks, blast explosions and the
mount knockdown also show their damage. These numbers appear at the monster's
position, including in multiplayer when another hunter caused them. Ballista
shots and shots fired by the map itself, such as the rail cannon, show for
every player. Wilds uses outlined digits and glowing stars for
critical hits. Rise has slanted critical slashes and larger weak-point numbers.
Classic has straight critical markers. All sizes scale with screen height.

If the impact position or camera projection is unavailable, numbers appear at
screen centre with the usual animation and spacing. A hit outside the camera
view stays hidden. It appears where it landed if you turn the camera toward it
before the number fades.

Numbers are gold when the physical hitzone value is 45 or higher, and white
when it is below 45 or unknown. Bounced attacks appear grey. Poison damage is
always grey, and blast and mount knockdown damage are always gold.

### Settings

Open **Plugin UI > Damage Numbers** in Forge's overlay. Closing the settings
row leaves numbers running.

| Setting | Default | Effect |
| --- | --- | --- |
| Show damage numbers | On | Turns the display on or off. |
| Show other damage | Off | Shows monster HP loss that no detected damage accounts for, at the monster's position. |
| Style | Wilds | Choose Wilds, Rise or Classic. |
| Number size | 1.0x | Adjust from 0.5x to 3.0x. |
| Duration | 0.9 seconds | Adjust from 0.4 to 2.0 seconds. |

Use **Save settings** to keep your changes for the next session.
**Restore defaults** resets the current session's settings. Press Save settings
afterward to keep those defaults.

Forge creates `damage_numbers.ini` with defaults once it can access MHGU's
save folder. If saving fails, load a player save and retry.

### INI settings

Stop the game before editing `damage_numbers.ini` at the root of MHGU's save
folder for the active user profile.

```ini
[DamageNumbers]
enabled = true
style = wilds
scale = 1
lifetime = 0.9
other = false
```

Set `enabled` and `other` to `true` or `false`, and `style` to lowercase
`wilds`, `classic` or `rise`. Size and duration use the ranges above. Missing or invalid settings
use their individual defaults. Restart the game to load your edits.

## Update and removal

Close MHGU before updating or removing the mod. To update, replace
its `damage-numbers` folder with the one from the new ZIP. To remove it,
disable the package or delete its folder from the emulator's mod directory.
This version uses the same settings file as earlier versions. Show other damage
starts off until you turn it on and save. If updating from r28, r29 or r30 with
all accompanying files already present, you can replace only
`romfs/nativeNX/plugins/damage_numbers.nro`.

On Switch, update the files under
`sd:/atmosphere/contents/0100770008DD8000/romfs/nativeNX/plugins/`.
To remove the mod, delete `damage_numbers.nro` and its `damage-numbers/` asset
folder from that directory.

## Compatibility and known issues

This correction has not yet been confirmed on the affected tablet or Switch.
If numbers remain displaced, report whether the game is in handheld or docked
mode and retain the emulator log.

Rise slashes are drawn over the screen and may appear over foreground scenery.

Map-fired shots have no owner in the game's data, so their numbers show even
when another hunter triggered the weapon.

Show other damage can't tell where its damage came from. Online, it may
include other hunters' attacks. Damage from several sources landing at once can
appear as one number.

If an artwork file is missing or unreadable, numbers continue without that
file's effects. If OpenSans.ttf is unavailable, numbers use Forge's font.
Restore the packaged files and restart the game to restore the full styling.

## Credits

- [MHGU Forge](https://github.com/Fexty12573/forge) by Fexty.
- [Original Forge](https://github.com/jeffi6375/forge) by Jeffi and the Skyline Project.
- [forge-rs](https://github.com/Fexty12573/forge-rs) by Fexty.
- [libnx](https://github.com/switchbrew/libnx) by Hydrosphere Developers.
- [Open Sans](https://github.com/googlefonts/opensans) by The Open Sans Project Authors.
