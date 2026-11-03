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

/// Fail if any id the game's code names is missing from `defs`.
pub fn require<'a>(
    category: &str,
    have: impl Fn(&str) -> bool,
    ids: impl IntoIterator<Item = &'a str>,
) -> Result<(), String> {
    let mut missing: Vec<&str> = ids.into_iter().filter(|id| !have(id)).collect();
    missing.sort_unstable();
    missing.dedup();
    let msgs: Vec<String> =
        missing.iter().map(|id| format!("data/{category}/{id}.toml is missing, but the game uses `{id}`")).collect();
    if msgs.is_empty() { Ok(()) } else { Err(msgs.join("\n")) }
}

/// How often a piece of gear turns up as loot, by depth:
/// `weight + per_depth × depth` from `min_depth` on, otherwise never.
/// Absent in a file means the item is never found as loot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Loot {
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub per_depth: u32,
    #[serde(default = "one")]
    pub min_depth: u32,
}

fn one() -> u32 {
    1
}

impl Loot {
    pub fn weight_at(&self, depth: u32) -> u32 {
        if depth >= self.min_depth { self.weight + self.per_depth * depth } else { 0 }
    }
}

/// What something costs at an Autovend, in crystals: `base + per_depth × depth`.
/// Written as a plain number (`price = 25`) or as
/// `price = { base = 25, per_depth = 3 }`. Zero means it's never sold.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(from = "RawPrice")]
pub struct Price {
    pub base: u32,
    pub per_depth: u32,
}

#[derive(serde::Deserialize)]
#[serde(untagged, expecting = "a price: a number, or { base = n, per_depth = n }")]
enum RawPrice {
    Flat(u32),
    Scaled {
        base: u32,
        #[serde(default)]
        per_depth: u32,
    },
}

impl From<RawPrice> for Price {
    fn from(raw: RawPrice) -> Price {
        match raw {
            RawPrice::Flat(base) => Price { base, per_depth: 0 },
            RawPrice::Scaled { base, per_depth } => Price { base, per_depth },
        }
    }
}

impl Price {
    pub const fn flat(base: u32) -> Price {
        Price { base, per_depth: 0 }
    }

    pub fn at(&self, depth: u32) -> u32 {
        self.base + self.per_depth * depth
    }

    pub fn for_sale(&self) -> bool {
        self.base > 0 || self.per_depth > 0
    }
}

/// Defines a content registry in the calling module: a `static` table loaded
/// once by `$load`, an `init()` that reports errors, and on the handle type
/// (a `u16` newtype) `all`, `lookup`, `named` and `def`. `$def` must have an
/// `id: String` field.
macro_rules! registry {
    ($handle:ident, $def:ty, $what:literal, $load:path) => {
        static TABLE: std::sync::OnceLock<Vec<$def>> = std::sync::OnceLock::new();

        /// Load this content now, reporting bad files as an error.
        pub fn init() -> Result<(), String> {
            if TABLE.get().is_none() {
                let _ = TABLE.set($load()?);
            }
            Ok(())
        }

        fn table() -> &'static [$def] {
            TABLE.get_or_init(|| $load().unwrap_or_else(|e| panic!(concat!("invalid ", $what, " data:\n{}"), e)))
        }

        #[allow(dead_code)]
        impl $handle {
            pub fn all() -> impl Iterator<Item = $handle> {
                (0..table().len() as u16).map($handle)
            }

            pub fn lookup(id: &str) -> Option<$handle> {
                table().iter().position(|d| d.id == id).map(|i| $handle(i as u16))
            }

            /// Something the game's code refers to by id; validated at startup.
            pub fn named(id: &str) -> $handle {
                $handle::lookup(id).unwrap_or_else(|| panic!(concat!($what, " `{}` is not defined"), id))
            }

            pub fn def(self) -> &'static $def {
                &table()[self.0 as usize]
            }
        }
    };
}
pub(crate) use registry;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct T {
        price: Price,
    }

    fn price(text: &str) -> Result<Price, String> {
        toml::from_str::<T>(text).map(|t| t.price).map_err(|e| e.to_string())
    }

    #[test]
    fn prices_parse_flat_or_scaled() {
        assert_eq!(price("price = 7").unwrap(), Price::flat(7));
        let p = price("price = { base = 25, per_depth = 3 }").unwrap();
        assert_eq!((p.at(0), p.at(4)), (25, 37));
        assert_eq!(price("price = { base = 9 }").unwrap(), Price::flat(9));
        assert!(price("price = -1").is_err());
        assert!(price("price = { cost = 5 }").unwrap_err().contains("a price"));
        assert!(!Price::default().for_sale());
    }
}
