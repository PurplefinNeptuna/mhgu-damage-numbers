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

## Prepared r4 camera and floating-number preview

R4 adds native capture timestamps and a renderer to the free-render callback.
It retains the six-hook default and one private arena/TLS slot. The existing
camera opt-in enables all eight hooks and the preview; both activation templates
contain `1` and a newline. The candidate package itself contains only the NRO.
The installed r3 package and emulator configuration remain unchanged.

The preview uses the captured final view/projection matrices, a scoped
`uQuestCamera` identity, a snapshot younger than 200 ms and a full-screen native
rectangle whose aspect matches Forge's swapchain. It renders outlined digits,
physical-HZV colors, native critical lines, pop/drift/fade and separate multi-hit
lanes. Diagnostic history survives transitions; floating numbers clear on
player/camera changes or stale/unsupported camera data. Hits delayed longer than
their lifetime are retained in diagnostics but omitted from the display.

This first visual trial accepts direct hunter/Prowler/palico kinds 0 and 3.
Secondary ownership/source classification, bounce gray, optional sources and
settings persistence remain pending. It does not infer a per-hit bounce from
the aggregate feedback getter or change native damage/affinity processing.

Automated checks: 435 host policy/presentation checks under ASan/UBSan, mailbox
overflow and eight-producer stress with 160,000 published packets, 23 hook
transaction scenarios, 238 compiled NRO checks with explicit SDK/kernel/font/
draw fixtures, and 260 original SDK TLS/clock ABI checks. The SDK clock checks
execute the original C return wrapper with a stubbed hardware counter and the
original frequency routine (19,200,000 Hz). The compiled renderer checks inspect
ARM32 draw arguments; they do not execute ImGui font rasterization or GPU work.

One short normal solo quest is the pending live check: hit placement while
turning the camera and normal disappearance after the quest. No installation,
activation, save edit or emulator-setting change was performed to prepare r4.

## 2026-10-03: r4 melee visuals accepted; palico projectile exclusion

The user reports that player and palico melee popups work well, with accepted
impact positioning, animation and size. They report no popup for palico
boomerangs or thrown barrels and have not encountered positive/negative critical
outcomes. The village screenshot retains 51 hits, 18 preview rejections and zero
scope/pool drops, orphan contributions, overwritten records or read failures.

The retained log contains 18 hunter and 33 local-palico records. Source tag
counts are `000100`: 18, `030403`: 15, `030402`: 15 and `030802`: 3. The latter
18 records are captured correctly as local palico but rejected by r4's preview
source filter. HZVs are -1, 75 and 80; critical bytes are all zero. The camera
log records a 1920x1080 native rectangle and matching Forge display, with four
distinct projected hits. Back in the village, the panel shows no active camera
rectangle and zero visible numbers. The screenshot is diagnostic evidence;
the user's report establishes the actual in-combat visual result.

Evidence is retained in
`backups/damage-numbers/capture-r4-visual-20261003-105730/`: log, supplied screenshot,
installed package snapshot and `runtime-evidence.json`. Log SHA-256:
`67b2eb0c39efa5bad82967e32e7aae4f3da1165a1db2eab877c91b9e90d84ddf`.
Installed r4 NRO SHA-256:
`4e4e1ff78c884f574aa8e5e8d9ac31ffe59d01e4eb6e424fa2e3908d4d89e218`.
Both activation flags are two-byte `1` plus newline files.

R5 admits only tags `0x030402` and `0x030802` when the resolved owner is the
local palico. Camera mapping, size, position, animation, lifetime and all combat
hooks remain unchanged. Compiled fixtures exercise secondary palico ownership,
white unknown-HZV barrel-like damage, physical-HZV projectile color, remote-palico
exclusion, unknown-source omission and healing rejection. Host checks: 441;
compiled NRO checks: 248. Its projectile display check is pending.

The next manual check is only palico boomerang and thrown-barrel popups during
ordinary play. A 100% save or save modification is unnecessary for this check;
critical-marker runtime acceptance remains pending until suitable equipment is
available. No installed package, setting, save or presentation value was changed
while preparing the fix.

## 2026-10-03: r5 projectiles and white coloring accepted

