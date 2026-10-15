# Damage Numbers in Rust

`r3` uses Forge 1.5.0 and forge-rs 1.5.0 for the Rust port of Damage Numbers r25. It retains the eight capture sites, ownership and damage rules, duplicate suppression, projection and placement, and Wilds, Rise and Classic presentation. Defaults are enabled, Wilds, size 1.0 and lifetime 0.9 seconds. The unchanged r3 payload is READY_TO_DISTRIBUTE after user acceptance on 2026-10-15. Coverage and log limits are recorded in [runtime evidence](docs/RUNTIME_RESULTS.json).

[Player guide](DISTRIBUTION_README.md), [build instructions](DEVELOPMENT.md), [code walkthrough](docs/CODE.md), [validation scope](tests/README.md) and [r3 results](docs/CANDIDATE_RESULTS.json) and [dependency provenance](THIRD_PARTY.md).

The standalone workflow is `./build.sh doctor`, `prepare`, `build`, `test`, `verify`, `package`, and `export --output DEST`. `test --host` omits ARM checks. Preparation, builds and packaging do not install or activate the plugin. The runtime package contains `damage_numbers_rs.nro`, `OpenSans.ttf`, and two external RGBA artwork files. Ordinary builds copy prepared assets; the runtime contains no artwork payloads or compression decoder. Existing C++ source and r25 packages remain preserved.

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches Forge 1.5.0's native ARM32 declaration: 24 bytes, with context and its update cell. Forge shares trampoline storage across hooks. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the recorded eight-site GE audit.

The load callback leaves the loader requirement record untouched, so Forge version checks do not gate startup. The build targets the 1.5.0 native ABI; other loader layouts can be incompatible. Initialization takes no arguments, matching the loader callback. Game-code signatures and instruction checks remain in place. The preserved [r1 results](docs/R1_CANDIDATE_RESULTS.json) describe the older candidate.

The prepared font retains Open Sans Semibold naming. Its glyph outlines, hinting, metrics and weight match r2; only naming metadata/checksums changed. Artwork pixels and atlas geometry are unchanged. The selected styles appear as Wilds, Rise, Classic; stored style IDs remain stable. Missing artwork omits its effects while readable damage numbers continue. Failure messages are deduplicated across context replacements.

The canonical local release is `build/damage-numbers-rs/releases/r3/damage-numbers-rs-r3.zip`. Use the coordinating workspace’s `python3 scripts/mod_distribution.py package damage-numbers-rs` and full `check` for approved delivery. The mod-local packaging command retains the development-candidate workflow.
