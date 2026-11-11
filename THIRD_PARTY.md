# Pinned development inputs

forge-rs is fetched at `d1865b68e9b194eb3eacc8bd71bbf601a5fac63e` (MIT, Fexty), version 1.5.0, with one local patch, `patches/forge-rs-c-char.patch`: four `.cast()` calls in the ImGui DrawList text wrappers, which the x86_64 host render harness needs because C `char` is `i8` there and `u8` on the ARM target. The NRO is identical with or without it. The 1.4.1 converter is supplied separately and fingerprinted. Loader metadata and the zero-argument native init callback are declared in owned source; vendored lifecycle macros are upstream. The upstream 24-byte Hook binding is used unchanged. Config/TLS/singleton/Patch/font helpers retain upstream behavior. Native field offsets and imports are checked against the copied official 1.5.1 loader input.

Cargo.lock pins runtime/procedural-macro dependencies. `./build.sh fetch` puts their source and license files in ignored vendor/crates. The additional sysroot packages supply this nightly's full resolver metadata; only core/alloc/compiler_builtins are built into the ARM runtime. `research/sysroot-Cargo.lock` pins that metadata. WASI libraries are not built for this ARM32 plugin. Runtime component notices are separate from these build-only dependencies.

The pinned C++ oracle and the gateway were copied from this workspace's Damage Numbers r25 source baseline at parent commit `5c982f4`. Oracle code exists only in host tests; it is never linked into damage_numbers.nro. The production gateway is byte-identical to the existing audited assembly. Its GE audit and native declarations are retained under research for reproducible validation. ARM SVC wrappers originate in the project's maintained libnx integration, with the ENTRY macro namespaced to avoid assembler collisions.

The host renderer uses ImGui/cimgui 1.92.8 (cimgui `d298666`, imgui `8936b58`), which `./build.sh fetch` puts in ignored vendor/cimgui with Roboto Medium from Forge `b23e114`; their MIT licenses are copied in licenses/. The prepared font originated from Open Sans at Google Fonts revision `9710da1eacb3be272583c3224dcb70f9da6eadbb` and the Forge/Roboto Medium reference for matching metrics. Font licenses and input hashes are preserved. Roboto and the host renderer are development inputs; the runtime ships the prepared Open Sans Semibold subset and two external procedural RGBA files. Font naming metadata was restored without changing outlines, hinting or metrics. Their current hashes and provenance are in assets/manifest.json.

The new Rust implementation, fixtures, recipes and explanatory documentation were written with OpenAI Codex assistance. Procedural artwork retains the prepared pixels from the previous authored generation recipes; this is code assistance, with no model-generated image or audio asset. Both RGBA files regenerate byte for byte from those recipes at commit `6a44aba` (`mods/damage-numbers-rs/scripts/prepare_damage_numbers_sprites.py` and `prepare_damage_numbers_rise.py`, with that commit's `requirements.txt`: fontTools 4.66.1, Pillow 12.3.0): `wilds-effects.rgba` from Open Sans digits and two coded four-point stars, `rise-slash.rgba` from a closed-form beam. Neither recipe reads game files. The explicit platform disclosure is recorded in DISTRIBUTION.json.

The native allocator fixture retains Forge `hook.c` and `hook.h` from revision `1671ad1526f6df5ee8d6e61a03518e0a427e202e`, with their MIT copyright/license text. It is development-only and is not linked into the NRO. The fixture substitutes host platform operations and omits the two ARM barriers while keeping allocator and branch construction source unchanged.

The frozen C++ asset headers and decoder are test-only reference inputs. The 24 r2 golden images and atlas hashes were captured from production renderer source at `56b804c` before changing its asset-loading or drawing implementation. They contain generated test scenes and licensed font/artwork pixels, not game captures.

Rust `core` and `alloc` are compiled into the ARM plugin by `.cargo/config.toml`
with `compiler_builtins` memory support. The pinned toolchain is
`c36f1457196e315bc204b9564a6a5a7fe7f5a51f`. `licenses/RUST-MIT.txt` is copied
unchanged from
https://github.com/rust-lang/rust/blob/c36f1457196e315bc204b9564a6a5a7fe7f5a51f/LICENSE-MIT
(SHA-256 `b71bd43a069ca0641a9ecfe585ca7b3c53b5cc1608f8b68321168698e28b5ea1`).
These notices cover the linked libraries. The compiler and host-only dependencies
are build tools and are excluded from release notices.

The linked ARM ELF contains
`core::unicode::unicode_data::white_space::WHITESPACE_MAP` as well as
`core`, `alloc` and `compiler_builtins` symbols (`nm -C --defined-only`).
The pinned toolchain's `COPYRIGHT-library.html` identifies
`library/core/src/unicode` as Unicode-3.0, copyright 1991-2024 Unicode, Inc.
`licenses/UNICODE-3.0.txt` is copied unchanged from that toolchain's
`share/doc/rust/licenses/Unicode-3.0.txt`
(SHA-256 `f5062c9a188d81dfe66b56db4182dcf9e4b17c0d9b0d311a8e20b3a1b075c443`).