The user confirmed that palico boomerang and thrown-barrel numbers now show.
After the color check, they also confirmed white melee damage. Their accepted
impact position, animation and size remain the r6 defaults. The log's first
256 recorded impacts contain known physical HZVs 47 through 80 and eight
unknown-HZV contributions; the later white-color acceptance comes from the
user's in-game observation, not an uncaptured log value.

The installed r5 NRO still has SHA-256
`81ed488cb38f13c7899e8cf7822b3b235df2d9c87d0fbeb12553e29fd2c244e0`.
Its package snapshot, log and acceptance note are retained under
`backups/damage-numbers/capture-r5-accepted-20261003-123254/`.
The copied log SHA-256 is
`55995b9d3d30328a6799bb622b036e4b4199d00f9fe785a0eef3e008b17e2cbc`.
Critical markings and multiplayer exclusion still need manual evidence.

## 2026-10-03: r6 settings, conservative bounce and optional sources staged

R6 adds an enable switch, size/duration controls, default restoration, an
explicit Save settings button and collapsed diagnostics. Preferences use two
checksummed SD files; game saves are not mounted or written. SDK filesystem
calls run inside a scoped thread-local abort disabler, restoring the previous
context even on storage failure. The mount/persistence path awaits Eden testing.

Gray bounce matches only the latest uniquely identified hunter contribution
to native resolved feedback within 100 ms, on the same thread and attack
counter. It leaves ambiguous, older, different-thread and earlier multi-hit
contributions at their HZV color. Optional monster dispatch and generic
hunter-owned object impacts have independent off-by-default switches. Other
environmental families and recurring ticks remain hidden. Remote weapon and
palico attacks do not become objects when the switches are enabled.

Automated checks passed: 470 ASan/UBSan core/presentation checks, mailbox overflow
and eight-producer stress with 160,000 published packets, 23 transaction cases,
278 compiled NRO checks with explicit SDK/kernel/ImGui fixtures, and 36 checks
executing the original SDK filesystem guard/context/abort-decision bodies.
The compiled checks cover settings round trips, recovery from a partial write,
source toggle cleanup, remote-source exclusion and constrained bounce matching
through the original feedback getter. None of these checks is an Eden/GPU test.

Staged NRO SHA-256:
`20e2b7ed3133609b2cb625e76d24d394b82b63d3eca77c5ed5d81ef874be14c7`.
R6 retains the same eight optional-preview hook sites, one private 4 KiB arena,
one native TLS slot and 248-byte mailbox records. Both two-byte activation
templates remain separate from the one-NRO candidate package. Installed
Forge, Better Input, damage-number files, emulator configuration and saves
were not changed while preparing it.

The first manual check is small: toggle numbers off/on, save a slight size
change and restart once. Restore defaults afterward. Bomb switch and actual
recoil checks can follow during normal play; no completed save is needed.

## 2026-10-03: r7 automatic startup staged

The user requested activation without extra flag files. R7 removes both RomFS
flag checks and their generated templates. Forge initializes all eight combat
and camera hooks automatically when the package is enabled in Eden. The
default preference shows numbers; the in-game switch and any saved disabled
preference still apply. Old `.enabled` files are ignored.

Title/build signatures, native clock validation, SDK TLS and transactional
installation checks remain. Hook sites, the private 4 KiB arena, mailbox
layout, source policy and accepted appearance are unchanged from r6. The
compiled NRO has no Forge filesystem imports for activation-file access.

Automated checks passed: 276 compiled NRO checks with explicit SDK/kernel/ImGui
fixtures, including automatic installation of all eight hooks without flag APIs,
rendering after a camera snapshot, saved visibility preferences and detection
of a modified camera hook. These are fixture results; r7 startup in Eden and
the r6 settings/bounce/optional-source additions still await manual acceptance.

Staged NRO SHA-256:
`43c6dc8a105789e32e4d22b7e8e514f235e1226641600176ba31527960685021`.
The candidate package contains only the 65,536-byte NRO. The installed r5 NRO
retains SHA-256
`81ed488cb38f13c7899e8cf7822b3b235df2d9c87d0fbeb12553e29fd2c244e0`.
No installed package, activation configuration or save was changed.

With Eden closed, replace only the damage-number NRO. The two old activation
files may be removed. Start normally and land one hit to confirm automatic
startup; the small settings/restart check can follow if it has not been done.
