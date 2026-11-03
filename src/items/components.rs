//! Crafting components gathered by harvesting and from creature remains,
//! loaded from `data/components/*.toml` (see `data/README.md`).

use serde::Deserialize;

use crate::content::{self, Problems};

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
}

fn build(id: String, raw: RawComponent) -> Result<ComponentDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.finish(|| ComponentDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        loot_weight: raw.loot_weight,
    })
}

content::registry!(Component, ComponentDef, "component", load);

fn load() -> Result<Vec<ComponentDef>, String> {
    let defs = content::load_defs("components", build)?;
    // Components the game's own tables name (harvest nodes, starting kits).
    let harvest = crate::world::Harvestable::ALL.into_iter().flat_map(|h| h.def().yields.iter().map(|(id, _, _)| *id));
    let kits =
        crate::creatures::Background::ALL.into_iter().flat_map(|b| b.starting_components().iter().map(|(id, _)| *id));
    content::require("components", |id| defs.iter().any(|d| d.id == id), harvest.chain(kits))?;
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
    }
}
