## r20 crit-size release, 2026-10-07

The user selected the host real-renderer 1.5× positive/negative crit preview
and requested direct readiness. Fresh host/compiled ARM32/asset/GE/package
checks passed; no new Eden, Ryujinx or Switch gameplay run was performed.
This is visual-design acceptance only. [Crit-size evidence](CRIT_SIZE.md)
records the scale, placement bounds and retained digit appearance. Earlier
reports below retain their recorded versions. Forge is unchanged.

## r19 readability release, 2026-10-07

The user approved the host real-renderer 0.5 px outline preview and requested a
built release ready to distribute. Fresh host/compiled ARM32/asset/GE/package
checks passed; no new Eden, Ryujinx or Switch gameplay run was performed.
This is visual-design acceptance only. Earlier reports below retain their
recorded payload/version. [Readability evidence](READABILITY.md) records the
changed edge and removed gold halo. Forge remains unchanged.

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

Evidence is in workspace
`backups/damage-numbers/startup-failure-20261003-084659/eden_log.txt`, SHA-256
`01cd94c2671124977ebd0e71868367cfa841f2214758e1e37c86d5f2bb61a41b`. No emulator
settings, installed packages or saves were changed during this diagnosis.
Removing the capture activation file and restarting restores the previous
inactive behavior; that recovery is unconfirmed by the user.

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
Next manual check: startup and the Forge status panel. A quest test can wait
until startup works.

## 2026-10-03: r2 startup failure and empty-file reproduction

The user reported another crash. The installed r2 NRO matched SHA-256
`fc81dad783daa93321aecc7748a65768d0697cd741123f95a3236e211bbcb9d1`.
Its capture flag was zero bytes and the camera flag was absent. The log confirms
six hooks and camera capture off. No ImGui setup or render-consumer marker runs.
At 13.113296 seconds Forge logs a complete `stbtt_GetFontOffsetForIndex`
assertion in `ImGui_ImplStbTrueType_FontSrcInit`, then the guest process exits.

The log and installed package snapshot are in
`backups/damage-numbers/startup-r2-failure-20261003-090155/`. Log SHA-256:
`a6bef204de6f9cc3077a7e6d2700db7ee8658a1002a2fd0869f4d85e5131930d`. The font
SHA-256 is `9d0d55a303bfd13b79a87721f65185e93f235e2d77fe398b2dca67ac519915f5`;
it matches `tools/forge-local/data/fonts/Roboto-Medium.ttf` byte for byte.

The empty capture flag and font receive the same RomFS offset. A model of Eden's
exact reported revision reproduces its unstable ordering, backward concatenation
end and padding covering the font. An absent flag and flags of 1, 2 and 8 bytes
preserve the font header. Source fingerprints, cases and limits are in
`build/damage-numbers/capture/romfs-flag-probe.json`; details and upstream links
are in [RESEARCH.md](RESEARCH.md).

The corrected activation template contains `1` and a newline. The r2 NRO is
unchanged. No installed file, font, emulator setting or save was changed during
this diagnosis. Next user check: put text in the existing capture flag and
confirm that startup reaches the main menu. That check later passed.

## 2026-10-03: successful capture and transition retention

The user reports r2 ran, showed damage records and completed a quest. After the
quest, the diagnostics cleared before they could show the values. This confirms
user-observed capture, but supplies no per-hit evidence for damage accuracy,
HZV, ownership or critical/bounce correlation.

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
R3's in-game retention check is pending. No installed files, emulator settings
or saves were changed to prepare it.

## 2026-10-03: r3 retention check passed

The user reports r3 works and supplied a screenshot back in the village. It
shows 23 captured hits, the last eight retained records, and zero scope drops,
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

Evidence is in `backups/damage-numbers/capture-r3-retention-20261003-101333/`:
the log, supplied village screenshot, installed package snapshot and
`runtime-evidence.json`. Log SHA-256:
`83e1dde7d67f16ee5661c9712f8ce1b3453511ea8ca00f1999415143e13eb542`. The
installed NRO matches the reviewed r3 candidate, SHA-256
`aaa5b873c786635881e49a5dcf65b4795c9c93683599b627cddb3a275c8a387f`. The capture
flag remains nonempty and the camera flag is absent.

Startup, local hunter/palico capture, hit logging and quest-to-village history
retention passed this manual trial. Independent damage/HP comparison, elemental
assembly, other attack/source cases, multiplayer exclusion and active-camera
projection are pending. No plugin code, installed package, emulator setting or
save was changed while recording this result.

## Prepared r4 camera and floating-number preview

R4 adds native capture timestamps and a renderer to the free-render callback. It
retains the six-hook default and one private arena/TLS slot. The existing camera
opt-in enables all eight hooks and the preview; both activation templates
contain `1` and a newline. The candidate package itself contains only the NRO.
The installed r3 package and emulator configuration are unchanged.

The preview uses the captured final view/projection matrices, a scoped
`uQuestCamera` identity, a snapshot younger than 200 ms and a full-screen native
rectangle whose aspect matches Forge's swapchain. It renders outlined digits,
physical-HZV colors, native critical lines, pop/drift/fade and separate
multi-hit lanes. Diagnostic history survives transitions; floating numbers clear
on player/camera changes or stale/unsupported camera data. Hits delayed longer
than their lifetime are in diagnostics but omitted from the display.

This first visual trial accepts direct hunter/Prowler/palico kinds 0 and 3.
Secondary ownership/source classification, bounce gray, optional sources and
settings persistence are pending. It does not infer a per-hit bounce from the
aggregate feedback getter or change native damage/affinity processing.

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

