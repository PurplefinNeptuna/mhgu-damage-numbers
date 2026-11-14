# Damage Numbers

Shows damage numbers at each impact, combining physical and elemental damage.
Choose Wilds, Rise or Classic style.

## Requirements

The Forge loader is supplied separately.

- MHGU 1.4.0 for Nintendo Switch.
- [Fexty's Forge 1.5.1](https://github.com/Fexty12573/forge/releases/tag/v1.5.1).
- Optional: Purple-lib r17 or later, to change the settings in the game's Options.

## Installation

1. Close MHGU.
2. Extract `damage-numbers-r48.zip`.
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

You can change the settings in two places. Both change the same settings and
save them to the same file.

With Purple-lib installed, open **Options > Mod Options** in the game and choose
**Damage Numbers** from the list of mods (with Purple-lib r19 or older, press L
or R until its page shows). A change applies at once and is saved when you close
the window. The help button shown in the button guide opens a topic for each
setting.

You can also open **Plugin UI > Damage Numbers** in Forge's overlay. Closing the
settings row leaves numbers running. Use **Save settings** to keep your changes
for the next session. **Restore defaults** resets the current session's
settings, so press Save settings afterward to keep those defaults.

Forge opens its overlay with the Insert key on a keyboard; `[menu] key` in Forge's
`config.ini` changes the key. On Ryujinx, turn on Direct Keyboard Access in the input
settings first. On MHGU Recomp, press Insert or choose **Open the Forge menu** on the
Mods tab of its settings menu (F1, or Plus and Minus on a controller).

| Setting | Default | Effect |
| --- | --- | --- |
| Show damage numbers | On | Turns the display on or off. |
| Style | Wilds | Choose Wilds, Rise or Classic. |
| Number size | 100% | Adjust from 50% to 300%. The overlay slider shows 0.5 to 3.0. |
| Duration | 0.9 seconds | Adjust from 0.4 to 2.0 seconds. |
| Spread | 80% | How far apart numbers move when they would overlap, from 0% (stacked on the hit) to 200%. The overlay slider shows 0 to 2.0. |
| Preview mode | Single | Mod Options only. Single shows one hit at each spot, Burst five hits 0.15 seconds apart, and Multi five hits at once. Not saved. |
| Random hit types | Off | Mod Options only. On gives every preview hit a random type. Not saved. |

On the Mod Options page, Number size and Spread change in steps of 10% and
Duration in steps of 0.1 seconds. The overlay sliders also allow the values in
between.

While the Mod Options page is open, a frame on the left of the screen repeats
example hits at three spots with your current settings, even when Show damage
numbers is off. The left spot shows a weak-point hit with a positive critical,
the middle a normal hit with a negative critical, and the right a bounce. In a
quest the spots stay where they are in the world, so they move like real hits
when you walk or turn the camera. In the village they stay at their places in
the frame. With Purple-lib r19 or later the frame hides while the page's help is
open.

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
spread = 0.8
```

Set `enabled` to `true` or `false`, and `style` to lowercase
`wilds`, `classic` or `rise`. Size, duration and spread use the ranges above. Missing or invalid settings
use their individual defaults. Preview mode and Random hit types are not in
the file. Restart the game to load your edits.

## Update and removal

Close MHGU before updating or removing the mod. To update, replace
its `damage-numbers` folder with the one from the new ZIP. To remove it,
disable the package or delete its folder from the emulator's mod directory.
This version uses the same settings file as earlier versions. Show other damage
is gone, and its line in the file is ignored. Spread starts at 80%, so numbers sit
a little closer together than before. Set it to 100% for the old spacing.

On Switch, update the files under
`sd:/atmosphere/contents/0100770008DD8000/romfs/nativeNX/plugins/`.
To remove the mod, delete `damage_numbers.nro` and its `damage-numbers/` asset
folder from that directory.

## Compatibility and known issues

Number placement on tablets and on Switch is unconfirmed. If numbers look
displaced, note whether the game is in handheld or docked mode and keep the
emulator log.

Rise slashes are drawn over the screen and may appear over foreground scenery.

Map-fired shots have no owner in the game's data, so their numbers show even
when another hunter triggered the weapon.

If an artwork file is missing or unreadable, numbers continue without that
file's effects. If OpenSans.ttf is unavailable, numbers use Forge's font.
Put the packaged files back and restart the game to get the full styling.

It can load alongside plugins that hook most of the same game functions.

On MHGURecomp the mod can't adjust its drawn text, so Wilds numbers lose their text outline and Rise numbers
lose their outline and wider digits. Classic numbers are unaffected.

## Credits

- [MHGU Forge](https://github.com/Fexty12573/forge) by Fexty.
- [Original Forge](https://github.com/jeffi6375/forge) by Jeffi and the Skyline Project.
- [forge-rs](https://github.com/Fexty12573/forge-rs) by Fexty.
- [libnx](https://github.com/switchbrew/libnx) by Hydrosphere Developers.
- [Open Sans](https://github.com/googlefonts/opensans) by The Open Sans Project Authors.
