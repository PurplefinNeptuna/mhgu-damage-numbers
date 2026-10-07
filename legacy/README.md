# Damage numbers

## Build this mod

Run `./build.sh doctor`, `./build.sh prepare`, `./build.sh build`,
`./build.sh test --host`, `./build.sh test`, `./build.sh verify`, and
`./build.sh package` from this folder. The entry point also works from any other
current directory. `package --release` requires the approved payload and full
validation. `build --diagnostics` produces a development build only.

Dependencies are pinned in `dependencies.lock.json`; Python requirements are
pinned in `requirements.txt`. Downloads occur only with `prepare --fetch`.
Copy `build.local.example.json` to ignored `build.local.json` and configure
copies of your MHGU 1.4.0 main/SDK, Forge loader, and required resource inputs.
No installed emulator package or emulator settings file is needed.

The coordinating workspace retains root `build/` and `work/` outputs. An
independent checkout uses its own ignored `build/` and `work/` directories.
The author keeps all rights to owned code; third-party and font licenses remain separate.


Distribution: [READY_TO_DISTRIBUTE](DISTRIBUTION.json); governed by the
[workspace standard](../../docs/DISTRIBUTION_STANDARD.md).

R17 applies the audited NZCVQ-only gateway fix for Ryujinx-based emulators.
Ready to distribute, as directed by the user on 2026-10-06. The package is local
and has not been published. Static validation passed. The user confirmed r17 working on Eden and
Ryujinx-based emulators on 2026-10-06; installed payloads match r17. The sole supported edition uses
**Open Sans Semibold 600** digits under
SIL OFL 1.1 and independently authored procedural crit stars. The game-derived
font/texture edition was retired on 2026-10-05. [Release/install notes](RELEASE.md),
[open asset provenance](OPEN_ASSETS.md), [retirement record](RETIREMENT.md),
[test results](TEST_RESULTS.md) and [runtime history](RUNTIME.md) retain the evidence.

Target: MHGU 1.4.0 Switch, Eden/Linux, and the tested patched Forge baseline.
Hook sites, damage calculations and ownership bindings are unchanged.
The nine gateway restore instructions now write NZCVQ while leaving GE as set
by helpers. [GE audit](GE_PRESERVATION_AUDIT.md) establishes why this is safe at
the current hook sites; [r17 notes](RYUJINX_COMPATIBILITY.md) record validation.
R16 shrinks the NRO from 256KiB to 152KiB while preserving every artwork byte.
The decoder runs once per ImGui context; rendering uses the same texture and
draw calls. [Compression evidence](COMPRESSION.md) records the checks and timing.

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
| Positive crit | Procedural pink-red star above | Digit-colored line above |
| Negative crit | Procedural blue star below | Digit-colored line below |

Wilds appearance uses the licensed Open Sans digit subset, normalized to the
accepted visible height. Digits have a smooth centered shadow; gold digits also
have a soft warm halo. Original procedural stars include colored bloom and a
pale center. They align to actual pixel-rounded glyph bounds and fade with the
numbers. Digit color remains independent of crit polarity. Classic keeps
Forge's font, outlined digits and straight critical lines.

[Font/effect preparation](fonts/README.md) and
[asset provenance](OPEN_ASSETS.md) describe the supported sources. No extracted
Wilds font or texture is read, embedded or packaged.

Both styles preserve the accepted impact anchor, visible digit height and size.
Wilds stamps from 1.2x to normal size over 0.2 seconds, holds still, then rises
30 pixels at 720p during the final 0.2-second fade. Duration is the total lifetime:
0.9 seconds gives a 0.7-second hold. Classic retains its original pop and drift. Measured glyph bounds and solid markers
reserve the maximum initial-pop dimensions. Placement checks 17 nearby staggered
positions within 1.6 font sizes horizontally and one vertically, minimizing
solid overlap before displacement. Glow may overlap. Offsets stay cached until
expiry; crowded bursts may overlap instead of growing a tall stack. Camera,
context, viewport, style and size changes reset placement caches.

Defaults: enabled, Wilds, scale 1.0, duration 0.9 seconds; 20-pixel font size at
720p. Wilds uses the stamp/hold/exit timing above. Invalid, behind-camera and off-screen projections are hidden.
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
`work/damage-numbers/open-visual/preview.png` with linear texture sampling.
This checks actual font and sprite output without claiming an Eden GPU test.

R9 passed all 16 LS/bowgun checks, settings persistence, monster-to-monster,
Prowler and bomb display. The reported wide LBG spacing prompted this update.
The r10 working checklist marks style, saved-style restart and spacing checks
complete. The user described the crit symbols as near-perfect and requested
closer font, thickness/alignment and weakspot glow matching. R12 passed the
completed visual checklist and the user confirmed the mod is finished.

## Build and delivery

Open assets are the default and only supported profile. The offline ARM32 build
verifies installed Forge exports and preserved game/SDK/Better Input hashes.
No new capture hooks, CodeMemory objects or TLS slots are added. The rebuilt
plugin must match its compiled-fixture and asset-audit NRO hashes before packaging.

```bash
python3 scripts/build_damage_numbers_capture.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/package_damage_numbers.py
```

Current ZIP: `build/damage-numbers/releases/r17/damage-numbers-r17.zip`.
Package: `build/damage-numbers/releases/r17/damage-number/`.
Source manifest: `build/damage-numbers/open-assets/candidate-manifest.json`.
The ZIP contains only the runtime NRO. README, combined redistribution notices,
provenance/audit manifest and checksums remain beside it. Keep `NOTICES.txt`
with the ZIP when redistributing. NRO and archive hashes are recorded there.

The user approved the open appearance and selected 20px at 720p as size 1.0
from the real-renderer comparison. This is 30px at 1080p and 40px at 1440p.
Existing version-2 scale values remain multipliers of the new base; no save is
changed. Installation is separate from packaging. Historical gameplay evidence
is retained; final-size verification uses host and compiled ARM32 fixtures.
