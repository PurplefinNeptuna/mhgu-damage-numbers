# Historical capture trial

This R9 record predates the supported open-only r14. See [README.md](README.md)
for current build paths, source behavior and presentation. Old candidate outputs
were retired to the workspace archive.

# Damage-number capture trial

Package: `build/damage-numbers/capture/candidate-package/`, containing only `romfs/nativeNX/plugins/damage_numbers_capture.nro`.
`build/damage-numbers/capture/candidate-manifest.json` records NRO/source hashes, signatures, imports and preserved
Forge/SDK inputs.

R9 starts combat/camera hooks when Forge loads the enabled Eden package. No
`.enabled` files are needed; old flags are ignored and can be removed
with Eden closed. Building stages only the NRO. Numbers default on unless saved
preferences disable them; **Show damage numbers** controls visibility.

Early flags isolated startup failures. SDK TLS and font-layout fixes remain.
Empty RomFS files are rejected because Eden `d16735f5b6` can overlap the
following font. Evidence: [RUNTIME.md](RUNTIME.md).

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

Forge's counted `savedata` mount accesses only `damage_numbers.ini` and
its recovery copy beside Better Input. Old SD preferences are ignored. Game
saves, Better Input's INI, damage/affinity, Forge and Eden settings are
preserved; normal gameplay still saves normally. The render consumer logs the
first 256 hits/process with damage, HZV, crit/effect, owner/tags, part, position
and thread identity. Each callback consumes at most eight hits/64 records,
visiting mailboxes round-robin. One limit marker ends hit logging; capture
continues. Preserve Eden logs before restart. Scope/pool/overwrite counters
expose loss; one weak compare-and-swap can undercount concurrent calls without
affecting records/serials. Thread claims try each of 16 slots once without
retrying contention.

R4 logs one camera snapshot (rectangle, display size, matrices) and up to four
distinct projections on the render side. Clock frequency is checked before
hook/TLS allocation.

## Small manual check

With Eden closed, replace only the damage-number NRO with staged r9. Keep Forge,
Better Input and emulator settings. Old activation files are unnecessary.

Startup, visuals, crits, bowgun separation, bounce, multiplayer filtering,
persistence and monster-to-monster damage have passed. Remaining checks:
Prowler, placed objects, compact row appearance and rapid-fire spacing polish.
No completed save or repeat of accepted tests is needed.

For further evidence, preserve Eden logs before restart and keep rollback under
workspace `backups/`. Record exact NRO/test coverage; fixtures do not
prove untested gameplay.

On signature/allocation rejection, stop and retain evidence. An external hook
change stops capture while retaining originals; remove the trial package and
restart.

## Rebuild and fixture checks

```bash
python3 scripts/build_damage_numbers_gateway.py
python3 scripts/build_damage_numbers_capture.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
```

Rebuilds use the offline compiler and pinned Unicorn, without installation or
activation files. Report: `build/damage-numbers/capture/compiled-capture-probe.json`. Collision/SDK/kernel services are
fixtures; absolute per-hit accuracy and untested attack cases remain open. R4
visuals and r5 Palico projectiles/white HZV passed manually.

R9 retains eight hook sites and one private arena. Persistence and gray bounce
now passed manually; compact-row appearance and placed-object coverage remain
unconfirmed.
