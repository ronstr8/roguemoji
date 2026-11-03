//! Embeds the stock content files under `data/` into the binary, so the game
//! runs as a single executable. Generates `$OUT_DIR/builtin_data.rs`.

use std::fmt::Write as _;
use std::path::Path;
use std::{env, fs};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    println!("cargo:rerun-if-changed=data");

    let mut entries = Vec::new();
    if let Ok(categories) = fs::read_dir(&root) {
        for category in categories.flatten().filter(|e| e.path().is_dir()) {
            let cat_name = category.file_name().to_string_lossy().into_owned();
            for file in fs::read_dir(category.path()).unwrap().flatten() {
                let path = file.path();
                if path.extension().is_some_and(|x| x == "toml") {
                    let id = path.file_stem().unwrap().to_string_lossy().into_owned();
                    entries.push((cat_name.clone(), id, path));
                }
            }
        }
    }
    entries.sort();

    let mut out = String::from("pub const BUILTIN: &[(&str, &str, &str)] = &[\n");
    for (cat, id, path) in &entries {
        writeln!(out, "    ({cat:?}, {id:?}, include_str!({:?})),", path.display().to_string()).unwrap();
    }
    out.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("builtin_data.rs"), out).unwrap();
}
