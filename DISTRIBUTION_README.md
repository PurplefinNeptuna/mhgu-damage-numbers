# Damage Numbers (Rust)

Displays separate numbers at each impact, including total physical and elemental damage, hunter/Prowler and Palico attacks, bombs and monster-to-monster hits. Choose Wilds, Classic or Rise presentation.

## Requirements

MHGU 1.4.0 for Nintendo Switch, and Forge 1.4.1 with the maintained multi-list ImGui renderer fix.

## Installation

1. Close the emulator and disable the C++ Damage Numbers package.
2. Extract `damage-numbers-rs-r1-candidate.zip`.
3. Copy `damage-numbers-rs` into the MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
4. Enable `damage-numbers-rs` and launch MHGU.

Keep both `romfs/nativeNX/plugins/damage_numbers_rs.nro` and the accompanying `damage-numbers-rs/wilds-digits.ttf` asset. To remove the mod, disable or remove its package in the emulator.

## Settings

Open **Plugin UI > Damage numbers (Rust)** in Forge's overlay. Closing the settings row leaves numbers running.

| Control | Default and range |
| --- | --- |
| Show damage numbers | Enabled. |
| Style | Wilds; also Classic and Rise. |
| Number size | 1.0, adjustable from 0.5 to 3.0. |
| Duration | 0.9 seconds, adjustable from 0.4 to 2.0 seconds. |
| Restore defaults | Reset the current settings. |
| Save settings | Save the current settings to MHGU's save folder. |

Wilds uses outlined digits and glowing critical stars; Classic uses straight critical markers; Rise uses slanted critical slashes. At size 1.0 and 1080p, Wilds/Classic use a 36 px base, and Rise uses a 30 px base enlarged to 42 px for weak-point numbers. Sizes follow screen height.

White digits indicate physical hitzone values below 45 or unknown values; gold indicates 45 or higher; bounced attacks appear gray.

Forge creates `damage_numbers_rs.ini` with defaults once the save-data filesystem is available. **Save settings** persists changes. **Restore defaults** changes the session; use Save to persist those defaults. If saving fails, load a player save and retry.

## INI settings

Stop the game before editing `damage_numbers_rs.ini` at the root of MHGU's save folder for the active user profile.

```ini
[DamageNumbers]
enabled = true
style = wilds
scale = 1
lifetime = 0.9
```

`enabled` accepts `true` or `false`; `style` accepts lowercase `wilds`, `classic` or `rise`. Size and duration use the ranges above. Missing or invalid individual settings use their defaults. Restart the game to reload edits.

## Known issues

Use the maintained Forge renderer if opening dropdowns blanks or corrupts the menu. Rise slashes are screen-space effects and may extend across foreground scenery.
