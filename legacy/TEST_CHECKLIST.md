# Damage numbers: in-game checklist

LS/bowgun checks cover crits, impact separation, element and bounce. Placement,
animation, size, white melee and Palico melee/boomerang/barrels already passed.

Stop after a couple of clear examples; returning early is fine. Crits usually
need 10 to 20 clean hits. Bad luck alone is not failure.

Record results in [TEST_RESULTS.md](TEST_RESULTS.md). Check boxes only for observed passes;
leave skipped/uncertain cases unchecked.

## Equipment preparation

Full Neset; no talisman; weapon slots empty. Upgrade defense for comfort.

| Slot | LS armor | Bowgun armor | Positive setup decoration |
| --- | --- | --- | --- |
| Head | Neset Helm | Neset Hat | Expert Jwl 2 |
| Chest | Neset Mail | Neset Vest | Expert Jwl 2 |
| Arms | Neset Gloves | Neset Brachia | Expert Jwl 3 |
| Waist | Neset Coat | Neset Elytra | Expert Jwl 2 |
| Legs | Neset Boots | Neset Feet | Expert Jwl 3 |

Slots: 2/2/3/2/3. Expert 19 + Neset Skill +2 = 21, activating Critical Eye +3
(+30% affinity). Sources: [Blademaster Neset](https://mhgu.kiranico.com/armor/59bb6), [Gunner Neset](https://mhgu.kiranico.com/armor/54be8),
[Expert](https://mhgu.kiranico.com/skill/4ee04), [Skill +2](https://mhgu.kiranico.com/skill/4ea5e).

- [x] Prepare the positive setup using the decorations above.
- [x] Prepare a neutral setup with all decorations and the talisman removed.

Check displayed affinity before quests; record differences caused by gameplay
mods.

## 1. LS critical markers

Leave Palicoes behind. On a familiar High Rank monster, repeat ordinary slashes
into one part with consistent sharpness/spirit gauge color.

| Check | Weapon | Armor setup | Expected affinity |
| --- | --- | --- | ---: |
| Positive crit | Avidya Saber | Positive Blademaster | +75% |
| Negative crit | Tigrine Edge LV1 | Neutral Blademaster | -20% |

Desperate Roar also works at -15%, with fewer negative crits. Sources:
[Avidya Saber](https://mhgu.kiranico.com/weapon/49e70), [Tigrex LS progression](https://mhgu.kiranico.com/weapon/47a66).

- [x] Positive: a couple of critical hits show a line above the number.
- [x] Positive: an ordinary hit shows no critical line.
- [x] Negative: a couple of negative critical hits show a line below the number.
- [x] Negative: an ordinary hit shows no critical line.

Color follows physical HZV; negative crits on soft parts can stay orange-yellow.
Compare the same move/part for larger positive and smaller negative crits.

## 2. HBG impacts and multiple hits

Orcus Quarcus + positive Gunner setup: +50% affinity. Leave Palicoes behind; use
a large familiar monster and a trap if helpful. Try all ammo in one outing.

Orcus Quarcus supports all these ammo types; Slicing S Lv2 is internal ammo.
Source: [Orcus Quarcus](https://mhgu.kiranico.com/weapon/40965).

- [x] Normal S Lv2: fire a few shots into one part. Each impact has one number;
  positive crits have an upper line, while ordinary hits have none.
- [x] Pierce S Lv1: fire 2 or 3 shots lengthwise through the body. Multiple
  impacts produce separate numbers along the projectile's path.
- [x] Pellet S Lv2: fire 2 or 3 close shots. Pellets that connect produce
  separate numbers, rather than one combined total.
- [x] Slicing S Lv2: fire 1 or 2 shots. The delayed cutting hits produce numbers.
- [x] Crag S Lv1: fire 1 or 2 shots. The delayed explosion produces numbers;
  a small initial impact number can also appear.
- [x] Clust S Lv1: fire 1 or 2 shots. Explosions that connect produce separate
  numbers.
- [x] Thunder S Lv1: fire 2 or 3 shots. Elemental ammunition produces numbers
  normally, with one combined damage value per resolved impact.

Look for separate Pierce/Pellet/Cluster hits without counting every number.
Misses, angle and body-path length change impact count.

Shot/cutting HZVs differ; HBG and LS can have different colors on one part.
Thunder weakness does not select orange-yellow. Unknown physical HZV falls back
to white. Check crits with normal shots, not Crag/Cluster explosions.

## 3. Elemental contribution comparison

Thunder popups confirm display. Comparing elemental power checks its
contribution to the total, not the complete formula.

Use Orcus Quarcus + neutral Gunner Neset. Compare a few unmarked Thunder hits in
the same quest/part/state before breaks or state changes.

- [x] With no decorations, note a few unmarked Thunder S Lv1 damage values.
- [x] For the second run, add one Bolt Jwl 2 each to head, chest and waist.
  Confirm Thunder Atk +1 is active; nine points plus Neset's Skill +2 gives 11.
- [x] Comparable unmarked Thunder S Lv1 hits show a higher total with the skill.
- [x] The corresponding hit's HZV color stays the same when only elemental
  power changes.

If the values overlap or the part/state changes, record "uncertain" and stop.
Source: [Thunder Atk and decorations](https://mhgu.kiranico.com/skill/4d684).

## 4. Gray bounce and bounce prevention

Tigrine Edge LV1 + neutral Blademaster Neset, against Basarios/Gravios.
Ordinary-slash a hard unbroken part until visible recoil. Avoid spirit attacks,
which can prevent bounce.

- [x] With bounce coloring enabled, a clear bounced hit shows a gray number.
- [x] Disable bounce coloring and repeat a clear bounce. The number uses its
  normal physical-HZV color.

Add Fencer Jwl 3 to arms and legs: Fencing 8 + Skill +2 activates Mind's Eye.
Confirm the active skill before testing.

- [x] With Mind's Eye and bounce coloring enabled, an ordinary slash into the
  hard part has no bounce recoil and no gray bounce number.

If clear recoil stays white with bounce coloring on, report missing gray.
Sources: [Fencer Jwl 3](https://mhgu.kiranico.com/item/04be8), [Fencing](https://mhgu.kiranico.com/skill/4ed04).

## Optional checks

Optional checks can wait until convenient.

### Negative bowgun crit

Diablastazooka + neutral Gunner Neset: -30% affinity. Repeat Normal S Lv2 into
one part until a couple of negative crits. Disable Bitter Affinity and affinity
boosts.

- [x] Negative bowgun crits have a line below the number; ordinary hits have none.

Source: [Diablastazooka](https://mhgu.kiranico.com/weapon/4663e).

### LBG rapid fire

Avidya Eye + either Gunner setup: two Pierce S Lv1 rapid-fire bursts through a
large monster.

- [x] The burst's projectiles and their piercing impacts produce separate
  numbers, without combining the whole burst into one total.

Source: [Avidya Eye](https://mhgu.kiranico.com/weapon/47ea5).

### Multiplayer filtering

With optional sources off, watch another hunter/Palico attack, then attack
yourself.

- [x] Other hunters' and their Palicoes' weapon attacks produce no numbers.
- [x] Your own attacks still produce numbers.

## Reporting

Record passed/failed/uncertain/skipped. Failures need weapon/ammo, armor,
monster/part and observed behavior; a screenshot/clip helps with markers, colors
or missing hits. Passed checks need only a short result.
