# Damage numbers: test results

R9 report, 2026-10-05: all 16 rows passed. The completed [TEST_CHECKLIST.md](TEST_CHECKLIST.md) covers
r10 cosmetics; these r9 results remain the completed combat evidence. LBG
spacing prompted the r10 changes. Blank metadata was not supplied.

Date:

Mod build/revision, if known: r9

Quest/monster(s):

Other gameplay mods, if relevant:

## Main checks

| Check | Result | Short notes |
| --- | --- | --- |
| LS positive crit: upper line; ordinary hit unmarked |good| |
| LS negative crit: lower line; ordinary hit unmarked |good| |
| HBG Normal S Lv2: individual impacts and positive crit markers |good| |
| HBG Pierce S Lv1: separate successive impacts |good| |
| HBG Pellet S Lv2: separate pellet impacts |good| |
| HBG Slicing S Lv2: delayed cutting impacts |good| |
| HBG Crag S Lv1: delayed explosion |good| |
| HBG Clust S Lv1: separate explosions |good| |
| HBG Thunder S Lv1: damage displays |good| |
| Thunder Atk +1 comparison: total increases, HZV color unchanged |good| |
| Actual bounce: gray with bounce coloring enabled |good| |
| Actual bounce: normal HZV color with bounce coloring disabled |good| |
| Mind's Eye: no recoil and no gray bounce number |good| |

## Optional checks

| Check | Result | Short notes |
| --- | --- | --- |
| Diablastazooka Normal S Lv2: negative crit lower line |good| |
| Avidya Eye Pierce S Lv1: rapid-fire impacts remain separate |good|separation kinda too spaced|
| Multiplayer: other hunter/Palico weapon hits hidden; yours visible |good| |

## Additional confirmation: settings persistence

Passed, confirmed 2026-10-05 separately from the 16 checks. No persistence
retest needed.

## Additional confirmation: monster-to-monster damage

Passed, 2026-10-05: "monster to monster also working well". Monster pair and
switch-off behavior were not specified.

## Additional confirmation: Prowler mode and bombs

Passed, 2026-10-05: "prowler mode and bomb works". This completes the remaining
Prowler and bomb damage display checks. Specific moves, bomb types and
switch-off behavior were not specified.

## Elemental comparison values

Weapon: Orcus Quarcus

Ammo: Thunder S Lv1

Quest, monster, part and state:

Unmarked values without decorations: 133

Unmarked values with three Bolt Jwl 2 and active Thunder Atk +2: 143

Color before/after:

Observed total: 133 to 143, with HZV color unchanged. Details say Thunder Atk
+2; recipe/result row says +1. Preserve both reports; exact skill level/full
formula remain unverified.

## Failed or uncertain case

Duplicate this block only if needed.

- Check:
- Weapon and ammo/move:
- Armor setup and displayed affinity:
- Monster, part and state:
- Relevant mod setting:
- Expected:
- Observed:
- Screenshot/clip, if available:

## R10 cosmetic results (completed)

| Check | Result | Short notes |
| --- | --- | --- |
| Gold digits / positive red star | Passed | User completed checklist |
| Gold digits / negative blue star | Passed | User completed checklist |
| White digits / positive red star | Passed | User completed checklist |
| Two LBG bursts: compact spacing and stable older numbers | Passed | User completed checklist |
| Classic / Wilds switching | Passed | User completed checklist |
| Saved style restored after one restart | Passed | User completed checklist |

## R12 final acceptance: 2026-10-05

All three rows in [the historical r12 visual checklist](history/r12/R12_VISUAL_CHECK.md) were checked by the user: white/gold
positive crit appearance, negative crit appearance/fade, and LBG bursts plus
Classic/Wilds switching. The user then confirmed: "all works well, good job,
now it is complete". No additional gameplay tests are pending.

Installed plugin and tested candidate both have SHA-256
`1a851eb2947a88ec3bb06685c5b852d2324b742ff0420d56fee13a1e31bf7b3f`.
The final package preserves those exact bytes. Appearance is user-accepted;
pixel-perfect equality with RE Engine has not been measured. Automated fixture
coverage remains separate from the user's real-game confirmation.

## Final r15 release verification — 2026-10-05

Finished and ready for distribution at the user's request. The approved open
appearance uses 20px at 720p as 1.0, or 30px at 1080p. Passed: 919 host
core/presentation checks, mailbox stress, 23 transaction scenarios, all 324
compiled ARM32 checks, real-ImGui preview and complete font/art provenance audit.
New compiled checks confirm 20px/30px defaults and 24px at 1.2; persistence
checks confirm existing scales remain multipliers of the new base. Historical
gameplay confirmations above remain the manual evidence; no new r15 gameplay
run is claimed. No further exhaustive manual checklist is requested.

## R16 compression verification — 2026-10-05

The 152KiB plugin preserves the accepted r15 artwork, font and real-ImGui
rendering byte for byte. Passed: 13 deterministic codec round trips, 20,000
malformed streams under ASan/UBSan, explicit guard/truncation checks, the
919 core/presentation checks, mailbox stress, 23 transaction scenarios and
338 compiled ARM32 checks. New compiled checks cover exact texture decode,
per-context initialization, no decoding during hit rendering, malformed
copies and cleanup/logging of invalid artwork.

BSS and imported APIs match r15; the GPU texture and draw output are unchanged.
The host decoder median is 143.431µs once per UI context, versus 139.387µs for
RLE. These are host CPU measurements, with no new gameplay or live FPS result
claimed. Installed r15 and its rollback materials remain unchanged. Detailed
evidence: [COMPRESSION.md](COMPRESSION.md). The known Forge overlay corruption
remains open.
