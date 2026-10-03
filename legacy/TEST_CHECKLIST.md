# Damage numbers: in-game checklist

Use LS for melee and HBG/LBG for ranged. These checks cover critical markers,
bowgun impacts, elemental damage and bounce coloring. Placement, animation,
size, white melee damage and Palico melee/boomerang/barrel numbers have already
passed the user's trials.

Stop each check after a couple of clear examples. You can return from the quest
once you have seen enough. For crits, about 10 to 20 clean hits should usually
be enough; an unlucky run does not establish a failure.

Record outcomes in [TEST_RESULTS.md](TEST_RESULTS.md). Check a box only after
observing its expected result. Leave skipped or uncertain checks unchecked.

## Equipment preparation

Use full Neset, no talisman and empty weapon slots. Upgrade armor defense as
needed for comfortable testing.

| Slot | LS armor | Bowgun armor | Positive setup decoration |
| --- | --- | --- | --- |
| Head | Neset Helm | Neset Hat | Expert Jwl 2 |
| Chest | Neset Mail | Neset Vest | Expert Jwl 2 |
| Arms | Neset Gloves | Neset Brachia | Expert Jwl 3 |
| Waist | Neset Coat | Neset Elytra | Expert Jwl 2 |
| Legs | Neset Boots | Neset Feet | Expert Jwl 3 |

