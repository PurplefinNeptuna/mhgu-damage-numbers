use std::{env, fs, path::Path};
// Cargo runs this on the host before compiling the Switch plugin. It supplies
// linker options and moves prepared asset constants into OUT_DIR for render.rs;
// there is no asset generation or file loading inside a native damage hook.
fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets = env::var("DN_ASSETS").expect("run build.sh prepare before building");
    println!("cargo:rustc-link-arg=-T{dir}/link/switch32.ld");
    println!("cargo:rustc-link-arg=-Wl,--build-id=sha1");
    // These lines tell Cargo which files/environment values invalidate this
    // build step. A new asset directory or changed linker script triggers a rerun.
    println!("cargo:rerun-if-changed=link/switch32.ld");
    println!("cargo:rerun-if-env-changed=DN_ASSETS");
    // Generated constants include only authored sprite streams and deterministic font metrics.
    let source = fs::read_to_string(Path::new(&assets).join("assets.rs")).unwrap();
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        source,
    )
    .unwrap();
    // Watch prepared asset content too; retaining the same DN_ASSETS path must
    // not allow Cargo to reuse stale compressed atlas/font constants.
    println!("cargo:rerun-if-changed={assets}/assets.rs");
}
