# R10 cosmetic confirmation

Run this after installing r10. Previously passed combat checks need no full
repeat: [r9 results](TEST_RESULTS.md), [runtime evidence](RUNTIME.md).
Stop after a couple of clear examples. Crits can take several hits; bad luck
alone is not a failure.

Start with Wilds, Number size 1.0 and Duration 0.9. Leave Palicoes behind if their
numbers make the comparison harder to see.

## Crit appearance: three reference cases

Reuse the tested positive setup: Orcus Quarcus with the positive Gunner Neset
setup, or Avidya Saber with positive Blademaster Neset. The positive setup used
Expert Jwl 2 in head/chest/waist and Expert Jwl 3 in arms/legs, with no talisman.
For negative crits, use Diablastazooka with neutral Gunner Neset and no affinity
boosts or Bitter Affinity.

- [ ] On a part giving gold numbers, see a positive crit: pink-red elongated
  four-point star above gold digits, with a small glow around the star.
- [ ] On a part giving gold numbers, see a negative crit: blue star below gold
  digits. Ordinary hits have no star.
- [ ] Find a part giving white numbers and see a positive crit: warm white
  digits with the same pink-red star above. Bowgun Normal S Lv2 avoids melee
  bounce turning this comparison gray.

Look for crisp digits with a small dark shadow, matching the accepted visible
height and motion. Colored glow should be concentrated on the crit marker.
These are visual approximations of the Wilds references.

## Burst spacing and saved style

- [ ] Fire two Avidya Eye Pierce S Lv1 rapid-fire bursts through a large monster.
  Impacts stay separate and nearby; older numbers do not jump sideways when new
  hits arrive. Some overlap in a dense burst is acceptable. The burst should not
  grow into a tall stack.
- [ ] Switch Style to Classic and land a couple of hits: old font/colors,
  outlined digits and straight crit lines return. Switch to Wilds and check
  the new font/colors/stars again.
- [ ] Select your preferred style, press Save settings, restart once, and check
  that style is restored.

Bounce coloring, recognized monster impacts and bombs/objects are always active
while damage numbers are enabled. Their old controls are absent. Existing
version-1 settings are ignored; save explicitly to create version 2.

## Completed combat evidence

- [x] Hunter/Palico melee, boomerangs and support barrels display.
- [x] White/weakspot HZV coloring and positive/negative crit detection passed.
- [x] LS and HBG Normal/Pierce/Pellet/Slicing/Crag/Clust/Thunder impacts passed.
- [x] Elemental totals, gray bounce, Mind's Eye and LBG rapid fire passed.
- [x] Other hunters' and Palicoes' weapon attacks stay hidden in multiplayer.
- [x] Settings persistence, monster-to-monster damage, Prowler and bombs passed.

These checked items record r9 and earlier user reports. The unchecked r10 items
above remain pending. Record only new observations in the r10 section of
[TEST_RESULTS.md](TEST_RESULTS.md).