The user reports player and palico melee popups work well, with accepted impact
positioning, animation and size. They report no popup for palico boomerangs or
thrown barrels and have not encountered positive/negative critical outcomes. The
village screenshot retains 51 hits, 18 preview rejections and zero scope/pool
drops, orphan contributions, overwritten records or read failures.

The retained log contains 18 hunter and 33 local-palico records. Source tag
counts are `000100`: 18, `030403`: 15, `030402`: 15 and `030802`: 3. The latter
18 records are captured correctly as local palico but rejected by r4's preview
source filter. HZVs are -1, 75 and 80; critical bytes are all zero. The camera
log records a 1920x1080 native rectangle and matching Forge display, with four
distinct projected hits. Back in the village, the panel shows no active camera
rectangle and zero visible numbers. The screenshot is diagnostic evidence;
the user's report establishes the actual in-combat visual result.

Evidence is in `backups/damage-numbers/capture-r4-visual-20261003-105730/`: log,
supplied screenshot, installed package snapshot and `runtime-evidence.json`. Log
SHA-256: `67b2eb0c39efa5bad82967e32e7aae4f3da1165a1db2eab877c91b9e90d84ddf`.
Installed r4 NRO SHA-256:
`4e4e1ff78c884f574aa8e5e8d9ac31ffe59d01e4eb6e424fa2e3908d4d89e218`. Both
activation flags are two-byte `1` plus newline files.

R5 admits tags `0x030402`/`0x030802` only for the local Palico.
Camera, visuals and hooks are unchanged. Fixtures cover secondary ownership,
white unknown-HZV barrels, physical projectile HZV, remote/unknown exclusion and
healing rejection: 441 host/248 NRO checks. Live projectiles were pending.

Planned check: Palico boomerang/barrels during normal play; no 100% save/edit
needed. Crit markers awaited equipment. Preparation changed no installed
package, settings, saves or visuals.

## 2026-10-03: r5 projectiles and white coloring accepted

User accepted Palico boomerang/barrel popups and white melee.
Placement/animation/size remain r6 defaults. First 256 logged impacts: known HZV
47–80, eight unknown; white acceptance is the user's observation, not a logged
value.

Installed r5 SHA-256: `81ed488cb38f13c7899e8cf7822b3b235df2d9c87d0fbeb12553e29fd2c244e0`. Evidence: `backups/damage-numbers/capture-r5-accepted-20261003-123254/`; log:
`55995b9d3d30328a6799bb622b036e4b4199d00f9fe785a0eef3e008b17e2cbc`. Crit/multiplayer evidence was pending.

## 2026-10-03: r6 settings, conservative bounce and optional sources staged

R6 adds visibility, size/duration, defaults, Save settings and collapsed
diagnostics. Two checksummed SD preferences; no game-save mount/write. Scoped
SDK FS abort disabling restores prior context on failure. Live persistence was
pending.

Gray bounce needs latest unique hunter contribution/native feedback within 100
ms on the same thread/counter. Ambiguous/old/earlier multi-hits keep HZV color.
Monster dispatch and generic hunter-owned objects have separate default-off
switches. Unknown environmental families/ticks stay hidden; remote
weapons/Palicoes never become objects.

Passed: 470 ASan/UBSan core/presentation checks, mailbox overflow/eight-producer
stress (160,000 packets), 23 transaction cases, 278 compiled SDK/kernel/ImGui
checks and 36 original SDK FS guard/context/decision checks. Fixtures cover
preferences/partial-write recovery, toggle cleanup, remote exclusion and
constrained bounce/native getter; no Eden/GPU trial.

R6 staged SHA-256: `20e2b7ed3133609b2cb625e76d24d394b82b63d3eca77c5ed5d81ef874be14c7`. Eight optional-preview sites, one private
4 KiB arena/TLS slot, 248-byte records. Two-byte activation templates remain
outside the one-NRO package. Preparation changed no installed Forge/Better
Input/damage package, emulator config or save.

Planned check: visibility toggle, save slight size change/restart, restore
defaults; bombs/recoil during normal play. No completed save needed.

## 2026-10-03: r7 automatic startup staged

R7 removes both activation checks/templates at the user's request. Enabling the
Eden package starts eight hooks through Forge. Numbers default on; saved
visibility/in-game switch still apply. Old `.enabled` files are ignored.

R6 signatures, clock/TLS/transaction gates, sites, private 4 KiB arena,
mailbox/source policy and visuals remain. No Forge activation-file FS imports.

Passed: 276 compiled SDK/kernel/ImGui checks covering automatic eight-hook
installation without flag APIs, camera/rendering, saved visibility and
camera-hook modification. Eden startup and r6 features were still pending.

R7 staged SHA-256: `43c6dc8a105789e32e4d22b7e8e514f235e1226641600176ba31527960685021`; package: one 65,536-byte NRO. Installed
r5: `81ed488cb38f13c7899e8cf7822b3b235df2d9c87d0fbeb12553e29fd2c244e0`. Preparation changed no installation, activation or save.

Planned install with Eden closed: replace damage NRO, optionally remove old
flags, hit once; settings/restart check if not already done.

## 2026-10-04: r8 preferences moved beside Better Input

R8 follows Better Input's counted player-save mount: only `savedata:/damage_numbers.ini` and
`savedata:/damage_numbers_backup.ini`, readable/versioned text ≤255 bytes. No SD API; old records
are ignored without migration/deletion. Invalid/missing new INI uses defaults.

