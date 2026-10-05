# Open digit font comparisons

The user requested fuller, readable curves and left-facing ends on digits 2
and 5. Hind Semibold was initially selected, then rejected after inspecting
its curves. The next comparison focuses on alternatives to Open Sans.
The user selected Open Sans Semibold for the r14 trial. The existing r13
releases remain intact.

All samples use weight 600, upright faces, and default width/optical-size axes.
Visible height is normalized from the zero glyph, with enlarged 2/5, all ten
digits, and small white/gold labels. These Pillow previews compare outlines;
they do not prove readability during camera movement in Eden.

| Group | Fonts | Preview under `work/damage-numbers/font-alternatives/` |
| --- | --- | --- |
| Initial | Roboto, Arimo, Source Sans 3, Inter, Lato, Hind | `comparison.png` |
| Hind alternatives | Roboto, Hind, Hind Madurai, Mukta, Open Sans, Source Sans 3 | `hind-alternatives.png` |
| Open Sans alternatives | Open Sans, Noto Sans, Fira Sans, Public Sans, Libre Franklin, Lato | `open-sans-alternatives.png` |
| Requested three-font comparison | Current Open Sans, Titillium Web, Hind | `open-sans-titillium-hind.png` |

Visual observations: Hind Madurai resembles Hind closely. Noto Sans is closest
to Open Sans in these digit samples. Fira Sans is narrower; Public Sans and
Libre Franklin have more distinct curve shapes. The user chose Open Sans itself.

Candidate sources and OFL notices were fetched from the pinned Google Fonts
revision `9710da1eacb3be272583c3224dcb70f9da6eadbb`. The
[recipe](fonts/alternatives.json) records URLs and full source/license hashes.
Fonts and generated comparisons stay in ignored workspace research files.
No proprietary visual input is used in these comparisons.

Primary metadata:
[Open Sans](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/opensans/METADATA.pb),
[Noto Sans](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/notosans/METADATA.pb),
[Fira Sans](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/firasans/METADATA.pb),
[Public Sans](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/publicsans/METADATA.pb),
[Libre Franklin](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/librefranklin/METADATA.pb).

Reproduce after placing the source files and OFL notices at the recorded paths:

```bash
python3 scripts/compare_damage_numbers_fonts.py --group initial
python3 scripts/compare_damage_numbers_fonts.py --group hind
python3 scripts/compare_damage_numbers_fonts.py --group open-sans
python3 scripts/compare_damage_numbers_fonts.py --group titillium
```

The three-font preview uses the current r14 Open Sans source and axes, with
Titillium Web Semibold and Hind Semibold at weight 600. It shows the same
enlarged 2/5, digit strip and small labels as earlier comparisons. The r14
candidate, release ZIP and installed NRO are fingerprinted before and after;
no build, deployment or settings change is part of this preview request.
[Titillium Web metadata](https://github.com/google/fonts/blob/9710da1eacb3be272583c3224dcb70f9da6eadbb/ofl/titilliumweb/METADATA.pb).

## Prepared crit bloom adjustment

The open crit sprite retains its 88x48 texture bounds and solid star polygon.
An additional broad colored halo and horizontal bloom streak improve its
visibility at gameplay size. They are baked into the same sprite without
extra runtime draw calls. Both red and blue markers retain their pale cores.
The game-asset generator was removed when the private edition was retired;
this procedural marker belongs to the sole supported open edition.

The first real-ImGui preview passed with the updated procedural art and
unchanged Roboto font. R14 applies the selected Open Sans font and keeps the
star's render scale independent of font height normalization so its accepted
visible dimensions do not grow. Final evidence is in [RUNTIME.md](RUNTIME.md).
