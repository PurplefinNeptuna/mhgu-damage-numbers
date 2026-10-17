# Damage Numbers

Shows damage numbers at each impact, combining physical and elemental damage.
This diagnostic candidate adds screen markers and bounded projection logs.

## Requirements

This is a temporary diagnostic build for misplaced numbers. Use the same Forge
loader as your working r28 setup. The loader is supplied separately.

- MHGU 1.4.0 for Nintendo Switch.
- [Fexty's Forge 1.5.1](https://github.com/Fexty12573/forge/releases/tag/v1.5.1).

## Installation

1. Close the emulator.
2. Extract `damage-numbers-r29-diagnostics-candidate.zip`.
3. Copy the whole `damage-numbers` folder into your emulator's MHGU mod directory:
   - Eden: `~/.local/share/eden/load/0100770008DD8000/`
   - Ryujinx: `mods/contents/0100770008dd8000/` inside its data directory.
   - Eden/suyu on Android: import or copy the extracted `damage-numbers` folder
     into the emulator's MHGU mod directory. The folder contains `romfs/`.
4. Enable `damage-numbers` and launch MHGU.

Keep `romfs/nativeNX/plugins/damage_numbers.nro` and its accompanying files:
`OpenSans.ttf`, `wilds-effects.rgba` and `rise-slash.rgba` in
`romfs/nativeNX/plugins/damage-numbers/`.

## Usage

### Collect a diagnostic sample

1. Start a fresh game session with this candidate replacing r28.
2. Enter a quest, hit a monster a few times, then rotate the camera and repeat.
   Keep the sample under 32 hits.
3. Record a short video showing the impact, numbers and reference crosses.
4. Stop the game and export its emulator log before another launch overwrites it.
   Return that log with the video and your Forge version or download link.

The Cyan cross labelled `DN diag centre` marks the calculated screen centre.
Magenta crosses mark projected hits. Yellow crosses show centre fallback.
Markers bypass the number animation and overlap spacing.

Search the exported log for `[DN-DIAG]`. There are at most 16 distinct camera
snapshots, 32 hit snapshots and 4 display changes per launch. Camera matrices
are included again with each sampled hit. Restart the game for a new sample.

These diagnostic messages deliberately use ERROR severity so Forge's default
filter includes them. They do not report a game crash. Stock Forge sends them
to the emulator log through debug output; this plugin creates no separate file.
If `[DN-DIAG]` is absent, retain the full log so loading and filtering can be checked.

On desktop Eden the log is `~/.local/share/eden/log/eden_log.txt`.
Android's storage location depends on the emulator package and selected user
directory, so use its log export or share function.

Numbers appear for hunter, Prowler and Palico attacks, bombs and
monster-to-monster hits. Wilds uses outlined digits and glowing stars for
critical hits. Rise has slanted critical slashes and larger weak-point numbers.
Classic has straight critical markers. All sizes scale with screen height.

If the impact position or camera projection is unavailable, numbers appear at
screen centre with the usual animation and spacing. Hits outside the camera
view also use this fallback.

Numbers are gold when the physical hitzone value is 45 or higher, and white
when it is below 45 or unknown. Bounced attacks appear grey.

### Settings

Open **Plugin UI > Damage Numbers** in Forge's overlay. Closing the settings
row leaves numbers running.

| Setting | Default | Effect |
| --- | --- | --- |
| Show damage numbers | On | Turns the display on or off. |
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
```

Set `enabled` to `true` or `false` and `style` to lowercase `wilds`, `classic`
or `rise`. Size and duration use the ranges above. Missing or invalid settings
use their individual defaults. Restart the game to load your edits.

## Update and removal

Close the emulator before updating or removing the mod. To update, replace
its `damage-numbers` folder with the one from the new ZIP. To remove it,
disable the package or delete its folder from the emulator's mod directory.
After collecting the sample, restore r28 to remove the diagnostic markers and
logging. The diagnostic build uses the same settings file.

## Compatibility and known issues

This build collects evidence. It retains the current camera-selection and
placement rules, including the reported displacement. It has not yet been
tested on the affected tablet or Switch. Logging may briefly affect frame time.

Rise slashes are drawn over the screen and may appear over foreground scenery.

If an artwork file is missing or unreadable, numbers continue without that
file's effects. If OpenSans.ttf is unavailable, numbers use Forge's font.
Restore the packaged files and restart the game to restore the full styling.