Both sets have 2/2/3/2/3 slots. The decorations provide 19 Expert points;
Neset's Skill +2 raises that to 21, activating Critical Eye +3 (+30% affinity).
Sources: [Blademaster Neset](https://mhgu.kiranico.com/armor/59bb6),
[Gunner Neset](https://mhgu.kiranico.com/armor/54be8),
[Expert](https://mhgu.kiranico.com/skill/4ee04),
[Skill +2](https://mhgu.kiranico.com/skill/4ea5e).

- [ ] Prepare the positive setup using the decorations above.
- [ ] Prepare a neutral setup with all decorations and the talisman removed.

Check the status screen's affinity before each quest. If it differs from the
values below, record the actual value; gameplay mods can alter equipment.

## 1. LS critical markers

Leave Palicoes behind for these checks so your hits are easy to identify.
Choose a familiar High Rank monster. Use ordinary slashes into the same part
and keep sharpness and spirit gauge color consistent when comparing values.

| Check | Weapon | Armor setup | Expected affinity |
| --- | --- | --- | ---: |
| Positive crit | Avidya Saber | Positive Blademaster | +75% |
| Negative crit | Tigrine Edge LV1 | Neutral Blademaster | -20% |

If you already have Desperate Roar, it also works for the negative check at
-15% affinity, although crits may take longer to appear. Sources:
[Avidya Saber](https://mhgu.kiranico.com/weapon/49e70),
[Tigrex LS progression](https://mhgu.kiranico.com/weapon/47a66).

- [ ] Positive: a couple of critical hits show a line above the number.
- [ ] Positive: an ordinary hit shows no critical line.
- [ ] Negative: a couple of negative critical hits show a line below the number.
- [ ] Negative: an ordinary hit shows no critical line.

Color still follows physical HZV. A negative crit on a soft part can stay
orange-yellow. Compare repeated instances of the same move and part if checking
whether positive crits are larger and negative crits smaller.

## 2. HBG impacts and multiple hits

Equip Orcus Quarcus with the positive Gunner setup (+50% affinity). Leave
Palicoes behind. Pick a large monster you are comfortable fighting and bring
a trap if it helps with aiming. Try the ammo below in one outing.

Orcus Quarcus supports all these ammo types; Slicing S Lv2 is internal ammo.
Source: [Orcus Quarcus](https://mhgu.kiranico.com/weapon/40965).

- [ ] Normal S Lv2: fire a few shots into one part. Each impact has one number;
  positive crits have an upper line, while ordinary hits have none.
- [ ] Pierce S Lv1: fire 2 or 3 shots lengthwise through the body. Multiple
  impacts produce separate numbers along the projectile's path.
- [ ] Pellet S Lv2: fire 2 or 3 close shots. Pellets that connect produce
  separate numbers, rather than one combined total.
- [ ] Slicing S Lv2: fire 1 or 2 shots. The delayed cutting hits produce numbers.
- [ ] Crag S Lv1: fire 1 or 2 shots. The delayed explosion produces numbers;
  a small initial impact number can also appear.
- [ ] Clust S Lv1: fire 1 or 2 shots. Explosions that connect produce separate
  numbers.
- [ ] Thunder S Lv1: fire 2 or 3 shots. Elemental ammunition produces numbers
  normally, with one combined damage value per resolved impact.

For Pierce, Pellet and Cluster, look for separate hits without trying to count
every digit. Misses, angle and the length of the path through the monster affect
the number of impacts.

Shot and cutting HZVs differ, so HBG and LS can have different colors on the
same part. Thunder weakness itself must not turn a number orange-yellow.
Explosions without an applicable physical HZV should use the unknown-HZV white
fallback. Crag/Cluster explosions are not the normal-shot crit test.

## 3. Elemental contribution comparison

Seeing Thunder numbers confirms that the ammo path displays damage. This
comparison checks whether changing its elemental power changes the displayed
total; it does not verify the complete damage formula.

Use Orcus Quarcus with neutral Gunner Neset. Keep the same quest, monster part
and ammo for both runs, and compare unmarked hits before the part breaks or the
monster changes state. A few examples from each setup are enough.

- [ ] With no decorations, note a few unmarked Thunder S Lv1 damage values.
- [ ] For the second run, add one Bolt Jwl 2 each to head, chest and waist.
  Confirm Thunder Atk +1 is active; nine points plus Neset's Skill +2 gives 11.
- [ ] Comparable unmarked Thunder S Lv1 hits show a higher total with the skill.
- [ ] The corresponding hit's HZV color stays the same when only elemental
  power changes.

If the values overlap or the part/state changes, record "uncertain" and stop.
Source: [Thunder Atk and decorations](https://mhgu.kiranico.com/skill/4d684).

## 4. Gray bounce and bounce prevention

Use Tigrine Edge LV1 with neutral Blademaster Neset against Basarios or Gravios.
Use an ordinary slash against a hard, unbroken part until the hunter visibly
recoils. Spirit attacks can prevent bounce and are unsuitable for this comparison.

- [ ] With bounce coloring enabled, a clear bounced hit shows a gray number.
- [ ] Disable bounce coloring and repeat a clear bounce. The number uses its
  normal physical-HZV color.

For the prevention check, add one Fencer Jwl 3 to the arms and one to the legs.
Eight Fencing points plus Neset's Skill +2 activates Mind's Eye. Confirm it in
the active skill list before the next outing.

- [ ] With Mind's Eye and bounce coloring enabled, an ordinary slash into the
  hard part has no bounce recoil and no gray bounce number.

The mod conservatively matches native bounce feedback. If a clear recoil stays
white with coloring enabled, record that as a missing-gray case for review.
Sources: [Fencer Jwl 3](https://mhgu.kiranico.com/item/04be8),
[Fencing](https://mhgu.kiranico.com/skill/4ed04).

## Optional checks

These can wait until convenient. They cover additional paths beyond the main
LS/HBG checks.

### Negative bowgun crit

Equip Diablastazooka HBG with neutral Gunner Neset (-30% affinity). Fire Normal
S Lv2 into the same part until you see a couple of negative crits. Keep Bitter
Affinity and affinity-boosting skills off for this check.

- [ ] Negative bowgun crits have a line below the number; ordinary hits have none.

Source: [Diablastazooka](https://mhgu.kiranico.com/weapon/4663e).

### LBG rapid fire

Equip Avidya Eye with either Gunner setup. Fire two Pierce S Lv1 rapid-fire
bursts through a large monster's body.

- [ ] The burst's projectiles and their piercing impacts produce separate
  numbers, without combining the whole burst into one total.

Source: [Avidya Eye](https://mhgu.kiranico.com/weapon/47ea5).

### Multiplayer filtering

When an online session is available, stand back while another hunter and their
Palico attack, then make a few attacks yourself. Keep optional source switches
off to isolate weapon ownership.

- [ ] Other hunters' and their Palicoes' weapon attacks produce no numbers.
- [ ] Your own attacks still produce numbers.

## Reporting

Use passed, failed, uncertain or skipped in the results sheet. For a failure,
record the weapon/ammo, armor setup, monster/part and what appeared. A screenshot
or short clip helps if the problem concerns a marker, color or missing impacts.
Routine successful checks only need a short result.
