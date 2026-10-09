# Components used by damage numbers

Build dependencies: maintained Forge, pinned libnx/cimgui headers and offline
devkitARM. The supported build embeds OFL Open Sans digits and independently
authored procedural crit stars and Rise beams. No game executable or extracted visual asset
is delivered.

`src/start.S` / `src/plugin.ld`: project-owned Long Sword scaffold.
`src/transaction.h`: modern-controls transaction plus six exact PC-independent
words. Private NRO components: `mods/modern-weapon-controls/src/native_platform.cpp` / `src/svc.S`; no
installed-plugin import or shared arena. Source hashes: capture manifest.

Private SVC assembly adapts pinned libnx ARM32 wrappers under the MIT notice
below. Unicorn 2.1.4 is fixture-only, never linked into the NRO.

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
[Full license](fonts/roboto-OFL.txt). Source provenance for those builds remains in Git
history. R12 does not embed or package the Roboto digit subset/OFL notice.

FontTools and Pillow/NumPy are preparation/fixture dependencies, not linked into
the NRO. Their upstream notices remain in the local tool checkouts. Retired
PAK/font-conversion notices and provenance remain in Git history.

## Supported open assets

The supported profile embeds an Open Sans Semibold 600 digit subset from pinned Google
Fonts; the NRO embeds the full SIL OFL notice. The release also supplies full
font/art/dependency notices in `NOTICES.txt` beside the runtime-only ZIP. It is renamed Damage Digits Open;
[source/axes/license hashes](fonts/open-source.json) document the modified
subset. Independently authored procedural crit artwork uses the separate
[notice](OPEN_ART_LICENSE.txt). Its binary and ZIP include no extracted
Wilds font/texture payload and do not include the Wilds extraction tools.
Forge remains an external dependency. [Font registration and asset preparation](fonts/README.md)
describe the build flow; [COMPRESSION.md](COMPRESSION.md) specifies the codec.

Google Fonts revision: `9710da1eacb3be272583c3224dcb70f9da6eadbb`.
Open Sans source SHA-256:
`36643644f318a812aab2d2ed3bb98f8cf0872527f835fe9398d95fe6b9adb878`.
The subset retains digits, space and question mark. The asset audit blocks
proprietary visual input reads and compares compiled font, license and
procedural RGBA payloads with the generated open assets.
