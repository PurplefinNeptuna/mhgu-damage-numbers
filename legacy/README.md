# Damage numbers

Manual confirmation: [in-game checklist](TEST_CHECKLIST.md) and
[results sheet](TEST_RESULTS.md), covering LS and bowguns.

The user's 2026-10-05 r9 trial passed all 16 result rows: LS/bowgun critical
markers, bowgun impact separation, elemental contribution, gray bounce and
Mind's Eye behavior, LBG rapid fire and multiplayer filtering. LBG rapid-fire
numbers were reported as too widely spaced; that cosmetic follow-up remains.
The user also confirmed settings persistence works after restart.
See [the runtime record](RUNTIME.md#2026-10-05-r9-manual-confirmation) for scope.

Development targets the preserved MHGU 1.4.0 Switch executable and local patched
Forge. R3 startup, hunter/palico capture, hit logging and quest-to-village
diagnostic retention passed the user's trial. Its screenshot and retained log
show 23 separate records with zero capture drops or read failures.

R4 adds a floating-number preview through Forge's free-render callback, using
captured final camera matrices, outlined digits, HZV colors, native critical
lines, pop/drift/fade and separate multi-hit lanes. The user accepts player/palico
melee placement, animation and size; those values are unchanged in r5. The r4 log
captured 51 hits, including 18 palico child attacks omitted by the preview filter.
R5 admits their observed source tags only with resolved local-palico ownership.
The user confirmed boomerang/barrel popups and white melee damage on a low-HZV
part. R6 added the settings panel, SD preferences, conservative gray bounce
matching and independent optional-source switches. Bounce passed the r9 trial,
and the user separately confirmed settings persistence. The optional-source
switches still need an in-game check. R7 starts combat and camera hooks
automatically when Eden loads
the enabled package. No activation files are required; existing `.enabled` files
are ignored. Numbers are on by default, with an in-game switch and saved
preferences. R8 stores a separate INI beside Better Input in the current player's
save container and ignores the old SD preferences. Earlier startup failures and the nonempty-flag correction are recorded in
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

## Settings and current trial

Open **Damage numbers** in Forge. Changes apply immediately. **Save settings**
keeps them after restart; **Restore defaults** returns the accepted size,
duration and source switches. R9 uses a compact, unframed tree row matching
**Better Inputs Config**, closed by default, with a nested compact diagnostics
row. Floating numbers continue while the settings row is closed. Settings
use `savedata:/damage_numbers.ini`, beside `better_inputs.ini` in the current
player's save container. `damage_numbers_backup.ini` retains the previous valid
configuration for recovery from a partial write. Both are readable INI files,
bounded to 255 bytes. The old SD files are ignored, without migration or deletion.

Only these two config files are opened; the game save files and Better Input's
INI are preserved. Preferences load at initialization or, if save storage is not
ready, retry at most once per second on the render consumer. Editing a setting
cancels that deferred load so it cannot overwrite a manual change. An unavailable
mount leaves settings active for the session and shows a storage message.

Gray bounce uses native feedback only when one embedded hunter attack matches
the latest finalized hit on the same thread, with unchanged flags, critical
outcome and attack counter, within 100 ms. Another collision invalidates the
match. Ambiguous or older feedback retains the physical-HZV color. Palico and
projectile effect zero never qualifies by itself.

Monster damage follows native dispatch subtype 2. The object switch covers
generic subtype-8 shell impacts with a resolved hunter owner, including remote
owners. Palico/Prowler support bombs remain ordinary owned attacks, and remote
weapons/palicoes remain hidden even with both optional switches enabled.
Other environmental families and recurring ticks remain unsupported. The new
optional source families still await a manual trial. Gray bounce, its off switch
and Mind's Eye prevention passed the user's r9 checklist.

Settings persistence has passed the user's manual check and does not need a
repeat trial. A placed bomb with its switch on/off can be checked later when
convenient. No completed save or extensive test session is needed.

The existing core is independent of game memory. `ownership.h` uses checked
reads, the native local-session index and a palico's current work pointer.
`camera.h` converts copied native view/projection matrices into the display's
matrix convention. Neither helper identifies an active gameplay context by
itself; those runtime gates remain required. `presentation.h` emits layout
commands. The preview renders outlined text through Forge; the user accepted
camera projection and melee impact placement in the r4 trial.

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

The first command passed 441 host checks with AddressSanitizer and UBSan.
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
passed 248 checks using explicit SDK/kernel fixtures and a synthetic collision
body that calls the original HZV and damage routines. The actual collision
traversal, raw/element assembly and camera selection remain unvalidated by these
fixtures. R4's camera projection, animation and size passed the user's melee
trial. The user confirmed that r2 starts and captures damage; individual hit
accuracy still needs independent comparison. R3 history retention and hit logging
are confirmed by the user's village screenshot and retained Eden log.

The r4 compiled checks record actual ARM32 ImGui call arguments with explicit
font/draw fixtures. They cover outlined glyphs, HZV colors, critical lines and
spacing, native capture timestamps, fade/expiry, stale-camera suppression,
viewport scaling and context cleanup. They do not execute Forge's GPU renderer.

The original SDK getter/setter and clock probe passed 260 ABI checks with
synthetic thread storage and a stubbed hardware counter. The native frequency
routine returns 19,200,000 ticks per second; the original C clock wrapper returns
a 64-bit integer in r0/r1. The compiled fixture uses a nonzero return register for
the void setter and checks both the six-hook default and the eight-hook camera
opt-in.

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
