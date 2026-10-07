# Damage Numbers r19 readability

The user approved the 0.5 px renderer comparison on 2026-10-07 and requested a
build ready to distribute. Wilds digits now have a near-black `#17130F` edge at
90% of number opacity. It measures 0.5 px at the default 20 px size and scales
with number size, resolution and the initial pop. The gold halo is removed;
the 22% centered shadow, font, fill colors, crit stars and animation stay the same.
Classic is unchanged.

ImGui truncates text origins. The renderer draws eight edge copies at the same
origin, then translates their emitted vertices by the subpixel offsets. This
keeps opposite edges symmetric without changing the fill or star anchor. Each
label adds eight text passes; they use the existing atlas and do not allocate
another texture, hook, CodeMemory arena or persistent buffer. The host fixture
checks a worst-case 128 ten-digit labels with crit stars: 51,712 vertices, within
16-bit indexing. Actual shared Forge frame load and GPU timing were not measured.

Fresh validation covers real ImGui geometry, 720p/1080p scaling, pop/fade,
font and sprite byte equality, host damage/placement, compiled ARM32 capture,
ABI/imports, SDK guards, gateway state/GE and package integrity. The reviewed
host draw/preview hashes are in `research/readability-r19.json`; the r15 asset
baseline remains intact. The font and sprite texture hashes are unchanged.
No new gameplay run was performed. Earlier manual reports retain their original
versions. Preview approval establishes the chosen appearance only.

The NRO is 151,552 bytes (148 KiB); BSS remains 1,093,632 bytes. The nine NZCVQ-only
gateway restores and eight hook sites are unchanged. The known Forge menu/render
issue remains unresolved. This release did not install, activate, publish or push.

Local delivery: `build/damage-numbers/releases/r19/damage-numbers-r19.zip`.
The previous r18 release and build/preview evidence were copied and hashed under
`backups/damage-numbers/readability-r19-20261007/`; its `SHA256.json` preserves
all files before this change. Release and candidate receipts record exact
source and payload hashes.
