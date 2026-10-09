// Share the production asset constants; visual fixtures never substitute their own artwork.
use std::{env, fs, path::Path};
fn main() {
    let assets = env::var("DN_ASSETS").unwrap();
    fs::copy(
        Path::new(&assets).join("assets.rs"),
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
    )
    .unwrap();
    println!("cargo:rerun-if-changed={assets}/assets.rs");
}
