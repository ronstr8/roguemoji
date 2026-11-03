//! Food and drink: crafted buffs and scavenged provisions, loaded from
//! `data/consumables/*.toml` (see `data/README.md`).

use serde::Deserialize;

use crate::content::{self, Problems};
use crate::effects::{EffectSpec, RawEffect};

/// A handle to a loaded consumable: an index into the consumable table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Consumable(u16);

#[derive(Debug)]
pub struct ConsumableDef {
    /// The file name without `.toml`, e.g. `herbal_tea`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub desc: String,
    /// Hundredths of a pound.
    pub weight: u32,
    /// Hunger restored.
    pub satiety: i32,
    /// Immediate healing.
    pub heal: i32,
    /// Removes poison and bleeding.
    pub cures: bool,
    pub effects: Vec<EffectSpec>,
    /// Relative chance among food found on the floor or sold at Autovends; 0 = never.
    pub loot_weight: u32,
    /// Autovend price in crystals.
    pub price: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConsumable {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    satiety: i32,
    #[serde(default)]
    heal: i32,
    #[serde(default)]
    cures: bool,
    #[serde(default)]
    effects: Vec<RawEffect>,
    #[serde(default)]
    loot_weight: u32,
    #[serde(default = "default_price")]
    price: u32,
}

fn default_price() -> u32 {
    5
}

fn build(id: String, raw: RawConsumable) -> Result<ConsumableDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.check(raw.satiety >= 0, "satiety cannot be negative");
    p.check(raw.heal >= 0, "heal cannot be negative");
    let effects: Vec<EffectSpec> = raw.effects.iter().map(RawEffect::spec).collect();
    p.check(effects.iter().all(|e| e.turns > 0), "effect turns must be positive");
    p.finish(|| ConsumableDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        satiety: raw.satiety,
        heal: raw.heal,
        cures: raw.cures,
        effects,
        loot_weight: raw.loot_weight,
        price: raw.price,
    })
}

content::registry!(Consumable, ConsumableDef, "consumable", load);

fn load() -> Result<Vec<ConsumableDef>, String> {
    let defs = content::load_defs("consumables", build)?;
    // Consumables the game's code names (starting kits).
    let kits = crate::creatures::Background::ALL.into_iter().flat_map(|b| b.starting_food().iter().map(|(id, _)| *id));
    content::require("consumables", |id| defs.iter().any(|d| d.id == id), kits)?;
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::EffectKind;
    use crate::stats::Attribute;

    fn parse(text: &str) -> Result<ConsumableDef, String> {
        let raw: RawConsumable = toml::from_str(text).map_err(|e| e.to_string())?;
        build("t".into(), raw).map_err(|e| e.join("; "))
    }

    const GOOD: &str = r#"
        name = "Test Soup"
        glyph = "🍲"
        description = "x"
        weight = 1
        satiety = 100
        effects = [{ kind = "boost", attribute = "wit", potency = 2, turns = 50 }]
    "#;

    #[test]
    fn stock_consumables_load() {
        load().expect("stock consumable data is valid");
        assert_eq!(Consumable::named("herbal_tea").def().name, "Herbal Tea");
    }

    #[test]
    fn parses_and_defaults() {
        let d = parse(GOOD).unwrap();
        assert_eq!((d.weight, d.heal, d.cures, d.price, d.loot_weight), (100, 0, false, 5, 0));
        assert_eq!(d.effects, vec![EffectSpec::new(EffectKind::Boost(Attribute::Wit), 2, 50)]);
    }

    #[test]
    fn rejects_bad_content() {
        assert!(parse(&GOOD.replace("\"wit\"", "\"charisma\"")).is_err());
        assert!(parse(&GOOD.replace("\"boost\"", "\"teleport\"")).is_err());
        assert!(parse(&GOOD.replace("turns = 50", "turns = 0")).unwrap_err().contains("turns"));
        assert!(parse(&GOOD.replace("satiety = 100", "satiety = -1")).is_err());
    }
}