Back up valid primary; flush/read-verify/commit replacement (Forge release
commits too). Partial primary uses backup. Close handles/release mount before
restoring FS context on every path. Single render consumer retries loading
once/second if storage unavailable; manual edits cancel it.

Passed: 490 ASan/UBSan core/presentation checks, overflow/eight-producer stress
(160,000 packets), 23 transactions, 285 compiled SDK/kernel/Forge/ImGui checks.
Storage fixtures cover INI/legacy exclusion, preserved game/Better Input data,
backup/primary/commit failure, balanced handles/mounts, delayed loading/manual
edits. No live mount/persistence trial.

R8 staged SHA-256: `aa29f54586068b1b2fdb895beb91cb61b48198df792d585ecafe93e4b79bb0f1`, 69632 bytes; BSS 1,089,536. Sites,
arena/TLS, policy/visuals unchanged. Then-installed r7: `43c6dc8a105789e32e4d22b7e8e514f235e1226641600176ba31527960685021`;
presence proves no new runtime acceptance. Preparation changed no installed
package/save/config.

Planned check: with Eden closed replace NRO; save slight size change/restart and
verify `damage_numbers.ini` beside `better_inputs.ini`; restore/save defaults. No
SD copy needed.

## 2026-10-04: r9 compact plugin rows staged

R9 matches Better Inputs Config under Plugin UI: both framed headers become
closed-by-default, unframed `igTreeNode_Str` rows. Better Input imports
it/`igTreePop`; pinned wrapper forwards to `ImGui::TreeNode`.
Plugin-specific outer/diagnostic IDs and scoped TreePop guards balance early
exits.

Passed: 292 compiled SDK/kernel/Forge/ImGui checks, including imports,
closed-row controls/save omission, numbers while closed, nested balance,
inactive capture and competing consumer. Live row appearance remains
unconfirmed; hooks, settings and visuals unchanged.

R9 staged SHA-256: `58914dd47c797356c892696cc6bdc9bd60543afeb5958f31e91bc40bb0f524fe`. Then-installed r8: `aa29f54586068b1b2fdb895beb91cb61b48198df792d585ecafe93e4b79bb0f1`.
Planned check: replace NRO with Eden closed; compare Damage numbers
row/open-close with Better Input.

AGENTS.md/FORGE_PLUGIN_GUIDELINES.md now require compact tree rows, balanced
pops and independent gameplay callbacks.

## 2026-10-05: r9 manual confirmation

User report: "ive done the tests". R9 [checklist](TEST_CHECKLIST.md): 24 checked boxes;
[results](TEST_RESULTS.md): 16 "good" rows. These are manual gameplay results.

The report confirms:

- Positive and negative LS critical lines, with ordinary hits unmarked.
- HBG Normal S Lv2 impact numbers and positive critical lines.
- Separate Pierce S Lv1 and Pellet S Lv2 impacts, delayed Slicing S Lv2 hits,
  Crag S Lv1 explosions and separate Clust S Lv1 explosions.
- Thunder S Lv1 display and increased total with Thunder Atk, with HZV color
  unchanged. The recorded unmarked values are 133 without decorations and 143
  with the skill. The detail labels it +2, whereas the recipe/result row says
  +1; retain that discrepancy without inferring an exact damage formula.
- Gray numbers for actual bounce with coloring enabled, normal HZV color when
  disabled, and no bounce recoil or gray bounce number with Mind's Eye.
- Diablastazooka Normal S Lv2 negative critical lines.
- Avidya Eye Pierce S Lv1 rapid-fire impact separation.
- Other hunters' and Palicoes' weapon damage hidden during multiplayer, while
  the local hunter's attacks remain visible.

LBG: "separation kinda too spaced". Impact separation passed; spacing remains
cosmetic follow-up. Recording changed no presentation/separation logic.

Quest, monster/part, test date and other gameplay mods are unspecified; r9 is
user-reported, without re-inspecting the live hash. Persistence, compact UI,
Prowler and optional object/monster switches were outside this checklist.

Checkboxes/original observations and this record are in local Git; no binary,
emulator or save/config changes.

## 2026-10-05: settings persistence confirmed

User: "setting persistence is already tested and its working". Persistence after
restart passed separately; no retest needed. Saved setting/test date
unspecified.

At this point, rapid-fire spacing needed polish; Prowler/object/monster switches
lacked confirmation. Recording changed no plugin/save/config.

## 2026-10-05: monster-to-monster damage confirmed

User: "monster to monster also working well". Optional monster damage passed;
pair/switch-off behavior unspecified. This extends the checklist/persistence
reports.

Remaining: wide rapid-fire spacing (cosmetic), Prowler and placed objects.
Recording changed no binary, emulator or save/config.

## 2026-10-05: Prowler mode and bombs confirmed

User: "prowler mode and bomb works". Prowler mode and bomb damage display
passed, completing the remaining Prowler/bomb coverage requested after the
LS/bowgun trial. Specific moves, bomb types and switch-off behavior were not
specified; this does not establish coverage of every environmental source.

All requested damage display groups now have user confirmation. Wide LBG
rapid-fire number spacing remains the only reported issue, a cosmetic follow-up.
Recording changed no binary, emulator or save/config.

## 2026-10-05: r10 staged cosmetics, static checks passed

