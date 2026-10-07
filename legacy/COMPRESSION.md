# R16 lossless artwork compression

R16 reduces the plugin from **262,144 bytes (256KiB) to 155,648 bytes
(152KiB)**, a 40.6% reduction. Its font, accepted RGBA artwork, real-ImGui
textures, draw vertices/indices/commands and composited preview match r15
byte for byte. The 20px default, animation, settings and capture hooks are
unchanged. Installation is separate; the installed r15 was read only.

## Cost and memory

The embedded artwork falls from 171,700 bytes of pixel RLE to 64,170 bytes of
DN-LZ1. Its decoded texture remains 512×256 RGBA32, or 524,288 bytes. Decoding
writes directly into Forge's existing texture allocation once per ImGui
context. No second image allocation or persistent scratch buffer is needed.
The compiled fixture verifies that hit capture and rendering never call the
decoder, repeated initialization reuses the texture, and a new context
initializes it once again.

On this Linux host, optimized native decoder medians across nine batches of
100 full-atlas decodes were **143.431 microseconds** for DN-LZ1 and **139.387
microseconds** for the former RLE. These are host initialization timings,
not Eden or Switch measurements. No live FPS result is claimed. Steady-state
rendering uses exactly the same texture and draw commands, so compression
adds no frame-time work. NRO BSS remains 1,093,632 bytes and the import set is
unchanged. The smaller read-only segment also reduces loaded payload size.

## API choice and format

The inspected maintained Forge headers, implementations and exports offer
font/texture registration, but no general-purpose compression decoder. Saved
research identifies game archive decompression methods with unverified
interfaces; the plugin does not call those private methods. It keeps Forge's
`ImTextureData_Create`, `ImTextureData_GetPixels` and `igRegisterUserTexture`
flow. The decoder uses already-imported libc helpers for fills and word copies.
No additional library, hook or CodeMemory allocation is introduced.

DN-LZ1 is an owned, allocation-free byte-stream codec:

- Control 0–127: copy the following control+1 literal bytes.
- Control 128–254: copy control−125 bytes from a backward reference.
- Control 255: read a little-endian 16-bit extension; length is 130+extension.
- Each copy's distance is a following little-endian 16-bit value. Overlapping
  copies are supported; zero distances and references before output are rejected.

The decoder bounds every input read and output extent, and requires the exact
decoded byte count. A failed stream destroys the unregistered texture and logs
the existing crisp-digit fallback through Forge. Neither pixels nor alpha are
quantized. Transparent RGB remains canonical zero, exactly as in r15.

## Verification and reproduction

Passed: deterministic round trips, ASan/UBSan, 20,000 malformed streams,
truncation and guard checks, complete decoded-atlas hashing, 919 host
core/presentation checks, mailbox stress and 23 transactional hook scenarios.
The compiled ARM32 fixture retains its 324 checks and adds 14 codec/context/
fallback checks, for **338**. Pinned real ImGui produces the exact previous
font/effect textures and draw output. The open-asset audit verifies the actual
embedded compressed payload and independently decodes it.

```bash
python3 scripts/test_damage_numbers_sprite_codec.py
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_capture.py
python3 scripts/test_damage_numbers_visual.py
python3 scripts/verify_damage_numbers_compression.py
python3 scripts/audit_damage_numbers_open_assets.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
python3 scripts/package_damage_numbers.py
```

The equivalence verifier uses the hashed r15 snapshot at
`backups/damage-numbers/r15-before-compression-20261005/`. Current generated
reports live in `build/damage-numbers/open-assets/`; provenance and performance
results accompany the clean r16 ZIP in its adjacent release manifest.

Prior gameplay confirmations remain recorded in [TEST_RESULTS.md](TEST_RESULTS.md).
No new gameplay run is claimed. If installing this trial, one normal boot and a
few hits in Wilds style are enough for a brief smoke check; combat coverage needs
no full repeat. The known Forge menu/damage-popup corruption is unchanged:
[handoff](docs/FORGE_IMGUI_DAMAGE_CORRUPTION_HANDOFF.md).
