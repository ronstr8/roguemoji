//! Crafting components gathered by harvesting and from creature remains,
//! loaded from `data/components/*.toml` (see `data/README.md`).

use serde::Deserialize;

use crate::content::{self, Price, Problems};
use crate::creatures::needs::{RawUnsafeDrink, UnsafeDrink};

/// A handle to a loaded component: an index into the component table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Component(u16);

#[derive(Debug)]
pub struct ComponentDef {
    /// The file name without `.toml`, e.g. `duct_tape`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub desc: String,
    /// Hundredths of a pound.
    pub weight: u32,
    /// Relative chance among components found lying on the floor; 0 = never.
    pub loot_weight: u32,
    /// Autovend price of one; never sold if zero.
    pub price: Price,
    /// Drinkable as is, at a risk of sickness.
    pub drink: Option<UnsafeDrink>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawComponent {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    #[serde(default)]
    loot_weight: u32,
    #[serde(default = "default_price")]
    price: Price,
    #[serde(default)]
    drink: Option<RawUnsafeDrink>,
}

fn default_price() -> Price {
    Price::flat(2)
}

fn build(id: String, raw: RawComponent) -> Result<ComponentDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    let drink = raw.drink.as_ref().map(|d| d.build("drink", &mut p));
    p.finish(|| ComponentDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        loot_weight: raw.loot_weight,
        price: raw.price,
        drink,
    })
}

/// The component whose `drink` also says what drinking from a water tile does.
pub const UNSAFE_WATER: &str = "dirty_water";

content::registry!(Component, ComponentDef, "component", load);

fn load() -> Result<Vec<ComponentDef>, String> {
    let defs = content::load_defs("components", build)?;
    // Components the game's own tables name (harvest nodes, starting kits).
    let harvest = crate::world::Harvestable::ALL.into_iter().flat_map(|h| h.def().yields.iter().map(|(id, _, _)| *id));
    let kits =
        crate::creatures::Background::ALL.into_iter().flat_map(|b| b.starting_components().iter().map(|(id, _)| *id));
    let water = std::iter::once(UNSAFE_WATER);
    content::require("components", |id| defs.iter().any(|d| d.id == id), harvest.chain(kits).chain(water))?;
    if defs.iter().any(|d| d.id == UNSAFE_WATER && d.drink.is_none()) {
        return Err(format!("data/components/{UNSAFE_WATER}.toml needs `drink`: water tiles are drunk like it"));
    }
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_components_load() {
        load().expect("stock component data is valid");
        let tape = Component::named("duct_tape").def();
        assert_eq!(tape.name, "Duct Tape");
        assert_eq!(tape.weight, 30);
        assert!(Component::named(UNSAFE_WATER).def().drink.is_some());
    }

    #[test]
    fn rejects_bad_content() {
        let parse = |text: &str| -> Result<ComponentDef, String> {
            let raw: RawComponent = toml::from_str(text).map_err(|e| e.to_string())?;
            build("t".into(), raw).map_err(|e| e.join("; "))
        };
        let good = "name = \"Gizmo\"\nglyph = \"⚙\"\ndescription = \"x\"\nweight = 0.25\n";
        assert_eq!(parse(good).unwrap().weight, 25);
        assert_eq!(parse(good).unwrap().loot_weight, 0);
        assert!(parse(&good.replace("0.25", "-1")).unwrap_err().contains("weight"));
        assert!(parse(&format!("{good}colour = 1\n")).is_err());
        let drink =
            "drink = { hydration = 300, risk = 10, sickness = { kind = \"poison\", potency = 1, turns = 8 } }\n";
        assert_eq!(parse(&format!("{good}{drink}")).unwrap().drink.unwrap().hydration, 300);
        assert!(parse(&format!("{good}{}", drink.replace("300", "0"))).unwrap_err().contains("hydration"));
    }
}
