# Damage Numbers in Rust

`r1` ports Damage Numbers r25 to Rust with forge-rs. It retains the eight capture sites, ownership and damage rules, duplicate suppression, projection and placement, and Wilds, Classic and Rise presentation. Defaults are enabled, Wilds, size 1.0 and lifetime 0.9 seconds. This is a DEVELOPMENT candidate; gameplay has not been tested.

[Player guide](DISTRIBUTION_README.md), [build instructions](DEVELOPMENT.md), [code walkthrough](docs/CODE.md), [validation scope](tests/README.md) and [r1 results](docs/CANDIDATE_RESULTS.json) and [dependency provenance](THIRD_PARTY.md).

The standalone workflow is `./build.sh doctor`, `prepare`, `build`, `test`, `verify`, `package`, and `export --output DEST`. `test --host` omits ARM checks. Preparation, builds and packaging do not install or activate the plugin. The candidate contains `damage_numbers_rs.nro` and a namespaced digit TTF. Existing C++ source and r25 packages remain preserved.

Forge owns entry, callbacks, logging, memory lookup, thread-local storage, singleton lookup, configuration, Patch objects, native trampoline construction and ImGui wrappers. Specialized font/texture operations use its raw ImGui bindings. TLS borrows end before assembly invokes native continuations. Queued records live in independent process-lifetime mailboxes, so thread teardown cannot invalidate them.

The vendored Hook binding matches the maintained native ARM32 declaration: 76 bytes, including context and two 28-byte Jit structures. `verify` compares compiler-derived Rust and C++ layouts in the actual target artifact. Assembly preserves NZCVQ and all GPR/VFP/FPSCR state under the recorded eight-site GE audit.
