# Damage-number capture trial

The prepared package is
`build/damage-numbers/capture/candidate-package/`. It contains only
`romfs/nativeNX/plugins/damage_numbers_capture.nro`. The manifest at
`build/damage-numbers/capture/candidate-manifest.json` records its hash, native
signatures, imports, source hashes and preserved Forge/SDK inputs.

This is capture/preview r5, prepared after the successful r4 melee visual trial. The build
command only stages files and does not install them. Its candidate-package opt-in file,
`romfs/nativeNX/forge/damage-numbers-capture.enabled`, is absent. Without that
file the candidate stays inactive and allocates no capture hooks, arena or TLS
slot. Creating the flag and installing the package require the separately
authorized runtime trial from the accepted plan.

The activation file must contain text, such as `1` followed by a newline.
Do not use an empty RomFS file: on Eden `d16735f5b6`, it can share the following
font's offset and cause its data to be replaced by padding. A prepared nonempty
file is at `build/damage-numbers/capture/activation/damage-numbers-capture.enabled`.
See [RUNTIME.md](RUNTIME.md) for the reproduced layout and startup failure.

## What the enabled candidate does

- Installs six fingerprinted combat hooks through one private 4 KiB arena.
  It allocates one SDK TLS slot, separate from Forge's context facilities.
- Camera hooks and floating numbers are off by default. The separate nonempty
  `damage-numbers-camera.enabled` flag adds the viewport/upload hooks and enables
  the projection preview. Its prepared template is under `activation/`.
- Captures positive native monster-damage contributions for the local
  hunter/Prowler and their palicoes. Healing type `0x1d`, nonpositive values and
  other hunters/palicoes are excluded. Values include accompanying elemental
  damage once the native collision path has combined it.
- Retains raw source tags for later bomb/support-move classification. A locally
  owned placed object can appear in these research records; source switches and
  their final display policy are not implemented in the capture candidate.
- Copies impact positions, physical HZV, native critical/effect bytes and part
  IDs. Each normalization application remains a distinct diagnostic record.
- Records both active weapon attacks when the native feedback getter runs.
  Scoped quest-camera matrices and rectangles require the separate camera trial.
- Shows counters and the latest eight local damage records in Forge's plugin
  panel. These copied records survive quest/village and local-player changes,
  roll forward with new hits and clear on restart. Bounce decisions remain
  aggregate diagnostics until their individual-hit correlation is verified.
- With the camera flag present, draws outlined numbers near copied impact
  positions with a short size pop, upward drift, 0.9-second lifetime and fade.
  Physical HZV selects white below 45 or orange-yellow at least 45; unknown HZV
  stays white. Native positive/negative critical outcomes add an outlined line
  above/below. Multi-hit lanes reserve space for those lines.
- Displays directly owned hunter/Prowler/palico attack kinds 0 and 3, plus the
  observed palico child attack tags `0x030402` and `0x030802` when ownership
  resolves to the local palico. Other secondary sources stay diagnostic. Gray
  bounce coloring and optional sources are not enabled in this preview.
- Uses only fresh (under 200 ms), scoped quest-camera matrices. A full-screen
  zero-origin native rectangle with matching display aspect maps to Forge's
  swapchain size. Unrecognized layouts hide the preview. Player/camera changes
  and stale cameras clear floating numbers without clearing diagnostic history.
  Capture timestamps prevent delayed records from acquiring a new lifetime.

The candidate does not mount or write save data, change gameplay values, reroll
affinity, upgrade Forge or alter Eden settings. Normal gameplay still has its
own save behavior. The first 256 consumed hits per process are sent to Eden's log
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

## Proposed bounded procedure

For the user, replace only the capture NRO with r5 while Eden is closed. Keep
both existing nonempty flags. In one ordinary short solo quest, watch whether
palico boomerangs and thrown barrels now produce separate popups on impact.
Position, animation and size were already accepted and are unchanged. No repeat
of the melee visual test or 100% save is needed. Critical checks can wait until
suitable equipment is available naturally. Report the projectile result before
another restart so the current log can be preserved.

The later development procedure below covers the remaining research cases; it
is not an exhaustive checklist for this retention check.

The r3 retention check passed: the user's village screenshot and retained Eden
log show 23 separate local hunter/palico hit records after the quest transition,
with zero capture drops or read failures. Evidence and remaining validation
limits are in [RUNTIME.md](RUNTIME.md). No repeat of this check is needed.

1. After authorization, confirm Eden is closed and recheck its package list,
   activation configuration and the preserved Forge/Better Input hashes.
   Preserve saves and the current Eden log in workspace `backups/`.
2. Stage a separate temporary package with the exact reviewed NRO and nonempty opt-in
   file. Keep rollback material outside Eden. Retain the existing emulator
   configuration for this capture trial.
3. Start the game and inspect Forge's **Damage numbers - projection preview**
   panel. Confirm capture is active, combat hook counters advance, and no scope,
   thread-pool or unexpected read failures appear.
4. In one short solo quest, check isolated hits on large and small monsters,
   rapid multi-hits and palico attacks. Compare a raw weapon with an elemental
   weapon, and cover a known physical HZV on each side of 45. Record the panel's
   values and impact positions alongside the attacks. Keep the trial to one
   quest or 15 minutes, whichever comes first.
5. Check low-sharpness bounces and a bounce-preventing move/skill. The two attack
   snapshots and native feedback return must explain which impacts can be
   colored gray. Do not accept raw effect zero alone as proof of a bounce.
6. Cover Prowler, ranged attacks, shells/phials and a quest/village transition
   in additional bounded runs as needed. Multiplayer exclusion needs its own
   actual multiplayer run; solo fixtures do not establish it.
7. Close Eden, preserve the new log and observations, remove the temporary
   package/flag and restart before normal play. Recheck the original installed
   hashes and controls. Keep the runtime outcome separate from fixture results.

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
environment. They do not install a package or create the opt-in flag. The compiled
capture report is `build/damage-numbers/capture/compiled-capture-probe.json`.
Its collision body, SDK services and kernel bridges are explicit fixtures;
live per-hit accuracy, r5 projectile display and untested attack cases remain
pending. R4 melee projection and presentation passed the user's visual trial.
