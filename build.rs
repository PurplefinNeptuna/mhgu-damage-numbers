use std::{env, fs, path::Path};
fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let assets = env::var("DN_ASSETS").expect("run build.sh prepare before building");
    println!("cargo:rustc-link-arg=-T{dir}/link/switch32.ld");
    println!("cargo:rustc-link-arg=-Wl,--build-id=sha1");
    println!("cargo:rerun-if-changed=link/switch32.ld");
    println!("cargo:rerun-if-env-changed=DN_ASSETS");
    // Generated constants include only authored sprite streams and deterministic font metrics.
    let source = fs::read_to_string(Path::new(&assets).join("assets.rs")).unwrap();
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        source,
    )
    .unwrap();
    println!("cargo:rerun-if-changed={assets}/assets.rs");
}
