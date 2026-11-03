//! Packs: worn carrying gear that raises how much you can haul before the
//! load starts slowing you down.
//!
//! Loaded from `data/packs/*.toml` (see `data/README.md`).

use serde::Deserialize;

use crate::content::{self, Loot, Price, Problems};

/// A handle to a loaded pack: an index into the pack table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pack(u16);

#[derive(Debug)]
pub struct PackDef {
    /// The file name without `.toml`, e.g. `hiking_pack`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    /// Extra carrying capacity, in hundredths of a pound.
    pub capacity: u32,
    pub desc: String,
    /// Hundredths of a pound.
    pub weight: u32,
    pub loot: Loot,
    /// Autovend price; never sold if zero.
    pub price: Price,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPack {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    /// Extra carrying capacity in pounds.
    capacity: f64,
    #[serde(default)]
    loot: Loot,
    #[serde(default)]
    price: Price,
}

fn build(id: String, raw: RawPack) -> Result<PackDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.check(raw.capacity >= 0.0 && raw.capacity.is_finite(), "capacity must be zero or more pounds");
    p.finish(|| PackDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        capacity: (raw.capacity * 100.0).round() as u32,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        loot: raw.loot,
        price: raw.price,
    })
}

content::registry!(Pack, PackDef, "pack", load);

fn load() -> Result<Vec<PackDef>, String> {
    content::load_defs("packs", build)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_packs_load() {
        load().expect("stock pack data is valid");
        assert_eq!(Pack::named("hiking_pack").def().capacity, 2500);
    }
}
