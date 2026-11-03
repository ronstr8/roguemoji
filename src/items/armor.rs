//! Wearable protection. Armor subtracts from incoming damage, reduced by the
//! attack's penetration; heavier pieces hamper dodging.
//!
//! Loaded from `data/armor/*.toml` (see `data/README.md`).

use serde::Deserialize;

use crate::content::{self, Loot, Price, Problems};

/// A handle to a loaded armor piece: an index into the armor table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Armor(u16);

#[derive(Debug)]
pub struct ArmorDef {
    /// The file name without `.toml`, e.g. `kevlar_vest`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub armor: i32,
    pub dodge_penalty: i32,
    pub desc: String,
    /// Hundredths of a pound.
    pub weight: u32,
    pub loot: Loot,
    /// Autovend price; never sold if zero.
    pub price: Price,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawArmor {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    armor: i32,
    #[serde(default)]
    dodge_penalty: i32,
    #[serde(default)]
    loot: Loot,
    #[serde(default)]
    price: Price,
}

fn build(id: String, raw: RawArmor) -> Result<ArmorDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.check(raw.armor >= 0, "armor cannot be negative");
    p.check(raw.dodge_penalty >= 0, "dodge_penalty cannot be negative");
    p.finish(|| ArmorDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        armor: raw.armor,
        dodge_penalty: raw.dodge_penalty,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        loot: raw.loot,
        price: raw.price,
    })
}

content::registry!(Armor, ArmorDef, "armor", load);

fn load() -> Result<Vec<ArmorDef>, String> {
    let defs = content::load_defs("armor", build)?;
    let kits = crate::creatures::Background::ALL.into_iter().filter_map(|b| b.starting_gear().armor);
    content::require("armor", |id| defs.iter().any(|d| d.id == id), kits)?;
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_armor_loads() {
        load().expect("stock armor data is valid");
        let riot = Armor::named("riot_gear").def();
        assert_eq!((riot.armor, riot.loot.weight_at(3), riot.loot.weight_at(5)), (4, 0, 5));
    }
}
