# R17 NZCVQ gateway compatibility

R17 applies the [GE preservation audit](GE_PRESERVATION_AUDIT.md) to the plugin.
The shared restore macro changes `msr APSR_nzcvqg, r2` to
`msr APSR_nzcvq, r2`, producing nine `e12cf002` → `e128f002` replacements:
the eight entries and the return gateway. It leaves GE untouched by the
restore instruction; it neither restores nor clears those bits.

## Why this change is bounded

The audit verified all eight current MHGU 1.4.0 hook continuations against
original NSO bytes. GE is dead at those sites, including the interior melee
HZV tap. Imported helper functions may change GE under the calling convention.
NZCVQ, all r0–r12, d0–d31, FPSCR, LR/SP and native stack arguments retain their
existing preservation contract. The frame stays 328 bytes, and capture keeps
the same eight signatures, hook offsets, one 4KiB arena and TLS slot.

The unmodified Ryujinx-Nextendo reference at
`05577ab543f4322cb7534c1f5caca0e0079791bf` accepts MSR mask 8 (NZCVQ) and throws
`APSR_g` for mask 4 (GE). R17 removes that GE-write path from this plugin. It
does not modify Forge or an emulator. Compatibility still depends on the
loader and the other APIs used by the plugin. The user confirmed working
gameplay on both tested installations on 2026-10-06.

This is not permission to drop GE at arbitrary new hooks. Re-audit any added
tap, especially one between GE-setting arithmetic and `SEL` or a GE-reading
`MRS`. The original audit's scratch results remain preserved.

## Validation

The rebuilt production NRO is compared against the exact audited r16 binary.
The verifier requires that only the nine restore instruction words differ in
the executable bytes after the NRO header. It also checks unchanged initialized
data, imports, BSS and segment sizes, and ties the instruction mask to the
recorded translator source. Font and effect assets remain unchanged.

The gateway fixture checks the reduced contract explicitly. Its original
9,031 checks retain every state comparison except GE restoration. An additional
256 checks exercise all 16 GE patterns across all eight entries with both
GE-preserving helpers and helpers that deliberately change GE. Each checks all
other return state and verifies the helper's GE value remains in place. Total:
**9,287 checks across 2,025 calls**, including native damage/hitzone calculations.

The compiled production capture fixture adds nine instruction checks to the
338 r16 checks, for **347**. Host checks, the asset audit and real-ImGui
comparisons retain prior capture, source filtering, settings and appearance
coverage. These fixtures do not establish Ryujinx or Eden gameplay acceptance.

```bash
python3 scripts/test_damage_numbers.py
python3 scripts/build_damage_numbers_gateway.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_gateway_probe.py
python3 scripts/build_damage_numbers_capture.py
python3 scripts/verify_damage_numbers_ge_compat.py
python3 scripts/audit_damage_numbers_open_assets.py
python3 scripts/test_damage_numbers_visual.py
python3 scripts/verify_damage_numbers_compression.py
tools/mwc-analysis-venv/bin/python scripts/damage_numbers_capture_probe.py
python3 scripts/package_damage_numbers.py
```

Build and fixture records are under `build/damage-numbers/open-assets/`.
The verified r16 baseline, gateway and source artifacts are preserved at
`backups/damage-numbers/r16-before-r17-ge-20261006/`. Installed Eden/Ryujinx
files, emulator configuration, caches and saves are not changed by these recipes.

## Package and brief runtime check

`build/damage-numbers/releases/r17/damage-numbers-r17.zip` contains only the NRO
inside `damage-number/romfs/nativeNX/plugins/`. README, notices, checksums and
provenance remain beside the ZIP. Keep the notices with redistributions.
The status is **ready to distribute**, as directed by the user on 2026-10-06.
The user confirmed r17 working on both Eden and Ryujinx-based emulators on
2026-10-06. Installed NROs match r17; latest logs were reviewed and retained.
See [runtime history](RUNTIME.md) for log details and coverage limits.
Publication has not been performed.

The compatibility confirmation is complete for the two tested installations.
The user report does not specify a repeat of every earlier combat test or
verification of every Ryujinx fork. The separate Forge menu/popups
corruption remains documented in the [renderer handoff](../../docs/FORGE_IMGUI_DAMAGE_CORRUPTION_HANDOFF.md).
