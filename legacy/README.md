# Damage numbers

R12 uses the original Wilds digit outlines and crit sprites, replacing the
font/star approximations. Soft texture shading replaces the hard text outline.
R12 is complete and user-accepted on 2026-10-05. The finished local package
uses the exact tested binary. [Release/install notes](RELEASE.md),
[completed visual check](R12_VISUAL_CHECK.md), [test results](TEST_RESULTS.md)
and [runtime history](RUNTIME.md) retain the evidence.

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
| Positive crit | Original pink-red sprite above | Digit-colored line above |
| Negative crit | Original blue sprite below | Digit-colored line below |

Wilds uses Frutiger Neue LT W1G Medium digit outlines from the local game,
normalized to retain the accepted digit height. Digits have a smooth, centered
shadow; gold digits also have a soft warm halo behind their crisp fill. Stars
use the original colored texture with its pale center and bloom. They align to
the actual pixel-rounded glyph bounds and fade with the numbers. Digit color
remains independent of crit polarity. Classic keeps Forge's font, outlined
digits and straight critical lines.

[Font/effect preparation](fonts/README.md) and
[asset provenance](WILDS_ASSETS.json) describe the local sources. The font
rasterizer and Gaussian blur differ from RE Engine's renderer, so an exact
pixel match has not been measured; the user accepted the in-game appearance.

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
python3 scripts/test_damage_numbers_visual.py
python3 scripts/build_damage_numbers_capture.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
```

Host fixtures check policies, measured spacing, animation, codecs and fixed
storage under ASan/UBSan. ARM32 fixtures execute the compiled NRO with explicit
SDK/kernel/font/draw stubs and original HZV/damage routines. They inspect font
registration, draw arguments, colors, marker geometry, fades, burst limits,
source filtering and config recovery. They do not run Forge's GPU renderer.
A separate host harness executes the pinned real ImGui font/texture/draw code,
then composites its actual draw commands into
`work/damage-numbers/host-visual/preview.png` with linear texture sampling.
This checks actual font and sprite output without claiming an Eden GPU test.

R9 passed all 16 LS/bowgun checks, settings persistence, monster-to-monster,
Prowler and bomb display. The reported wide LBG spacing prompted this update.
The r10 working checklist marks style, saved-style restart and spacing checks
complete. The user described the crit symbols as near-perfect and requested
closer font, thickness/alignment and weakspot glow matching. R12 passed the
completed visual checklist and the user confirmed the mod is finished.

## Build and delivery

The local asset recipes pin FontTools 4.66.1, original resource hashes and
converted font/texture hashes. Original game-derived payloads stay outside Git
and the personal candidate includes `LOCAL_ASSETS.txt`. Installed Forge exports
and preserved game/SDK/Better Input hashes are checked during the offline ARM32
build. No capture hooks, CodeMemory objects or TLS slots are added.

Finished ZIP: `build/damage-numbers/releases/r12/damage-numbers-r12.zip`.
Package: `build/damage-numbers/releases/r12/damage-number/`.
Release manifest/checksums: beside that ZIP; acceptance: [RELEASE.json](RELEASE.json).
Original build provenance: `build/damage-numbers/capture/candidate-manifest.json`.
The build manifest retains its historical pre-test status; release acceptance
is recorded separately without rebuilding the tested bytes.
R11 rollback: `backups/damage-numbers/pre-r12/2b55907b6f19b57f04d776db75eba0a209e6c4bccf0e9d2e46ee9f610eae6330/`.
The r10 rollback remains under `backups/damage-numbers/pre-r11/`.
The earlier r9 rollback remains under `backups/damage-numbers/r9/`.
Preparation preserves installed packages, emulator settings and save files.
Installation, game launch, Forge upgrades and pushes require separate scope.
