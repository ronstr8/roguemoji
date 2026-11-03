//! Data-driven content: TOML files grouped by category (`monsters`, …).
//!
//! The stock files in the repository's `data/` directory are compiled into the
//! binary. At runtime, extra files are read from `data/` next to the
//! executable and from the directory named by `ROGUEMOJI_DATA`, in that order.
//! A file's id is its name without `.toml`; a later file with the same id
//! replaces an earlier one, so drop-ins can both add and override content.
//! Entries are returned sorted by id so a seed always produces the same world.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

mod builtin {
    include!(concat!(env!("OUT_DIR"), "/builtin_data.rs"));
}

/// One content file's text and where it came from (for error messages).
pub struct Source {
    pub id: String,
    pub origin: String,
    pub text: String,
}

fn runtime_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("data"))) {
        dirs.push(exe_dir);
    }
    if let Some(dir) = std::env::var_os("ROGUEMOJI_DATA") {
        dirs.push(PathBuf::from(dir));
    }
    dirs
}

/// All sources for a category: built-ins, then runtime drop-ins.
pub fn sources(category: &str) -> Result<Vec<Source>, String> {
    let mut by_id: BTreeMap<String, Source> = BTreeMap::new();
    for (cat, id, text) in builtin::BUILTIN {
        if *cat == category {
            let origin = format!("(built-in) data/{cat}/{id}.toml");
            by_id.insert(id.to_string(), Source { id: id.to_string(), origin, text: text.to_string() });
        }
    }
    for dir in runtime_dirs() {
        let Ok(entries) = fs::read_dir(dir.join(category)) else { continue };
        let mut paths: Vec<PathBuf> =
            entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "toml")).collect();
        paths.sort();
        for path in paths {
            let id = path.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            by_id.insert(id.clone(), Source { id, origin: path.display().to_string(), text });
        }
    }
    Ok(by_id.into_values().collect())
}

/// Parse every source in a category into `T`, collecting all errors.
pub fn load<T: DeserializeOwned>(category: &str) -> Result<Vec<(Source, T)>, String> {
    let mut out = Vec::new();
    let mut errors = Vec::new();
    for src in sources(category)? {
        match toml::from_str::<T>(&src.text) {
            Ok(v) => out.push((src, v)),
            Err(e) => errors.push(format!("{}: {e}", src.origin)),
        }
    }
    if errors.is_empty() { Ok(out) } else { Err(errors.join("\n")) }
}

/// Parse a category and convert each entry with `build(id, raw)`, which
/// returns its validation problems. All errors are reported together.
pub fn load_defs<R: DeserializeOwned, D>(
    category: &str,
    build: impl Fn(String, R) -> Result<D, Vec<String>>,
) -> Result<Vec<D>, String> {
    let mut defs = Vec::new();
    let mut errors = Vec::new();
    for (src, raw) in load::<R>(category)? {
        match build(src.id, raw) {
            Ok(def) => defs.push(def),
            Err(errs) => errors.extend(errs.into_iter().map(|e| format!("{}: {e}", src.origin))),
        }
    }
    if defs.is_empty() && errors.is_empty() {
        errors.push(format!("no {category} defined (data/{category}/*.toml)"));
    }
    if defs.len() > u16::MAX as usize {
        errors.push(format!("too many {category} definitions"));
    }
    if errors.is_empty() { Ok(defs) } else { Err(errors.join("\n")) }
}

/// Collects validation failures for one content file.
#[derive(Default)]
pub struct Problems(pub Vec<String>);

impl Problems {
    pub fn check(&mut self, ok: bool, msg: impl Into<String>) {
        if !ok {
            self.0.push(msg.into());
        }
    }

    pub fn finish<D>(self, def: impl FnOnce() -> D) -> Result<D, Vec<String>> {
        if self.0.is_empty() { Ok(def()) } else { Err(self.0) }
    }
}
