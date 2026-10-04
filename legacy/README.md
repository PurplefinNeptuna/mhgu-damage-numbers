# Damage numbers

Manual confirmation: [in-game checklist](TEST_CHECKLIST.md) and
[results sheet](TEST_RESULTS.md), covering LS and bowguns.

R9 passed all 16 LS/bowgun checks on 2026-10-05: crit markers, separate impacts,
elemental contribution, bounce/Mind's Eye, rapid fire and multiplayer filtering.
Settings persistence, monster-to-monster damage, Prowler mode and bomb damage
also passed. Rapid-fire spacing remains the only reported issue and needs
cosmetic polish. See [the runtime record](RUNTIME.md#2026-10-05-r9-manual-confirmation).

Target: MHGU 1.4.0 Switch and local patched Forge. R3 passed startup,
hunter/palico capture, hit logging and quest/village history retention: 23
records, zero drops/read failures.

R4 added Forge free-render numbers using final camera matrices, outlined digits,
HZV colors, native crit lines and separate pop/drift/fade lanes. The user
accepted melee placement, animation and size. Its 51-hit log exposed 18 filtered
Palico child attacks; r5 admitted their observed tags with local ownership,
fixing boomerang/barrel popups and passing white melee coloring. R6 added
settings, SD preferences, conservative bounce matching and optional sources. R7
starts combat/camera hooks automatically; `.enabled` files are ignored.
Numbers default on unless preferences disable them. R8 moved preferences beside
Better Input and ignores old SD files. Startup/font failures:
[RUNTIME.md](RUNTIME.md), [CAPTURE_TRIAL.md](CAPTURE_TRIAL.md).

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

Bounce coloring defaults on. Color follows the resolved physical attack type and
current part state. Equipment affinity/sharpness alone cannot prove crit or
bounce. Ticks have no crit marker.

Outlined numbers pop, drift upward and fade at the impact position. Defaults: 24
pixels at 720p, 0.9-second lifetime, 0.25-second fade; white `#FFFFFF`,
gray `#9A9A9A`, orange-yellow `#FFBF3F`. Invalid,
behind-camera and off-screen projections are hidden. Ticks require a verified
monster-body anchor.

Limits: 256 queued events, 128 visible numbers, 512 recent identities. Overflow
drops/counts oldest entries without summing damage. Settings discard newly
excluded events; context changes clear queued/visible numbers.

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

Open **Damage numbers** in Forge. Changes apply immediately; **Save settings**
persists them and **Restore defaults** restores size, duration and source
switches. R9 uses a closed-by-default compact tree row like **Better Inputs
Config**, with nested diagnostics; numbers render while closed. Preferences:
`savedata:/damage_numbers.ini`, beside `better_inputs.ini`; `damage_numbers_backup.ini` recovers
partial writes. Both are readable INIs limited to 255 bytes. Old SD files are
ignored, never migrated/deleted.

Only the two plugin configs are opened; game saves and Better Input's INI are
preserved. Loading starts at initialization and retries at most once/second on
the render consumer if storage is unavailable. Manual edits cancel deferred
loading. A failed mount keeps session settings and reports its status.

Gray bounce requires one embedded hunter attack matching the latest finalized
hit on the same thread within 100 ms, with unchanged flags, crit outcome and
attack counter. Another collision invalidates it. Ambiguous/older feedback keeps
HZV color; Palico/projectile effect zero alone never qualifies.

Monster damage uses dispatch subtype 2. Objects cover generic subtype-8 shell
impacts with a resolved hunter owner, including remote owners. Palico/Prowler
support bombs remain owned attacks; remote weapons/palicoes stay hidden even
with both optional switches on. Other environmental families/ticks are
unsupported. Monster damage, Prowler mode, bomb damage, bounce on/off and
Mind's Eye passed the user's trials. Bomb types and object-switch off behavior
were not separately described in the report.

Persistence, Prowler mode and bomb damage need no repeat display trial.
Rapid-fire spacing is the remaining reported cosmetic issue.

The core is independent of game memory. `ownership.h` checks reads,
local-session index and current Palico work; `camera.h` converts copied
native view/projection matrices; `presentation.h` emits layout commands.
Ownership/camera helpers still require gameplay-context gates. Forge draws the
outline; r4 passed projection and melee placement.

Bindings: [RESEARCH.md](RESEARCH.md). Serialize model access without game locks or
spinning in damage detours. Copy events; retain no game pointers in render
lists.

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

Host: 441 ASan/UBSan checks with fixed storage. LeakSanitizer is disabled
because ptrace breaks thread enumeration. Mailboxes passed overflow/wrap and
eight-producer stress (160,000 packets) under ASan/UBSan and ThreadSanitizer.
One mailbox/native thread, one enforced render consumer; overwritten records are
counted, never combined.

ARM: 438 normalization, 144 camera, 92 ownership, 502 crit-outcome and 10,464
feedback checks across 3,488 native getter calls. Unicorn 2.1.4 ARM_MAX supports
ARMv8 AArch32 floats; copied routines run against explicit fixtures and C++
helpers, with documented scene/quest/tangent stubs. This does not prove
collision capture, viewport selection, hook safety, multiplayer attribution or
Eden behavior. Feedback aggregates both active attacks and cannot mark every hit
as bounced.

Gateway: 9,031 checks/1,769 calls, including 1,440 native melee-HZV taps,
register/float/flag preservation, nested scopes and capacity bypass. Capture
NRO: 248 SDK/kernel fixture checks using a synthetic collision body and original
HZV/damage routines. Original collision traversal, raw/element assembly and
camera selection are outside fixture coverage. User trials confirmed r2
startup/capture, r3 history/logging and r4 visuals; absolute per-hit accuracy
still needs independent comparison.

R4 ARM32 ImGui fixtures inspect draw arguments for outlines, HZV colors, crit
lines/spacing, capture timestamps, fade/expiry, stale cameras, viewport scaling
and context cleanup. They do not run Forge's GPU renderer.

SDK TLS/clock: 260 ABI checks with synthetic thread storage/hardware counter.
Native frequency: 19,200,000 Hz; C wrapper: 64-bit r0/r1 return. Fixtures retain
the void setter's nonzero register and test six combat hooks plus eight-hook
camera opt-in.

Offline RomFS modeling of Eden `d16735f5b6` and dump/package metadata
reproduces an empty flag shadowing Forge's font with padding. Absent/nonempty
flags preserve the header; the user confirmed nonempty-flag startup.

Inputs/logs: `work/damage-numbers/`; native reports/candidate: `build/damage-numbers/`.
Commands preserve originals, saves, installed packages and emulator settings.
Reuse saved DTI results; keep the dumper removed.

## Delivery boundaries

Preserve tested Forge/artifacts. Before installation, verify build, patch bytes,
ABI, relocation, CodeMemory demand, imports and package structure. Keep rollback
outside Eden and logs before restart. Installation/runtime trials require
separate authorization. Forge upgrades, emulator changes and pushes are outside
scope.