R10 adds the saved Wilds/Classic selector, embedded Roboto Condensed Semibold
600 digits, warm-white/gold palette, independent red/blue glowing crit stars,
and bounded stable placement. Confirmed bounce coloring, recognized monster
impacts and recognized bomb/object impacts are always on under the master
switch. Unsupported ticks and remote hunter/Palico weapons remain excluded.
The tested eight capture hook sites are unchanged.

Config version 2 requires five fields, rejecting version 1 and obsolete keys.
No legacy preferences migrate; old files stay untouched until explicit Save
settings. The existing counted save mount and transactional backup remain.

Validation:

- 891 host core/presentation checks passed under ASan/UBSan; mailbox
  overflow/eight-producer stress and 23 transactional-hook scenarios passed.
- 306 compiled ARM32 NRO checks passed with explicit SDK/kernel/font/draw
  fixtures. These cover registration ownership/fallback, both styles,
  independent colors, star dimensions/gap/glow/fade, measured different-width
  digits, stable bounded dense placement and the 128-number cap. They also
  check source/master filtering, v2 round trips/recovery, legacy rejection and
  preservation of unrelated files.
- Verified the static font's weight 600, complete digits/fallback glyphs,
  embedded OFL notice, package license, pinned inputs and installed API exports.
  Build uses no dynamic C++ initializers or new capture hooks/arena/TLS slots.
- Python syntax and Git whitespace checks passed.

Candidate NRO SHA-256: `4030d8c4d564b3f61f4b7c3a4b23421a30874be032cbf4d54ebdcf164a05bf6b`
(90,112 file bytes; 1,093,632 BSS bytes).
Package: `build/damage-numbers/capture/candidate-package/`.
Full provenance/import/source manifest and fixture report:
`build/damage-numbers/capture/candidate-manifest.json` and
`build/damage-numbers/capture/compiled-capture-probe.json`.
Font subset SHA-256: `892deb9800fa0261d2dc07fe58972746ee680a545540eed865e64e4e59c73c8d`
(10,372 bytes). Source and license hashes are retained in
`fonts/source.json`; the generated manifest also records height normalization.

Installed r9 SHA-256 was rechecked unchanged:
`58914dd47c797356c892696cc6bdc9bd60543afeb5958f31e91bc40bb0f524fe`.
A matching rollback copy is retained under `backups/damage-numbers/r9/` outside
Eden. No installation, emulator launch or real save write occurred.

The fixtures inspect draw arguments and modeled storage; they do not prove
Forge GPU output or Eden gameplay. R10 manual confirmation remains pending:
three reference crit cases, two LBG bursts, style switching and one saved-style
restart. Earlier accepted combat coverage is retained in TEST_RESULTS.md.

## 2026-10-05: r11 screenshot matching, staged

The user described r10 positive/negative crit symbols as near-perfect, then
supplied Wilds/mod screenshots showing thin/narrow digits and a star core that
needed more thickness. They clarified that weakspot digits should glow instead
of using the r10 dark shadow. Their existing r10 checklist has all six cosmetic,
spacing, style-switch and saved-style restart items checked; those edits are
preserved without changing or committing their checklist in this update.

R11 replaces Condensed Semibold 600 with full-width Roboto Bold 700. The zero's
ink width/height ratio changes from 0.554 to 0.658, keeping height normalization
at 1.0. Star height is 0.30 rather than 0.24 font sizes, with a fuller waist and
brighter pale core; its width, polarity/colors and default gap remain. Stars
now center on the actual pixel-rounded glyph bounds. Weakspot digits use a
symmetric warm halo behind crisp fill and no black shadow. White/gray digits
retain a small shadow. Classic, capture hooks, damage/source behavior, animation,
settings format and cached bounded offsets remain unchanged.

Validation:

- 895 ASan/UBSan host checks passed, including thicker marker bounds, actual
  glyph centering and preserved Classic origins. Mailbox stress and 23
  transactional-hook scenarios passed.
- 312 compiled ARM32 NRO checks passed with explicit fixtures, including
  font registration/ownership/fallback, warm halo without black shadow,
  symmetric rounded halo offsets, fractional-position star centering for
  both polarities, final fade and the 128-number dense limit.
- Static TTF checks confirm width 100, weight 700, complete digits/fallbacks
  and no remaining variable axes. Python syntax and Git whitespace passed.
  Installed Forge exports and the baseline input hashes were checked by build.

Candidate NRO SHA-256: `2b55907b6f19b57f04d776db75eba0a209e6c4bccf0e9d2e46ee9f610eae6330`
(90,112 file bytes; 1,093,632 BSS bytes).
Font subset SHA-256: `495d951b5c118e5c1169928eb8a606d4bf79f945f6f654e2dc34515eb96d0b1b`
(10,440 bytes). Source/axes/license hashes:
`fonts/source.json`; package license: `Roboto-OFL.txt`.

Installed r10 was verified as
`4030d8c4d564b3f61f4b7c3a4b23421a30874be032cbf4d54ebdcf164a05bf6b`,
with a matching rollback under `backups/damage-numbers/pre-r11/` outside Eden.
The installed binary, game saves and emulator settings were not changed.

The fixtures record modeled font/draw calls and do not execute the Forge GPU
renderer. R11 appearance still needs a short comparison in-game, described in
R11_VISUAL_CHECK.md. Earlier damage/persistence coverage needs no full repeat.

## 2026-10-05: r12 original Wilds assets, staged

