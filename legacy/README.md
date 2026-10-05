# Damage numbers

R11 refines the Wilds font, crit star thickness/alignment and weakspot glow
against the new comparison screenshots. The candidate is staged; installation
is separate. Use the short [visual check](R11_VISUAL_CHECK.md). The [r9 results](TEST_RESULTS.md) and
[runtime history](RUNTIME.md) retain the completed combat checks.

Target: MHGU 1.4.0 Switch, Eden/Linux, and the tested patched Forge baseline.
Capture hooks, damage calculations and ownership bindings are unchanged.

## Display behavior

- Show positive damage dealt to monsters by your hunter/Prowler and Palicoes.
  Hide other hunters' and their Palicoes' weapon attacks.
- Show recognized monster-to-monster and bomb/object impacts whenever the
  master switch is enabled. Generic hunter-owned subtype-8 objects include
  remote owners; Palico/Prowler support bombs remain owned attacks.
- Preserve individual impacts, including same-frame hits, and include elemental
  damage in each finalized native total. Physical HZV alone selects digit color.
- Confirmed bounced hits always use gray. Ambiguous feedback retains HZV color.
  Exclude healing, zero damage, status buildup and unsupported recurring ticks.

| Condition | Wilds | Classic |
| --- | --- | --- |
| Physical HZV below 45 or unknown | Warm white `#F4F1EC` | White `#FFFFFF` |
| Physical HZV at least 45 | Gold `#F2C45B` | Orange-yellow `#FFBF3F` |
| Confirmed bounce | Gray `#9A9A9A` | Gray `#9A9A9A` |
| Positive crit | Pink-red `#F1689B` star above | Digit-colored line above |
| Negative crit | Blue `#68B9FF` star below | Digit-colored line below |

Wilds uses full-width Roboto Bold 700 as a visual approximation of the
references, retaining the accepted digit height. Gold digits have a symmetric
warm halo and crisp center, without a black shadow. White/gray digits keep a
small dark shadow. Stars have a slightly fuller center, a brighter pale core
and independent red/blue glow. They align to the actual pixel-rounded glyph
bounds; their colors are independent of digit color.
Classic keeps Forge's font, outlined digits and straight critical lines.
Font provenance, reproducible preparation and license: [fonts/README.md](fonts/README.md).

Both styles preserve the accepted impact anchor, visible digit height, size,
pop, upward drift, duration and fade. Measured glyph bounds and solid markers
reserve the maximum initial-pop dimensions. Placement checks 17 nearby staggered
positions within 1.6 font sizes horizontally and one vertically, minimizing
solid overlap before displacement. Glow may overlap. Offsets stay cached until
expiry; crowded bursts may overlap instead of growing a tall stack. Camera,
context, viewport, style and size changes reset placement caches.

Defaults: enabled, Wilds, scale 1.0, duration 0.9 seconds; 24-pixel font size at
720p, 1.25 initial pop, 0.1-second pop settlement, 30-pixel lifetime drift and
0.25-second fade. Invalid, behind-camera and off-screen projections are hidden.
Limits: 256 queued events, 128 visible numbers and 512 recent identities.
Overflow drops oldest entries without adding their values together.

## Settings

The compact **Damage numbers** tree under Forge's **Plugin UI** starts closed.
It contains enable, Style, Number size, Duration, Save settings, Restore defaults
and nested diagnostics. Rendering continues while the panel is closed.

Settings live beside Better Input in `savedata:/damage_numbers.ini`; the
transactional recovery copy is `savedata:/damage_numbers_backup.ini`. Version 2
requires only `version`, `enabled`, `style`, `scale` and `lifetime`, within 255
bytes. Legacy versions and obsolete source/bounce keys are rejected. Old primary,
backup and SD files preserve no preferences and are not migrated or deleted.
**Save settings** explicitly writes version 2; Restore defaults changes the
session until saved. Game payloads and Better Input's INI are preserved.

Loading retries at most once per second if the player-save mount is unavailable.
Manual edits cancel deferred loading. Filesystem operations use Forge's counted
save mount and the existing SDK scoped abort guard.

Gray bounce attribution still requires a unique matching embedded hunter attack,
same native thread/counter, unchanged flags and crit outcome, and feedback within
100 ms. New collisions invalidate attribution. Projectile effect zero alone
does not establish a bounce. Other environmental families remain unsupported.

## Verification

```bash
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_capture.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
```

Host fixtures check policies, measured spacing, animation, codecs and fixed
storage under ASan/UBSan. ARM32 fixtures execute the compiled NRO with explicit
SDK/kernel/font/draw stubs and original HZV/damage routines. They inspect font
registration, draw arguments, colors, marker geometry, fades, burst limits,
source filtering and config recovery. They do not run Forge's GPU renderer.

R9 passed all 16 LS/bowgun checks, settings persistence, monster-to-monster,
Prowler and bomb display. The reported wide LBG spacing prompted this update.
The r10 working checklist marks style, saved-style restart and spacing checks
complete. The user described the crit symbols as near-perfect and requested
closer font, thickness/alignment and weakspot glow matching. Only those r11
visual refinements need another look.

## Build and delivery

The font preparation recipe pins Google Fonts and FontTools 4.66.1, records
source/license/subset hashes and embeds the static font and OFL notice in the
NRO. The package also includes the license. Installed Forge exports and the
preserved game/SDK/Better Input hashes are checked during the offline ARM32 build.
No new capture hooks, CodeMemory objects or TLS slots are added.

Candidate: `build/damage-numbers/capture/candidate-package/`.
Hashes and sources: `build/damage-numbers/capture/candidate-manifest.json`.
R10 rollback: `backups/damage-numbers/pre-r11/4030d8c4d564b3f61f4b7c3a4b23421a30874be032cbf4d54ebdcf164a05bf6b/`.
The earlier r9 rollback remains under `backups/damage-numbers/r9/`.
Preparation preserves installed packages, emulator settings and save files.
Installation, game launch, Forge upgrades and pushes require separate scope.
