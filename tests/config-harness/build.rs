// Compile the pinned config module unchanged except for its relocated submodule path.
use std::{env, fs, path::Path};
fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../vendor/forge-rs/forge/src")
        .canonicalize()
        .unwrap();
    let source = fs::read_to_string(root.join("config.rs")).unwrap().replace(
        "pub mod ini;",
        &format!("#[path={:?}] pub mod ini;", root.join("config/ini.rs")),
    );
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("config.rs"),
        source,
    )
    .unwrap();
    println!(
        "cargo:rerun-if-changed={}",
        root.join("config.rs").display()
    );
}