The user confirmed the supplied reference shows default Wilds damage numbers,
not the Enhanced Damage Numbers mod present in their installation. Selective
read-only extraction found UI020020's font slot, original Frutiger Neue LT W1G
Medium font, the red/blue critical sprites and native text blur properties.
Original assets and generated payloads stay in ignored work/build directories;
tracked recipes and WILDS_ASSETS.json retain provenance. Wilds was not launched
or modified.

R12 replaces Roboto with the original digit outlines, converted from cubic CFF
to quadratic TrueType at maximum error 0.25/1000 font units. The height factor
0.8621484555 retains accepted visible height. Original RGBA critical sprite
crops replace triangle stars, preserving their baked pale center and bloom.
Gaussian effect masks replace the repeated text outline with smooth centered
shading and a gold halo. The compact spacing/caches, animation, Classic style,
config v2, source policies and eight tested capture hooks remain unchanged.

Validation:

- 895 ASan/UBSan host checks, eight-producer mailbox stress and 23 transactional
  hook scenarios passed.
- 312 compiled ARM32 NRO checks passed. They inspect font registration and
  fallback, once-per-context texture registration, decoded texture fingerprints,
  original red/blue sprite selection, separate digit colors, marker placement,
  fade, stable spacing, 128-number limit, filtering and configuration recovery.
- A separate harness executes the pinned real ImGui font atlas, custom texture
  registration and draw-list code. Its actual draw commands are composited with
  linear texture sampling into `work/damage-numbers/host-visual/preview.png`.
  This exercises the original digit shapes and crit pixels without an Eden GPU
  or runtime claim. Reproduce with `scripts/test_damage_numbers_visual.py`.
- Offline ARM32 build verifies installed Forge exports and preserved input
  fingerprints. Python syntax, JSON and Git whitespace checks passed.

Candidate NRO SHA-256:
`1a851eb2947a88ec3bb06685c5b852d2324b742ff0420d56fee13a1e31bf7b3f`
(188,416 file bytes; 1,093,632 BSS bytes). Embedded font: 2,352 bytes, SHA-256
`b2a79e1f114f576f4cf47f68547cab3202f1e48a1b2864e7557984ad5e061b2c`.
Effects RLE: 109,915 bytes; decoded RGBA SHA-256
`a121a8925f60bfc6adc9ac56428e4118ad702cc95dbb9d7962f2a0b5bc62bbd4`.
Package: `build/damage-numbers/capture/candidate-package/`, with local-asset
notice. Full build/fixture reports remain beside the package.

Installed r11 was rechecked unchanged:
`2b55907b6f19b57f04d776db75eba0a209e6c4bccf0e9d2e46ee9f610eae6330`.
Matching rollback: `backups/damage-numbers/pre-r12/` outside Eden. Saves,
emulator settings and installed packages were not changed. The user's completed
r10 checklist edits remain untouched and uncommitted.

Original font outlines and crit pixels remove the asset approximations, but
Forge/STB rasterization and Gaussian conversion still differ from RE Engine's
blur shader. A pixel-perfect final result is not established. The brief
R12_VISUAL_CHECK.md requests white/gold/crit shading comparison and a couple of
LBG bursts; completed combat and persistence coverage needs no full repeat.

## 2026-10-05: r12 accepted and packaged

The user completed all three R12_VISUAL_CHECK.md checks, then confirmed:
"all works well, good job, now it is complete". The earlier six r10 checklist
items remain checked. Their recorded results are preserved and included in the
completion commit. No further gameplay confirmation is required for this scope.

The installed plugin now matches the tested r12 candidate SHA-256
`1a851eb2947a88ec3bb06685c5b852d2324b742ff0420d56fee13a1e31bf7b3f`.
This replaces the previous observation of installed r11; installation was
performed outside this packaging action. The exact candidate is frozen in
`build/damage-numbers/releases/r12/damage-numbers-r12.zip`, with local asset
notice, installation notes, acceptance manifest and file hashes. ZIP contents
are reopened and compared byte-for-byte with the tested input. Release
acceptance is separate from the historical pending candidate-build status.

Packaging changes no NRO bytes, installed files, save/config contents or
emulator settings. R11/r10/r9 rollback material remains outside Eden.
The final appearance is accepted by the user; no pixel-perfect renderer claim
or public release is made.

Final ZIP SHA-256:
`7c0a4c50b4f02fe151c71ed49a40ae95986da97b8341cc85a795de9e026f67e9`.
Two packaging runs produced identical bytes. Reopened ZIP CRC, exact member
inventory and every payload comparison passed; installed r12 stayed unchanged.

## 2026-10-05: alternate open visual assets

Requested after completion of the personal r12 edition. The alternate uses a
Roboto Semibold 600 digit subset named Damage Digits Open under SIL OFL 1.1,
and independently authored procedural four-point crit art. The font's full
notice is embedded and included; original art has a separate notice.
No extracted Wilds font/texture is included. Gameplay/source/render policies,
spacing, animation, config paths/version, Classic mode and UI are unchanged.
Both editions are alternatives for the same plugin, not simultaneous installs.

Generation and build outputs are isolated under `open-assets/`; the accepted
r12 NRO/ZIP and installed plugin remain unchanged. The font and atlas input
pipeline is audited with proprietary visual file reads blocked. Compiled font,
full license and RLE sprite buffers are compared to the open payloads. The
real-ImGui preview uses the alternate font/effect texture at
`work/damage-numbers/open-visual/preview.png`.

