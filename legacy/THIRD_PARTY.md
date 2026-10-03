# Components used by the damage-number capture candidate

The candidate uses the maintained local Forge headers, pinned libnx/cimgui
headers and existing offline devkitARM container. Those dependencies and their
licenses remain unchanged. No game executable or extracted asset is delivered.

`src/start.S` and `src/plugin.ld` reuse the project-owned Long Sword scaffold.
`src/transaction.h` adapts the project-owned modern-controls transaction with
six additional exact, PC-independent instruction words. The native build links
`mods/modern-weapon-controls/src/native_platform.cpp` and `src/svc.S` as private
components of this NRO; it does not import the installed controls plugin or
share that plugin's arena. Source hashes are recorded in the capture manifest.

The private SVC assembly adapts selected pinned libnx ARM32 wrappers. Its MIT
notice is reproduced below. Unicorn 2.1.4 is used only by workspace fixtures and
is not linked into the candidate.

`tests/romfs_flag.cpp` is a test-only model of the RomFS layout and concatenation
logic in Eden revision `d16735f5b6`, derived from `fsmitm_romfsbuild.cpp` and
`vfs/vfs_concat.cpp`. It retains the yuzu copyright notice and
GPL-2.0-or-later license identifier. It is not compiled or linked into the NRO.
Source URLs and fingerprints are in the generated RomFS probe report.

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
