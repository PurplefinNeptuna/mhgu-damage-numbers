# Damage Numbers r20 crit size

The user selected the real-renderer 1.5× crit-size preview on 2026-10-07 and
requested direct distribution readiness. Both Wilds crit stars now use that
uniform scale. The original licensed font and procedural artwork bytes remain
unchanged, as do the 0.5 px digit outline, colors, shadow and animation.
Classic is unchanged.

The solid star-core height and width used for placement also grow by 1.5×.
The distance from each core edge to the digits remains two layout units.
Bloom may overlap, as before. This preserves the approved appearance while
allowing placement to account for the larger markers in crowded hits.
The source uses one scale constant for the renderer and solid bounds.

The real ImGui fixture checks both crit polarities at 720p and 1080p, uniform
art aspect ratio, 1.5× full sprite height, unchanged core gap, and enlarged
solid placement bounds. Existing subpixel outline, fade and maximum-vertex
checks remain. Compiled ARM32 checks independently verify sprite dimensions
and positive/negative core spacing. Host damage/placement, gateway/GE, SDK,
ABI/import, open-asset and package checks passed. The pinned draw/preview hashes
are in `research/readability-r20.json`; r19's baseline remains historical.
Font and sprite textures are byte-identical to r19. There are no extra draws,
textures, hooks or persistent buffers for this change.

The NRO remains 151,552 bytes (148 KiB), with BSS at 1,093,632 bytes. The nine
NZCVQ-only gateways and eight hook sites are unchanged. No new gameplay run
or GPU timing measurement was performed. Preview approval establishes the
chosen visual design only. Forge's known menu/render issue remains unresolved.

Local delivery: `build/damage-numbers/releases/r20/damage-numbers-r20.zip`.
The r19 release and previous build/preview files were copied and hashed under
`backups/damage-numbers/crit-size-r20-20261007/`, with a `SHA256.json` receipt.
This release did not install, activate, publish or push.