Alternate NRO SHA-256:
`87390164d5541d0922f072a95033fc7bf61e3687e3a23e1b5ae25154196d9ab2`.
The full build, asset audit and compiled fixture reports are recorded under
`build/damage-numbers/open-assets/`. The package recipe requires a passing
312-check compiled report with the same hash before creating the ZIP.
The alternate's in-game appearance is pending; earlier gameplay coverage
remains evidence for the unchanged implementation. No installation, game
launch, save write or publishing is part of this alternate build.

Alternate verification completed: 312 compiled ARM32 checks passed, including
full registered-font fingerprints, decoded sprite/effect fingerprints, both
styles, fading, spacing, source exclusion/master switch and save recovery.
Real pinned ImGui font/texture rendering and the proprietary-read-blocked asset
audit passed. Python syntax and Git whitespace passed. The NRO is 212,992 bytes
with 1,093,632 BSS bytes; no capture/renderer C++ implementation was changed.
The default profile still regenerates the exact accepted r12 font/RLE payloads,
and the original final ZIP and installed NRO retain their recorded hashes.

## 2026-10-05: r13 open Wilds animation trial

User requested a 1.2x stamp settling in 0.2 seconds, a stationary interval,
then faster upward motion during disappearance. Only the open-asset build's
Wilds style changes. A compile-time profile flag selects this behavior; the
personal game-asset edition and Classic retain their previous animation.

Duration stays the total display lifetime and settings/codec/defaults are
preserved. The final 0.2 seconds rise 30 pixels at 720p while fading. At the
existing 0.9-second default, the number holds its projected position for 0.7
seconds; a 1.2-second Duration gives a 1-second hold. The initial 0.2-second
size settlement uses quadratic easing. The maximum placement reservation is
now the 1.2x stamp for this style, with stable bounded offsets unchanged.

Host checks cover both profile builds, including hold timing, attached critical
markers, total expiry, minimum duration, resolution scaling and Classic's old
animation. Compiled fixture checks inspect stamp, settled size, fully opaque
hold, fast exit and Classic preservation. The open font/texture provenance
and licenses remain unchanged. This trial is staged separately as r13-open-assets;
no game-asset version is retired and no installation or save write is performed.

Verification passed: 917 open and 900 legacy host core/presentation checks,
the eight-producer mailbox stress and 23 transactional-hook scenarios,
321 compiled ARM32 checks, the pinned real-ImGui visual harness, and the
open-asset provenance/compiled-payload audit. Python syntax and Git whitespace
checks passed. The NRO is 212,992 bytes with 1,093,632 BSS bytes, SHA-256
`1937e96bcb3b52bac86a7252e1967ba5799cd466930d51a0f19b34add9e869e8`.
The existing r12 releases and installed personal NRO retain their hashes;
the r12 open NRO is also preserved under workspace backups for rollback.
Fixtures are not an Eden gameplay trial. The brief open-edition animation
check remains pending in [OPEN_ASSETS.md](OPEN_ASSETS.md).

## 2026-10-05: r14 open font and crit bloom trial

The user requested stronger crit bloom at the accepted star dimensions and
fuller readable digit curves. Twelve OFL families were compared at weight 600
and equal visible height. Hind was rejected; the user selected Open Sans
Semibold. [Comparison notes](FONT_COMPARISON.md) and pinned source/license hashes
retain the research. The subset is renamed Damage Digits Open and its complete
Open Sans OFL notice replaces the embedded/package Roboto notice. Historical
Roboto licensing is preserved separately.

The procedural star adds a broad colored halo and horizontal streak behind
the unchanged polygon and pale core. Its 88x48 texture bounds and render scale
are preserved independently of font normalization, with no new draw calls.
The accepted open r13 stamp/hold/exit timing, capture code, source filtering,
Classic style, saved preferences and compact Forge panel are unchanged.
The personal game-asset edition is preserved; no installation, game launch
or save changes are part of this trial.

R14 verification passed: all 321 compiled ARM32 fixture checks, the real pinned
ImGui font/texture/draw preview, proprietary-read-blocked asset provenance and
compiled-payload audit, all three comparison groups, Python syntax and Git
whitespace checks. The unchanged r13 presentation/policy core retains its
earlier host-test evidence; no new host combat test is claimed. These fixtures
do not run Eden's GPU renderer or establish camera-motion readability in-game.

NRO SHA-256: `be263fbebf8fe2057514aabda07c12724bb0706aad2592ee1c3761100603f158`.
File size is 262,144 bytes; BSS remains 1,093,632 bytes. Open Sans subset SHA-256:
`24b1f1efddfd8107ec1cbb1ca866ab50632618794933989eed9968791d72c4ef`.
Procedural effect RGBA SHA-256:
`fe15190c98259443a01add264777b2b4bee994bbd58f9c8f5f12338abb399f2b`.
The r13 open and r12 personal ZIPs retain their recorded hashes; the r13 NRO
is preserved in workspace backups outside Eden. A fresh installed-file read
matched r13 open, which this task did not replace. The r14 visual trial remains
pending; [OPEN_ASSETS.md](OPEN_ASSETS.md) contains the brief confirmation checks.

## 2026-10-05: private edition retired; open r14 is the default

At the user's request, the game-derived edition is retired. The installed NRO
already matches open r14 SHA-256 `be263fbebf8fe2057514aabda07c12724bb0706aad2592ee1c3761100603f158`. No installed files or saves are changed. Default build, font/sprite
preparation, real-ImGui preview and packaging now use only open assets. The
private extraction/conversion entry points and legacy animation branch are
removed. Historical provenance moved to `history/r12/`; private payloads were
archived outside active output directories with a 160-file hash inventory.

