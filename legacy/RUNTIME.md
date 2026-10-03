# Capture runtime observations

## 2026-10-03: first enabled candidate

The installed candidate matched the prepared NRO SHA-256
`6f9c7a67d22ecb5b662e958c0711d5c341935ee25e7f8b065bd5af1e3558cff4`.
The user first reported normal gameplay with no visible changes. Inspection
found the capture opt-in file absent; the log showed Forge loading the NRO.
This was an inactive-plugin test, not evidence of damage capture.

After creating the opt-in file, the user reported that the game did not reach
the main menu and Eden closed completely. The retained log records successful
installation of all eight capture hooks at 12.504815 seconds. At 12.736447
seconds its final line begins `[ERROR] Assertion fai` and is truncated. There
is no complete assertion, guest backtrace or confirmed failing hook in this log.
The shader-version messages immediately preceding it also occurred in the
earlier inactive run; they do not establish the cause of this exit.

Evidence is retained in workspace
`backups/damage-numbers/startup-failure-20261003-084659/eden_log.txt`, SHA-256
`01cd94c2671124977ebd0e71868367cfa841f2214758e1e37c86d5f2bb61a41b`.
No emulator settings, installed packages or saves were changed during this
diagnosis. Removing the capture activation file and restarting restores the
previous inactive behavior; that recovery has not yet been confirmed by the user.

## Prepared recovery r2

The SDK TLS setter ABI was wrong: its native implementation returns void, while
the supplied Forge header declares `Result`. The old capture adapter treated its
nonzero return register as a failure even after the TLS value had been stored.
This defect is reproduced with original SDK instructions and corrected in the
plugin. It is not confirmed as the cause of Eden's exit.

The new candidate installs six combat hooks. Camera/viewport hooks stay off
unless a separate `damage-numbers-camera.enabled` file is present. One-time
startup markers identify entry/completion of ImGui setup and the first render
consumer's TLS setup. The candidate remains diagnostic-only and uninstalled.
The next manual check is startup and the Forge status panel. A quest test can
wait until startup works.

## 2026-10-03: r2 startup failure and empty-file reproduction

The user reported another crash. The installed r2 NRO matched SHA-256
`fc81dad783daa93321aecc7748a65768d0697cd741123f95a3236e211bbcb9d1`.
Its capture flag was zero bytes and the camera flag was absent. The log confirms
six hooks and camera capture off. No ImGui setup or render-consumer marker runs.
At 13.113296 seconds Forge logs a complete `stbtt_GetFontOffsetForIndex`
assertion in `ImGui_ImplStbTrueType_FontSrcInit`, then the guest process exits.

The log and installed package snapshot are retained in
`backups/damage-numbers/startup-r2-failure-20261003-090155/`. Log SHA-256:
`a6bef204de6f9cc3077a7e6d2700db7ee8658a1002a2fd0869f4d85e5131930d`.
The font SHA-256 is
`9d0d55a303bfd13b79a87721f65185e93f235e2d77fe398b2dca67ac519915f5`;
it matches `tools/forge-local/data/fonts/Roboto-Medium.ttf` byte for byte.

The empty capture flag and font receive the same RomFS offset. A model of Eden's
exact reported revision reproduces its unstable ordering, backward concatenation
end and padding covering the font. An absent flag and flags of 1, 2 and 8 bytes
preserve the font header. Source fingerprints, cases and limits are in
`build/damage-numbers/capture/romfs-flag-probe.json`; details and upstream links
are in [RESEARCH.md](RESEARCH.md).

The corrected activation template contains `1` and a newline. The r2 NRO is
unchanged. No installed file, font, emulator setting or save was changed during
this diagnosis. The next user check is to put text in the existing capture flag
and confirm that startup reaches the main menu. That check subsequently passed.

## 2026-10-03: successful capture and transition retention

The user reports that r2 ran, showed damage records and completed a quest. After
the quest, the diagnostics cleared before they could show the values. This
confirms user-observed capture, but supplies no per-hit evidence for damage
accuracy, HZV, ownership or critical/bounce correlation.

The retained log is
`backups/damage-numbers/capture-success-20261003-094534/eden_log.txt`, SHA-256
`fa80bc224ef6732120c7d0b65e42ee36ad0eee9037f01035da2237a4a7ed0d6f`.
It records six hooks with camera capture off, completed ImGui setup and a ready
render TLS context. The installed NRO still matched r2 SHA-256
`fc81dad783daa93321aecc7748a65768d0697cd741123f95a3236e211bbcb9d1`;
the capture flag was two bytes (`1` followed by a newline), SHA-256
`4355a46b19d348dc2f57c046f8ef63d4538ebb936000f3c9ee954a27460dd865`.
The camera flag was absent. R2 did not log individual hits, so the lost values
cannot be recovered from this log.

Capture r3 keeps the last eight copied damage records across local-player
changes. Camera and feedback snapshots still reset with the player context.
Already queued owned hits are retained even if the player table disappears
before consumption. The first 256 hits per restart are logged from the render
consumer, with at most eight hits and 64 total records consumed per callback.
Capture continues after the log cap. No combat hooks, damage calculation,
ownership rules or final floating-number lifecycle changed.

The compiled NRO fixture checks transition retention, queued hits without a
player, distinct log ordinals, copied values/positions, callback budgets, the
single log-limit marker, continued panel updates and empty history on restart.
R3's in-game retention check remains pending. No installed files, emulator
settings or saves were changed to prepare it.

## 2026-10-03: r3 retention check passed

The user reports that r3 works and supplied a screenshot back in the village.
It shows 23 captured hits, the last eight retained records, and zero scope drops,
pool drops, orphan contributions, overwritten records and read failures. There
were 26 collisions/normalizations, 23 melee HZV queries and three exclusions.
The panel's 1,340 consumed records include 1,317 feedback records and 23 hits.
Camera hooks remained off.

The current Eden log contains all 23 hit lines, with consecutive log ordinals,
distinct native record serials, damage, physical HZV and copied impact positions.
The records identify 15 local hunter hits (`owner 0`) and eight local palico hits
(`owner 1`). Damage ranges from 3 to 23. All records have HZV 80, critical byte 0
and effect byte 4; this sample cannot validate the HZV color boundary, positive
or negative critical markings, or bounce correlation.

Evidence is retained in
`backups/damage-numbers/capture-r3-retention-20261003-101333/`: the log, supplied
village screenshot, installed package snapshot and `runtime-evidence.json`.
Log SHA-256:
`83e1dde7d67f16ee5661c9712f8ce1b3453511ea8ca00f1999415143e13eb542`.
The installed NRO matches the reviewed r3 candidate, SHA-256
`aaa5b873c786635881e49a5dcf65b4795c9c93683599b627cddb3a275c8a387f`.
The capture flag remains nonempty and the camera flag is absent.

Startup, local hunter/palico capture, hit logging and quest-to-village history
retention passed this manual trial. Independent damage/HP comparison, elemental
assembly, other attack/source cases, multiplayer exclusion and active-camera
projection remain pending. No plugin code, installed package, emulator setting
or save was changed while recording this result.
