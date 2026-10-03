# Damage-number capture trial

The prepared package is
`build/damage-numbers/capture/candidate-package/`. It contains only
`romfs/nativeNX/plugins/damage_numbers_capture.nro`. The manifest at
`build/damage-numbers/capture/candidate-manifest.json` records its hash, native
signatures, imports, source hashes and preserved Forge/SDK inputs.

This is recovery r2, rebuilt after the first enabled candidate made Eden exit
before the main menu. The rebuilt package is not installed. Its opt-in file,
`romfs/nativeNX/forge/damage-numbers-capture.enabled`, is absent. Without that
file the candidate stays inactive and allocates no capture hooks, arena or TLS
slot. Creating the flag and installing the package require the separately
authorized runtime trial from the accepted plan.

## What the enabled candidate does

- Installs six fingerprinted combat hooks through one private 4 KiB arena.
  It allocates one SDK TLS slot, separate from Forge's context facilities.
- Camera hooks are off. Leave `damage-numbers-camera.enabled` absent during
  the recovery check. That separate opt-in adds the viewport/upload hooks.
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
  panel. There are no floating numbers yet. Bounce decisions remain aggregate
  diagnostics until their individual-hit correlation is verified.

The candidate does not mount or write save data, change gameplay values, reroll
affinity, upgrade Forge or alter Eden settings. Normal gameplay still has its
own save behavior. Captured data stays in bounded memory; routine hit logging
and disk output are avoided. Scope, thread-pool and overwritten-record counters
make capture loss visible.
Shared diagnostic counters use one weak compare-and-swap attempt and can
undercount concurrent calls. Individual record serials and damage values are
independent of those counters. Thread-context claims have at most 16 attempts,
one per slot, and do not retry a contested slot.

## Proposed bounded procedure

For the user, the recovery check is just startup: replace the old installed NRO
with this candidate while the game is closed, retain the capture opt-in file,
leave the camera opt-in absent, then restart. Open Forge's **Plugin UI** and
check the capture status. Report whether the main menu loads; a screenshot of
the status panel is enough if it does. No quest is needed for this retry.

The later development procedure below remains pending until startup works.

1. After authorization, confirm Eden is closed and recheck its package list,
   activation configuration and the preserved Forge/Better Input hashes.
   Preserve saves and the current Eden log in workspace `backups/`.
2. Stage a separate temporary package with the exact reviewed NRO and opt-in
   file. Keep rollback material outside Eden. Retain the existing emulator
   configuration for this capture trial.
3. Start the game and inspect Forge's **Damage numbers - capture research**
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
live hit correlation, camera selection and Eden success remain pending.
