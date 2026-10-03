# Damage numbers

Development targets the preserved MHGU 1.4.0 Switch executable and local patched
Forge. Phase 1 combat-binding research is in progress. The host display core and
selected native routines have passed fixture checks. An ARM32 diagnostic capture
NRO has been built and inspected. The first enabled candidate caused Eden to
exit before the main menu in the user's trial. Recovery r2 corrects the SDK TLS
setter ABI and leaves camera hooks off by default. Its second startup trial also
failed: the empty activation file can shadow Forge's font through Eden's RomFS
layout. With the nonempty activation file, the user confirmed startup, damage
capture and quest completion. The panel then cleared its records on the quest
transition. Capture r3 retains the last eight hits across transitions and sends
the first 256 hits per restart to Eden's log. The user confirmed r3 works; its
village screenshot and retained log show 23 separate hunter/palico records with
zero scope/pool drops, orphan contributions, overwritten records or read failures.
Quest-to-village retention and live hit logging passed this trial. See
[RUNTIME.md](RUNTIME.md) and [CAPTURE_TRIAL.md](CAPTURE_TRIAL.md).

## Accepted behavior

- Display positive damage dealt to large and small monsters by the local
  hunter/Prowler and their palicoes. Hide other hunters and their palicoes.
- Keep every distinct impact separate, including hits in the same frame and
  separate phial explosions. Show physical damage plus accompanying elemental
  damage as one number per impact, using native damage resolution and rounding.
- Exclude healing, zero damage, status buildup and damage received by hunters.
- Optional monster-to-monster damage, placed bombs/environmental objects and
  recurring damage ticks have independent switches, all off by default. Objects
  and ticks include all owners when enabled. Prowler/Palico support bomb moves,
  shells and phials remain ordinary owned attacks.

| Condition | Appearance |
| --- | --- |
| Physical HZV below 45 | White |
| Physical HZV at least 45 | Orange-yellow |
| No applicable or known physical HZV | White |
| Actual bounce, when bounce coloring is enabled | Gray |
| Recurring damage tick | Gray |
| Positive critical outcome | Line above |
| Negative critical outcome | Line below |

Bounce coloring defaults to on. The physical hitzone for the resolved attack
type and current monster-part state selects the color. Equipment affinity and
sharpness alone cannot establish a critical or bounced hit. Ticks have no
critical marker.

Numbers use the actual impact position, outlined digits, a short scale pop,
upward drift and fade. Defaults are 24 pixels at a 720p viewport, a 0.9-second
lifetime and a 0.25-second fade, with white `#FFFFFF`, gray `#9A9A9A` and
orange-yellow `#FFBF3F`. Invalid, behind-camera and off-screen projections are
hidden. Recurring ticks need a verified monster-body anchor.

The model bounds its queue at 256 events, visible list at 128 numbers and recent
event identities at 512. Overflow removes oldest entries and counts drops;
damage is never summed to make room. Settings changes discard newly excluded
events. Context changes clear the queue and visible list.

## Phases and acceptance

1. Verify final damage, ownership, impact identity/position, HZV, bounce,
   critical outcomes and the active gameplay camera. Record original-byte
   fingerprints and ARM32 calling conventions. Then conduct a separately
   authorized bounded capture trial.
2. Integrate the verified bindings in an independent C++ plugin, using a
   private hook arena. Preserve original processing, RNG and gameplay state.
   Validate ordinary hunter, Prowler and palico capture and filtering.
3. Render through Forge's free-render callback and add settings persistence.
   Validate outlined text, critical lines, camera movement, overlap handling,
   FOV changes, resolutions and cleanup through area/quest transitions.
4. Add optional sources and test all switches. Validate melee, ranged
   multi-hits, elemental weapons, shells/phials, Prowler/Palico attacks,
   remote-player exclusion, crowded combat, save loading and repeated restarts.
   Multiplayer filtering requires an actual multiplayer test.

The existing core is independent of game memory. `ownership.h` uses checked
reads, the native local-session index and a palico's current work pointer.
`camera.h` converts copied native view/projection matrices into the display's
matrix convention. Neither helper identifies an active gameplay context by
itself; those runtime gates remain required. `presentation.h` emits layout
commands. The native outlined-text renderer is pending.

Research evidence and open bindings are listed in [RESEARCH.md](RESEARCH.md).
The runtime adapter must serialize model access without taking game locks or
spinning in a damage detour, and must copy event values without retaining game
object pointers in the render list.

## Current checks

```bash
python3 scripts/test_damage_numbers.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_arm_probe.py
python3 scripts/build_damage_numbers_gateway.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_gateway_probe.py
python3 scripts/build_damage_numbers_capture.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_sdk_tls_probe.py
python3 scripts/test_damage_numbers_romfs_flag.py
```

The first command passed 416 host checks with AddressSanitizer and UBSan.
LeakSanitizer is disabled because thread enumeration fails in the ptraced
sandbox; the tested core uses fixed storage.
The mailbox also passed overflow/wrap checks and eight-producer stress with
160,000 published packets under AddressSanitizer/UBSan and ThreadSanitizer.
Each native thread has its own mailbox; the render side has one enforced
consumer. Overwritten records are counted and never combined.

The second passed 438 original ARM32 damage-normalization checks, 144 camera
checks, 92 ownership checks, 502 critical-outcome checks and 10,464 weapon-feedback
checks across 3,488 native getter calls. It uses Unicorn
2.1.4's ARM_MAX CPU model for ARMv8 AArch32 floating-point instructions and
validates selected copied native routines
against explicit fixtures and the actual C++ helpers. It stubs the named
scene/quest services and tangent import documented in its report. These checks
do not establish collision capture, active viewport selection, hook safety,
multiplayer attribution or Eden acceptance. The feedback getter combines both
active attacks; its aggregate result is not assigned as a bounce flag to every
individual hit.

The compiled gateway passed 9,031 checks across 1,769 calls. These include 1,440
original melee-HZV queries with a mid-routine tap, complete register/float/flag
preservation, nested scopes and scope-capacity bypass. The capture NRO probe
passed 213 checks using explicit SDK/kernel fixtures and a synthetic collision
body that calls the original HZV and damage routines. The actual collision
traversal, raw/element assembly and camera selection remain unvalidated by these
fixtures. The user confirmed that r2 starts and captures damage; individual hit
accuracy still needs independent comparison. R3 history retention and hit logging
are confirmed by the user's village screenshot and retained Eden log.

The original SDK getter/setter probe passed 256 ABI checks with synthetic thread
storage. The compiled fixture uses a nonzero return register for the void setter
and checks both the six-hook recovery default and the eight-hook camera opt-in.

The RomFS model reproduces the zero-byte flag selecting padding at Forge's font
offset using the inspected dump/listed-package metadata and Eden `d16735f5b6`
layout logic. Absent and nonempty flags retain the font header. This is an offline
layout check. The user subsequently confirmed startup with the nonempty flag.

Generated research inputs and logs stay in `work/damage-numbers/`; native reports
and the capture candidate stay in `build/damage-numbers/`. Originals, saves,
installed packages and emulator settings remain untouched by these commands.
The saved DTI results are reused; the dumper must remain removed.

## Delivery boundaries

Keep the tested Forge branch and installed artifacts intact. Verify build
identity, patch-site bytes, native ABI, relocation, CodeMemory demand, imports
and package structure before any installation. Prepare rollback outside Eden,
preserve logs before restarting and get separate authorization for installation
and runtime trials. No Forge upgrades, emulator-setting changes or pushes are
part of this implementation.
