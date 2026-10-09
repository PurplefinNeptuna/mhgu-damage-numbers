# Build and validate

Copy `build.local.example.json` to ignored `build.local.json` and supply the tool paths and copied native inputs. Use Rust nightly-2026-10-02 with rust-src, the pinned cargo-forge binary built from forge-rs revision `7f572ed8d0e0a6cfc562571fdbc1ad57441a8438`, devkitARM GCC 15.2.0, and the Python dependencies recorded in requirements files. The Rust compiler and cargo-forge binary fingerprints are in `dependencies.lock.json`. No installed emulator package is an input.

`inputs` points to a directory with `main`, `sdk`, and the maintained Forge NSO named `forge`. Expected hashes are recorded in the lock. `main` must be the MHGU 1.4.0 executable with build ID `FB08F1D20FD1204FE7070672777F869B`. Preserve supplied originals. Configure `arm_python` separately if Unicorn and the asset/visual Python packages are installed in different environments.

| Command | Purpose |
| --- | --- |
| `./build.sh doctor` | Validate the compiler, converter and owned dependency fingerprints. |
| `./build.sh prepare` | Generate deterministic digit TTF and procedural atlases offline. |
| `./build.sh prepare --fetch` | Explicitly fetch checksum-pinned dependencies when refreshing inputs. |
| `./build.sh build` | Build ARM32 ELF/NRO and stage the runtime NRO and namespaced font. |
| `./build.sh test --host` | Run portable behavior, pinned config, production installer, C++ parity and real ImGui comparison tests. |
| `./build.sh test` | Add execution of the production ARM gateways and native verification. Requires a completed build. |
| `./build.sh verify` | Check exact native imports, callback exports, game windows, layouts, asset hashes and NZCVQ restore instructions. |
| `./build.sh package` | Require complete tests for the exact NRO, then verify the candidate ZIP and adjacent guides/notices/checksums. |
| `./build.sh export --output DEST` | Export the explicit source allowlist to an unused directory. |

Workspace outputs are under `build/damage-numbers-rs/candidates/r1/` and `work/damage-numbers-rs/candidates/r1/`. Cargo caches, the unstripped ELF and converter output stay under `work/damage-numbers-rs/candidates/r1/cargo-target/`. The pinned converter requires a temporary `target` symlink and loose NRO in the source root while building; the wrapper removes both when the converter exits, including on failure. An independent source export uses its own ignored build/work folders. Registry sources are vendored, including metadata needed by this nightly's build-std resolver. Host tests explicitly load the source-only Cargo configuration so they do not inherit the ARM target or build-std settings. Default builds and tests are offline.

After adding source files, refresh the allowlist with `python3 scripts/export_source.py --refresh`. Exports omit native game inputs, outputs, private configurations, caches and Git metadata. They retain dependency bytes exactly so Cargo checksum validation remains meaningful. Supply your toolchain/input paths in the exported repo's own configuration. No sibling mod import is needed.

The root distribution packager reserves releases for approved READY_TO_DISTRIBUTE records. This DEVELOPMENT mod's `package` command creates a verified candidate under `candidates/r1/delivery/`; it does not promote the status. Run `python3 scripts/mod_distribution.py check` from the coordinating workspace to validate every registered mod and preserve the existing releases. Gameplay, installation, activation, publication and push remain separate user-requested actions.
