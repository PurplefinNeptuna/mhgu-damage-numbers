# Damage numbers r18

Local delivery: `build/damage-numbers/releases/r18/damage-numbers-r18.zip`.
The named `damage-number` package contains one plugin NRO. Keep the adjacent README, notices, release manifest and checksums with it.

R18 retains appearance/behavior settings and the damage overlay. Its panel contains settings and save feedback only. Forge logs one startup summary and actionable failures, with process-lifetime deduplication. Default builds omit hit and camera dumps; developer diagnostics require an explicit build option and add no diagnostic UI.

The audited eight hooks and nine NZCVQ-only gateway restores are unchanged. Fresh host, original/compiled ARM32, ABI/import, GE and rendering fixtures passed. No new gameplay run was performed. R17 was confirmed on Eden and Ryujinx; that confirmation does not extend to every feature of this new payload.

Forge remains a separate dependency. Its known menu/background rendering issue remains unresolved and unchanged; see [renderer handoff](docs/FORGE_IMGUI_DAMAGE_CORRUPTION_HANDOFF.md). Prepare/install instructions are in [the delivery README](DISTRIBUTION_README.md). This work did not install or activate anything.

Previous immutable r17 deliveries are preserved under `backups/damage-numbers/standalone-settings-cleanup-20261007/releases/r17/`. Source snapshots, checks and original receipts remain in Git and [the migration record](docs/STANDALONE_MIGRATION.json).