Fresh host verification passed 917 core/presentation checks, eight-producer
mailbox stress and 23 transactional-hook scenarios. The rebuilt NRO matches
the installed and previously compiled-tested r14 bytes. The existing 321 ARM32
checks cover the identical NRO. The asset audit and real-ImGui preview passed. This cleanup
adds no fresh in-game validation claim. See [RETIREMENT.md](RETIREMENT.md).

## 2026-10-05: r15 finished, 20px default, ready for distribution

The user selected 20px at 720p as Number size 1.0 and requested finished status.
The new base is 30px at 1080p, 40px at 1440p and 60px at 4K. Marker gaps and
collision margins retain the selected preview proportions. Stamp/hold/exit
behavior, font/art data, capture hooks and version-2 config fields are unchanged.
Saved scale values multiply the new base; no save is edited or migrated.

Verification passed: 919 host core/presentation checks, eight-producer mailbox
stress, 23 transactional-hook scenarios, all 324 compiled ARM32 NRO checks,
real-ImGui rendering at the new default, the refreshed six-size chart and the
proprietary-read-blocked asset audit. The first compiled run exposed two stale
45px expectations for saved Classic size 1.5; the corrected 37.5px expectations
passed in the complete rerun. No game behavior fix was needed for that fixture.

NRO SHA-256: `7c112c6bf497207f8bdf475687bab3b0056c0726ac4db1707db8f8adc01e07da`.
Size remains 262,144 bytes and BSS 1,093,632 bytes. Font and procedural RGBA
hashes are unchanged from r14. The installed r14 NRO and r14 ZIP are preserved;
a hashed r14 candidate/fixture/audit rollback copy is under
`backups/damage-numbers/r14-before-r15-20261005/`. R15 is staged at
`build/damage-numbers/releases/r15/damage-numbers-r15.zip`; installation is
separate. Prior user-confirmed gameplay coverage is retained, with no fresh
r15 game run claimed. The release status is `FINISHED_DISTRIBUTION_READY`.

## 2026-10-05: clean r15 installable archive

The first r15 ZIP put eight README/license/provenance/checksum files inside the
installable `damage-number` folder. The user requested modern-control's clean
layout. The packager now asserts one member: the NRO at its runtime path.
README, combined font/art/dependency notices, full source/audit release manifest
and checksums remain beside the ZIP. Notices accompany redistribution.

The previous package is preserved under
`backups/damage-numbers/r15-before-clean-package-20261005/` with verified hashes.
No plugin behavior changes: the NRO remains
`7c112c6bf497207f8bdf475687bab3b0056c0726ac4db1707db8f8adc01e07da`;
the completed 324-check ARM32 report remains applicable to these same bytes.
No gameplay rerun or installation is part of this packaging correction.

Clean packaging validation reopens a one-member ZIP, compares its NRO with the
324-check report, checks every adjacent checksum, and verifies full font/art/
dependency notices. Two packaging runs produce identical bytes. A fresh read
of the installed NRO now matches r15; this packaging task did not install it.

## 2026-10-05: r16 lossless embedded artwork compression

At the user's request, replace pixel RLE with an owned bounded DN-LZ1 decoder.
The 64,170-byte compressed stream reproduces the exact 524,288-byte RGBA
atlas (`fe15190c98259443a01add264777b2b4bee994bbd58f9c8f5f12338abb399f2b`).
The NRO becomes 155,648 bytes (152KiB) from 262,144 (256KiB); BSS remains
1,093,632 bytes and imports are unchanged. The native decoder runs only at
ImGui context initialization, directly into Forge's texture allocation.
No per-hit decoding, persistent scratch storage or extra draw calls are added.

The host decoder median is 143.431µs, compared with the previous RLE's
139.387µs, measured across nine batches of 100 decodes. This is initialization
CPU timing on the host, not Eden/ARM/GPU performance. Exact real-ImGui texture,
vertex/index/command and composited-preview comparisons pass. All 338 compiled
ARM32 checks and the host/codec/provenance suites pass. Prior gameplay evidence
remains applicable; no new runtime test is claimed.

R15 input artifacts are hashed and preserved at
`backups/damage-numbers/r15-before-compression-20261005/`. The clean r16 ZIP
contains only the runtime NRO, with notices and provenance beside it.
Installed r15, Forge, Better Input, saves and configuration are left unchanged.
The known multi-draw-list corruption in Forge's renderer remains unresolved.
See [compression details](COMPRESSION.md).

## 2026-10-06: r17 NZCVQ-only gateway compatibility candidate

The shared restore macro now emits `msr APSR_nzcvq, r2` (`e128f002`) rather
than `msr APSR_nzcvqg, r2` (`e12cf002`). Exactly nine words change across the
eight production entries and return gateway. This is bounded by the original
MHGU 1.4.0 eight-site GE-liveness audit. GE remains as set by callbacks; it is
neither restored nor zeroed. The stale fixture-only entries-6/7 comment is
corrected without changing hooks or the 328-byte frame.

The NRO is `1eb78f3430f6c5c5ce1f60a382fa8bd4fc18ee56cf267649b95a94a462115ea4`,
155,648 bytes. Every other executable byte matches audited r16; initialized
data, imports, BSS and segment sizes also match. Font, compressed art,
20px default, animation, UI and version-2 settings remain unchanged.

