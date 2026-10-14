# Build and validate

Copy `build.local.example.json` to ignored `build.local.json` and supply the tool paths and copied native inputs. Use Rust nightly-2026-10-02 with rust-src, the pinned cargo-forge 1.4.1 converter built from revision `7f572ed8d0e0a6cfc562571fdbc1ad57441a8438` (retained independently of the 1.5.0 runtime crates), devkitARM GCC 15.2.0, and the Python dependencies recorded in requirements files. The Rust compiler and cargo-forge binary fingerprints are in `dependencies.lock.json`. No installed emulator package is an input.

`inputs` points to a directory with `main`, `sdk`, and the mutex-corrected Forge 1.5.0 NSO named `forge`. Expected hashes are recorded in the lock. `main` must be the MHGU 1.4.0 executable with build ID `FB08F1D20FD1204FE7070672777F869B`. Preserve supplied originals. Configure `arm_python` separately if Unicorn and the visual-test Python packages are installed in different environments.

| Command | Purpose |
| --- | --- |
| `./build.sh doctor` | Validate the compiler, converter and owned dependency fingerprints. |
| `./build.sh prepare` | Validate the prepared assets without generating or compressing data. |
| `./build.sh prepare --fetch` | Explicitly fetch checksum-pinned dependencies when refreshing inputs. |
| `./build.sh build` | Build ARM32 ELF/NRO and stage the NRO, OpenSans.ttf and both RGBA files. |
| `./build.sh test --host` | Run portable behavior, pinned config, production installer, native pooled allocator, C++ parity and real ImGui comparison tests. |
| `./build.sh test` | Add execution of the production ARM gateways and native verification. Requires a completed build. |
| `./build.sh verify` | Check exact native imports, callback exports, game windows, layouts, asset hashes and NZCVQ restore instructions. |
| `./build.sh package` | Require complete tests for the exact NRO, then verify the candidate ZIP and adjacent guides/notices/checksums. |
| `./build.sh export --output DEST` | Export the explicit source allowlist to an unused directory. |

Workspace outputs are under `build/damage-numbers-rs/candidates/r3/` and `work/damage-numbers-rs/candidates/r3/`. Cargo caches, the unstripped ELF and converter output stay under `work/damage-numbers-rs/candidates/r3/cargo-target/`. The pinned converter requires a temporary `target` symlink and loose NRO in the source root while building; the wrapper removes both when the converter exits, including on failure. An independent source export uses its own ignored build/work folders. Registry sources are vendored, including metadata needed by this nightly's build-std resolver. Host tests explicitly load the source-only Cargo configuration so they do not inherit the ARM target or build-std settings. Default builds and tests are offline.

After adding source files, refresh the allowlist with `python3 scripts/export_source.py --refresh`. Exports omit native game inputs, outputs, private configurations, caches and Git metadata. They retain dependency bytes exactly so Cargo checksum validation remains meaningful. Supply your toolchain/input paths in the exported repo's own configuration. No sibling mod import is needed.

The root distribution packager reserves releases for approved READY_TO_DISTRIBUTE records. This DEVELOPMENT mod's `package` command creates a verified candidate under `candidates/r3/delivery/`; it does not promote the status. Run `python3 scripts/mod_distribution.py check` from the coordinating workspace to validate every registered mod and preserve the existing releases. Gameplay, installation, activation, publication and push remain separate user-requested actions.

forge-rs source is pinned at `d1865b68e9b194eb3eacc8bd71bbf601a5fac63e`; the native hook fixture is pinned to Forge `1671ad1526f6df5ee8d6e61a03518e0a427e202e`. The copied loader input has SHA-256 `1e452243cc85108a2271dda99167f9728d051887075766b38e36a1ce94a29b42`. The native fixture compiles the actual allocator with host JIT/TLS/cache doubles, omits ARM barriers, and uses ASan/UBSan. LeakSanitizer is disabled under sandbox tracing; the fixture counts creation and closure of every mock JIT chunk. These checks do not measure emulator allocation or establish gameplay acceptance.

The owned `assets/` folder contains the exact prepared runtime files and their hash/provenance manifest. Ordinary builds need no FontTools or image-generation dependencies. `requirements.txt` supplies Pillow/NumPy only for visual tests. Deleted authoring tools remain recoverable from Git; they are not needed by standalone builds.

`tests/r2-pixels/` holds 24 reference scenes captured from untouched r2 rendering before this refactor, at 720p/1080p and the minimum/maximum scale. Atlas hashes and composited RGB pixels must match exactly. The frozen C++ oracle headers and decoder remain test-only under `tests/oracle/`; no reference artwork or decoder is linked into the production NRO.

External RGBA files are opened through Forge's exposed ImGui APIs, length-checked before texture allocation, read directly into the pixel buffer, and closed on every return. Wilds loads during initialization; Rise loads on its first critical effect. Failed assets are not retried in the same context. Each new context retries loading, while failure logs remain deduplicated for the resource owner's process lifetime. Independent context invalidation prevents stale font handles even when no artwork texture could be registered.

The r3 standalone build, full tests and packaging passed offline with an empty Cargo home. NRO hashes differ across build roots; both exact artifacts passed complete validation. See `docs/STANDALONE_RESULTS.json` for exact fingerprints.
