# Components used by the damage-number capture candidate

Build dependencies: maintained Forge, pinned libnx/cimgui headers and offline
devkitARM. No game executable is delivered. The r12 personal candidate includes local
Wilds font outlines and crit sprite crops, described below.

`src/start.S` / `src/plugin.ld`: project-owned Long Sword scaffold.
`src/transaction.h`: modern-controls transaction plus six exact PC-independent
words. Private NRO components: `mods/modern-weapon-controls/src/native_platform.cpp` / `src/svc.S`; no
installed-plugin import or shared arena. Source hashes: capture manifest.

Private SVC assembly adapts pinned libnx ARM32 wrappers under the MIT notice
below. Unicorn 2.1.4 is fixture-only, never linked into the NRO.

`tests/romfs_flag.cpp` models Eden `d16735f5b6` RomFS/concatenation from
`fsmitm_romfsbuild.cpp` / `vfs/vfs_concat.cpp`. It retains yuzu copyright and
GPL-2.0-or-later; it is never linked into the NRO. Source URLs/fingerprints:
RomFS probe report.

Copyright (c) 2019 Hydrosphere Developers

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## Historical Roboto digit subsets (r10/r11)

Copyright 2011 The Roboto Project Authors
(https://github.com/googlefonts/roboto-classic). Licensed under SIL OFL 1.1.
[Full license](fonts/OFL.txt). Source provenance for those builds remains in Git
history. R12 does not embed or package the Roboto digit subset/OFL notice.

## Local Wilds resources (r12)

Frutiger Neue LT W1G Medium digit outlines and crit texture crops are derived
from the user's local Monster Hunter Wilds installation. They are game assets,
not OFL fonts. Generated payloads remain ignored and are used in this personal
local build. No redistribution license is asserted. Source hashes, resource
paths, conversion tolerances and tool revisions: [WILDS_ASSETS.json](WILDS_ASSETS.json)
and [font manifest](fonts/source.json).

FontTools, Pillow/NumPy and GDeflate are preparation/fixture dependencies, not
linked into the NRO. Their upstream notices remain in the local tool checkouts.

## Selective PAK reader

`scripts/extract_wilds_damage_assets.py` adapts the index-cipher algorithm from
[Eigeen/ree-pak-rs](https://github.com/eigeen/ree-pak-rs), pinned revision
`59883125d959c7d7238a69d10ec547078b65042e`. Its MIT notice follows.

MIT License

Copyright (c) 2024 Eigeen

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
