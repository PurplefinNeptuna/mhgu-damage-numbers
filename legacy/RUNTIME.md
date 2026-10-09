# Damage Numbers runtime evidence

Current release is r25. Build, asset/import and emitted gateway checks passed;
the user accepted the 1080p size preview and explicitly skipped further behavior
suites and gameplay. [R25_SIZE_CHANGE.json](docs/R25_SIZE_CHANGE.json) records
that scope. Earlier confirmations apply to their recorded versions and bytes.

## Early capture trials, 2026-10-03

R2 started successfully after its empty activation flag was replaced with text.
The empty-file RomFS/font overlap and SDK TLS setter ABI findings remain in
[RESEARCH.md](RESEARCH.md). R3 confirmed 23 hunter/Palico records retained after
a quest-to-village transition, with zero reported capture faults; that sample
had HZV 80 and no critical outcomes. R4's hunter/Palico melee placement, animation
and size were accepted. R5 added accepted Palico boomerang/barrel display and
white coloring. Per-hit HP accuracy and broader coverage were not established
by those early samples. Raw evidence remains under the dated
`backups/damage-numbers/capture-*` directories; superseded trial notes remain in Git.

## R9 combat and R10/R12 visual reports, 2026-10-05

| Report | User-confirmed scope |
| --- | --- |
| R9, all 16 result rows | LS positive/negative crits; HBG Normal, Pierce, Pellet, Slicing, Crag, Clust and Thunder impacts; bounce/Mind's Eye; LBG rapid fire; remote hunter/Palico weapon exclusion. |
| Separate R9-era reports | Settings persistence, monster-to-monster damage, Prowler and bombs. Specific moves, monster pair, bomb types and switch-off behavior were not supplied. |
| R10 cosmetic checklist | Gold positive/negative and white positive crit markers; compact stable LBG bursts; Classic/Wilds switching and saved-style restart. |
| R12 final visual checklist | Positive/negative crit appearance and fade, LBG bursts, Classic/Wilds switching; user confirmed completion. This was the retired game-derived edition. |

R9 was user-reported without a fresh installed-hash check; quest/monster/part and
other gameplay mods were not supplied. Orcus Quarcus Thunder S Lv1 totals were
133 without decorations and 143 with Thunder Atk, with unchanged HZV color.
The detailed report says +2 while its recipe/result row says +1; exact skill
level and formula remain unverified. Wide LBG spacing prompted R10 cosmetics.
Test setups/checklists remain recoverable from Git history.

The tested R12 NRO was
`1a851eb2947a88ec3bb06685c5b852d2324b742ff0420d56fee13a1e31bf7b3f`.
Its visual acceptance does not establish pixel equality with RE Engine or
gameplay validation of the later open font. R15's default size and R16 compression
had host/ARM/real-ImGui evidence without a fresh game run; the compression
details remain in [COMPRESSION.md](COMPRESSION.md).

## R17 confirmed on Eden and Ryujinx, 2026-10-06

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


## R18 through R21 presentation changes

R18 standalone/settings/log cleanup has separate static evidence in
`CLEANUP_VALIDATION.json`, without a new game run. R19's half-pixel outline and
R20's 1.5× crit stars were accepted from real-renderer previews; their host/ARM,
asset and gateway checks passed without fresh gameplay. R21-rise1 introduced
Rise through host/ARM and real-ImGui previews, with no gameplay acceptance.
Current presentation rules are in [RESEARCH.md](RESEARCH.md).

## Primary-only config acceptance, 2026-10-08

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

## 2026-10-08: r24-savehint save-location wording

The success message names this plugin's INI and MHGU's save folder. Public
instructions use the same location without referencing another plugin.
Persistence, config fields, hooks and gameplay behavior are unchanged.
Scoped validation is recorded in [SAVE_HINT_VALIDATION.json](docs/SAVE_HINT_VALIDATION.json).
No new gameplay run or installation was performed for this wording revision.

## 2026-10-08: r25 user-approved base sizes

The user accepted the 1080p renderer preview and requested r25 ready to
distribute without further testing. At scale 1.0, 720p bases are 24 px for
Wilds/Classic and 20 px for Rise; Rise keeps its 1.4× weak-point boost.
At 1080p, settled weak-point numbers are 36/42/36 px in Wilds/Rise/Classic order.
Build/static/package audits are recorded in [R25_SIZE_CHANGE.json](docs/R25_SIZE_CHANGE.json).
No fresh host/compiled behavior suite, game run or installation was performed.
Earlier gameplay reports retain their original version and bytes.
