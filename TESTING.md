# Damage Numbers r48: testing

r48 removes Show other damage, adds Spread, and puts the settings on a Mod Options page with a
help topic per setting and a live preview. The page needs Purple-lib r17 or later installed
beside it; with Purple-lib r19 the preview also hides while the page's help is open. Without
Purple-lib, the Plugin UI panel and the INI work as in r42 and no Purple-lib line appears.

## Steps

1. Keep an r42 `damage_numbers.ini` (with its `other =` line) and start the game. The mod loads,
   the Plugin UI panel shows Spread at 0.8 and no Show other damage, and the other settings are
   the ones the file had.
2. In the village or hub, in English, open **Options > Mod Options** and choose **Damage Numbers**
   in the list: eight rows, Show damage numbers through Restore defaults. A frame labelled
   "Damage number preview" sits on the left of the screen, clear of the Options window, with
   three white dots in it.
3. With Preview mode on Single, each dot gets one number, then the frame is empty for about half
   a second, then the loop repeats. Left is gold with a pink star above, middle white with a blue
   star below, right grey with no star (Wilds style). Switch to Burst (five numbers per dot in
   quick succession) and Multi (five at once, spread apart).
4. Change Style to Rise and Classic: the preview's numbers switch style at once. Change Number
   size, Duration and Spread: new numbers follow them. Spread 0% stacks Multi's numbers on each
   dot; 200% spreads them widely.
5. Turn Random hit types on: hits of every colour and critical appear at all three dots. Turn
   Show damage numbers off: the preview still shows numbers.
6. Press the help button ((−) "Mod Help", or (+) if you swapped them): a help titled "Damage
   Numbers" with seven topics, each page fitting its box. With Purple-lib r19 the preview is gone
   while the help is open and returns when you close it; with r17 or r18 it stays.
7. In a quest, open the page, walk and turn the camera: the dots and their numbers stay at their
   places in the world, loop after loop, and move on screen as you move. When a dot leaves the
   screen, all three come back inside the frame. In the village the dots stay at their places in
   the frame. Close the page: the preview disappears.
8. Close the page after changing Number size: the `settings saved` line appears. Close it after
   changing only Preview mode: no save line.
9. Restart the game: the saved settings are kept, Preview mode is Single and Random hit types Off.
10. With the game in French, German, Spanish, Italian or Chinese, repeat steps 2 and 6: rows,
    labels, help-bar lines and help pages are in that language, and Duration shows "0,9 s" (or
    "0.9秒" in Chinese).
11. Launch once each on Eden, Ryubing and MHGURecomp.

## Log lines

These are info lines; MHGURecomp's `recomplog.txt` shows them.

| Line | Meaning |
| --- | --- |
| `[damage-numbers] r48 loaded: 11 Forge hooks` | Started with every hook installed |
| `[damage-numbers] settings and help added to Purple-lib` | Purple-lib took the page and its seven topics |
| `[damage-numbers] settings saved` | The INI was written (Save settings, closing the page after a saved row changed, or Restore defaults on the page) |
| `[damage-numbers] Save failed: ...` | The INI could not be written; the settings stay for this session |
| `Purple-lib API version N is older than the supported 3` | An older Purple-lib called the mod: install r17 or later |
| `Purple-lib refused the registration` | Send the log |
