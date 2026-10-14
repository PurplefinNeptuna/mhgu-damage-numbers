# Damage Numbers in Rust

`r2` uses Forge 1.5.0 and forge-rs 1.5.0 for the Rust port of Damage Numbers r25. It retains the eight capture sites, ownership and damage rules, duplicate suppression, projection and placement, and Wilds, Classic and Rise presentation. Defaults are enabled, Wilds, size 1.0 and lifetime 0.9 seconds. This is a DEVELOPMENT candidate; gameplay has not been tested.

[Player guide](DISTRIBUTION_README.md), [build instructions](DEVELOPMENT.md), [code walkthrough](docs/CODE.md), [validation scope](tests/README.md) and [r2 results](docs/CANDIDATE_RESULTS.json) and [dependency provenance](THIRD_PARTY.md).

The standalone workflow is `./build.sh doctor`, `prepare`, `build`, `test`, `verify`, `package`, and `export --output DEST`. `test --host` omits ARM checks. Preparation, builds and packaging do not install or activate the plugin. The candidate contains `damage_numbers_rs.nro` and a namespaced digit TTF. Existing C++ source and r25 packages remain preserved.

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches Forge 1.5.0's native ARM32 declaration: 24 bytes, with context and its update cell. Forge shares trampoline storage across hooks. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the recorded eight-site GE audit.

The load callback leaves the loader requirement record untouched, so Forge version checks do not gate startup. The build targets the 1.5.0 native ABI; other loader layouts can be incompatible. Initialization takes no arguments, matching the loader callback. Game-code signatures and instruction checks remain in place. The preserved [r1 results](docs/R1_CANDIDATE_RESULTS.json) describe the older candidate.