Fresh gateway coverage passes 9,287 checks across 2,025 calls, including
256 explicit GE-boundary cases (16 patterns × eight entries × preserving/
clobbering callbacks). The complete production fixture passes 347 checks,
including nine emitted-instruction checks. Host and appearance/provenance
checks pass. Original r16/gateway/work/source artifacts and installed-file
fingerprints are preserved at `backups/damage-numbers/r16-before-r17-ge-20261006/`.
No installed files, emulator binaries, configurations, caches or saves are
changed. At preparation, this was a validated compatibility candidate awaiting
a user-run quest check. The subsequent confirmation below records r17 working
on both emulators. See [compatibility notes](RYUJINX_COMPATIBILITY.md).

## 2026-10-06: r17 confirmed on Eden and Ryujinx-based emulators

User report: "it is already confirmed to work on both, so please update the status".
Gameplay confirmation is recorded for both tested installations. Both installed
NROs freshly match the released r17 SHA-256:
`1eb78f3430f6c5c5ce1f60a382fa8bd4fc18ee56cf267649b95a94a462115ea4`.
The report confirms working gameplay; it does not enumerate individual smoke
check actions or a repeat of every prior combat test.

Latest logs were reviewed before updating the status. Copies, original paths,
hashes and installed-file fingerprints are preserved under
`backups/damage-numbers/r17-runtime-confirmation-20261006/`.

- Eden: `eden_log.txt`, SHA-256
  `49a360e21e23936198812f3fe1a145091d7138e912c8d7084ff800013a7b777f`.
  Eden build `master-10bcd2d849-master` boots MHGU 1.4.0, main build
  `FB08F1D20FD1204FE7070672777F869B`. It logs Forge
  `1.4.1-originalbi1`, loading Better Input, Damage Numbers, Key Quest Markers
  and Modern Weapon Controls. Damage capture's eight-hook startup, ImGui setup
  and render TLS setup complete. The inherited "Capture r11" message is not
  the package revision; the installed-file hash identifies r17. The log does
  not enumerate the complete named resource-mod set.
- Ryujinx: `Ryujinx_1.8.12_2026-10-06_13-49-59.log`, SHA-256
  `ab04a03878ac02ab6ee4643b408fcd19d56c532f43f1922955a727adeddb17b7`.
  The log includes an earlier different game; only its MHGU launch segment
  is used here. MHGU 1.4.0/main build `FB08F1D20FD1204FE7070672777F869B`
  loads with enabled `forge`, `better-inputs`, `damage-number`,
  `key-quest-markers`, `modern-control`, `HD Project`, `Z_Balanced` and
  `FOV (Medium)`. The `forge-patched` and `better-inputs-patched` packages
  are disabled in this run. Forge reports version 1.4.1; the enabled cheats
  include 60 FPS, quest-result skip, small-monster marking, Great Luck,
  crafting-requirement removal and lock-on target.

Neither inspected log contains an `APSR_g` failure. They are not error-free:
both contain the shader gpuCode 1.16 diagnostic; Eden also contains GPU macro
assertions, an unmapped-memory message and a forced stop, while Ryujinx contains
audio-pool unmap failures and a GPU syncpoint timeout. These records do not
establish their cause or prove clean shutdown. They remain separate from the
user's successful damage-number gameplay report. The known Forge overlay
menu/popups issue also remains open.

Distribution remains `READY_TO_DISTRIBUTE`; fresh runtime confirmation is now
recorded. Support metadata is refreshed with a verified backup while r17 NRO
and ZIP bytes remain unchanged. This status update installs, launches and
publishes nothing.

## 2026-10-06: distribution status

The user directed that damage numbers r17, modern weapon controls r10 and
key quest markers r1 all be marked ready to distribute. R17's distribution
status is `READY_TO_DISTRIBUTE`. This status update preserves the validated
binary and recorded evidence, including the confirmed r17 gameplay on both
emulators above. The label update ran no additional gameplay test. No public
publication was performed.

## r18 standalone/settings/log cleanup

This binary has its own fresh compiled/static checks, recorded in CLEANUP_VALIDATION.json. No new gameplay test was run; the preceding observations retain their original version and scope. Forge remains unchanged.

## 2026-10-08: primary-only config release acceptance

The user reported "all works well" and requested promotion to ready to
distribute, keeping Forge LOCAL_ONLY. The tested Eden installation matches
r22-config1 NRO SHA-256 `c9e1f0740e12bb2806926ef985f575221aa2785384be6a064fd8181b47567f70`. The supplied reward-screen screenshot
shows the Forge menu and PDB gold highlights visible together. Eden's retained
log records Forge 1.4.1-render1, PDB r17-config1 and successful Damage Numbers
settings saves. Damage Numbers still prints its older r20 startup label; its
installed file hash identifies the tested r22-config1 bytes.

[Release runtime evidence](docs/CONFIG_RELEASE_RUNTIME.json) records the
log/screenshot hashes and scope. This is the user's confirmation of the tested
setup, not exhaustive dropdown/transition/save-restart coverage or a clean
shutdown claim. The newest Ryujinx log used older plugin bytes and is not
confirmation of this release. Distribution preparation rebuilds no runtime,
installs nothing and publishes nothing. Earlier entries remain historical.

## 2026-10-08: r23-config2 config simplification

The config codec now writes no version marker and reads existing `version=2`
files without changing their preferences. Host tests execute production storage
with a recording filesystem; compiled ARM32 fixtures cover old-file load,
explicit versionless save and restart. These are simulated storage results,
not a new emulator/game run. Earlier runtime reports keep their original
version and binary scope. Nothing was installed or activated by this task.
