# Damage-number capture trial

The prepared package is
`build/damage-numbers/capture/candidate-package/`. It contains only
`romfs/nativeNX/plugins/damage_numbers_capture.nro`. The manifest at
`build/damage-numbers/capture/candidate-manifest.json` records its hash, native
signatures, imports, source hashes and preserved Forge/SDK inputs.

R8 starts automatically when Forge loads the enabled package in Eden. Both
combat and camera hooks start together; no `.enabled` files are needed. Old
capture/camera flags are ignored and may be removed while Eden is closed.
The builder stages the NRO without installing it or creating activation files.
Numbers start on unless a saved preference turns them off; the in-game **Show
damage numbers** switch controls visibility.

The old flags isolated startup failures during early trials. The retained SDK
TLS and font-layout fixes remain in place. Empty RomFS files are still rejected:
on Eden `d16735f5b6`, they can overlap the following font's data offset. See
[RUNTIME.md](RUNTIME.md) for the historical failure evidence.

## What the enabled candidate does

- Installs eight fingerprinted combat and camera hooks through one private
  4 KiB arena. It allocates one SDK TLS slot, separate from Forge's context
  facilities. Title/build checks and transactional installation still apply.
- Captures positive native monster-damage contributions for the local
  hunter/Prowler and their palicoes. Healing type `0x1d`, nonpositive values and
  other hunters/palicoes are excluded. Values include accompanying elemental
  damage once the native collision path has combined it.
- Retains raw source tags for later bomb/support-move classification. A locally
  owned placed object can appear in these research records. Optional monster
  and placed-object display switches are independent and off by default.
- Copies impact positions, physical HZV, native critical/effect bytes and part
  IDs. Each normalization application remains a distinct diagnostic record.
- Records both active weapon attacks when the native feedback getter runs.
  Captures scoped quest-camera matrices and rectangles automatically.
- Shows counters and the latest eight local damage records in Forge's plugin
  panel. These copied records survive quest/village and local-player changes,
  roll forward with new hits and clear on restart. Gray bounce coloring requires
  uniquely matching feedback for the latest hit on the same thread and counter;
  ambiguous feedback leaves its HZV color unchanged.
- Draws outlined numbers near copied impact
  positions with a short size pop, upward drift, 0.9-second lifetime and fade.
  Physical HZV selects white below 45 or orange-yellow at least 45; unknown HZV
  stays white. Native positive/negative critical outcomes add an outlined line
  above/below. Multi-hit lanes reserve space for those lines.
- Displays directly owned hunter/Prowler/palico attack kinds 0 and 3, plus the
  observed palico child attack tags `0x030402` and `0x030802` when ownership
  resolves to the local palico. Other supported sources follow their individual
  display switches; unknown source families remain hidden.
- Uses only fresh (under 200 ms), scoped quest-camera matrices. A full-screen
  zero-origin native rectangle with matching display aspect maps to Forge's
  swapchain size. Unrecognized layouts hide the preview. Player/camera changes
  and stale cameras clear floating numbers without clearing diagnostic history.
  Capture timestamps prevent delayed records from acquiring a new lifetime.

The candidate uses Forge's counted `savedata` mount to read/write its own
`damage_numbers.ini` and recovery copy beside Better Input. It ignores the old
SD preferences. Game save files and Better Input's INI are preserved. Gameplay
values, affinity, Forge and Eden settings are unchanged. Normal gameplay still
has its own save behavior. The first 256 consumed hits per process are sent to Eden's log
through Forge, with damage, HZV, critical/effect bytes, owner, source tags, part,
impact position and per-thread identity. Logging runs only in the render consumer,
with at most eight hits and 64 total records consumed per callback. A round-robin
cursor visits each thread mailbox. After one limit marker, the panel continues
capturing without further hit logs. Eden can rotate its log on restart; preserve
the current log before another launch. Scope, thread-pool and overwritten-record counters
make capture loss visible.
Shared diagnostic counters use one weak compare-and-swap attempt and can
undercount concurrent calls. Individual record serials and damage values are
independent of those counters. Thread-context claims have at most 16 attempts,
one per slot, and do not retry a contested slot.

R4 also logs one valid camera snapshot (rectangle, display size and matrices)
and up to four distinct projected hits. These diagnostics are render-side and
bounded. The SDK clock frequency is checked before hooks/TLS are allocated.

## Small manual check

With Eden closed, replace only the installed damage-number NRO with the staged
r8 candidate. The two old activation files are unnecessary. Keep the current
Forge, Better Input and emulator settings.

Start normally and land one hit to confirm automatic startup. For the untested
settings persistence, turn numbers off/on, save a small size change, restart
once and check that it remains. Restore defaults and save afterward. Bounce
and optional-source checks can follow during ordinary play. A completed save
and a repeat of the accepted appearance tests are unnecessary.

When collecting further runtime evidence, preserve the current Eden log before
restarting and keep rollback material under workspace `backups/`. Critical
markings, Prowler/ranged cases and multiplayer filtering still need suitable
in-game evidence; the compiled fixtures do not establish those results.

If initialization rejects a signature or allocation, stop the run and retain
the evidence. If capture reports an external hook change, it stops collecting
and retains originals; remove the trial package and restart. Runtime conclusions
must describe the exact reviewed NRO and test cases.

## Rebuild and fixture checks

```bash
python3 scripts/build_damage_numbers_gateway.py
python3 scripts/build_damage_numbers_capture.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
```

These commands use the existing offline compiler container and pinned Unicorn
environment. They do not install a package or create activation files. The compiled
capture report is `build/damage-numbers/capture/compiled-capture-probe.json`.
Its collision body, SDK services and kernel bridges are explicit fixtures;
live per-hit accuracy and untested attack cases remain pending. R4 presentation,
r5 palico projectiles and white physical-HZV damage passed the user's trial.

R8 keeps the same eight hook sites and one private CodeMemory arena. Player-save
INI persistence, conservative gray bounce and optional-source runtime acceptance
remain pending.
