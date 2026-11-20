//! Food and drink: crafted buffs and scavenged provisions, loaded from
//! `data/consumables/*.toml` (see `data/README.md`).

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::content::{self, Price, Problems};
use crate::creatures::needs::{self, Need};
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
    /// Thirst quenched; negative for drinks that dry you out.
    pub hydration: i32,
    /// Immediate healing.
    pub heal: i32,
    /// Removes poison and bleeding.
    pub cures: bool,
    /// Changes to any need, e.g. Hope +350 and Satiety −100.
    pub needs: Vec<(Need, i32)>,
    pub effects: Vec<EffectSpec>,
    /// Logged on use in place of "You consume the …".
    pub message: Option<String>,
    /// Kept after use, and usable again once its cooldown passes.
    pub reusable: Option<Reusable>,
    /// Groups a recipe can ask for by name, e.g. `water`.
    pub tags: Vec<String>,
    /// Relative chance among food found on the floor or sold at Autovends; 0 = never.
    pub loot_weight: u32,
    /// Autovend price; never sold if zero.
    pub price: Price,
}

/// A consumable that isn't used up: it rests between uses instead.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reusable {
    /// Turns after a use before it works again.
    pub cooldown: u32,
    /// Logged when you try it too soon.
    pub refusal: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConsumable {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    #[serde(default)]
    satiety: i32,
    #[serde(default)]
    hydration: i32,
    #[serde(default)]
    heal: i32,
    #[serde(default)]
    cures: bool,
    #[serde(default)]
    needs: BTreeMap<String, i32>,
    #[serde(default)]
    effects: Vec<RawEffect>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    reusable: Option<Reusable>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    loot_weight: u32,
    #[serde(default = "default_price")]
    price: Price,
}

fn default_price() -> Price {
    Price::flat(5)
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
    let needs = needs::resolve(raw.needs, &mut p, "needs");
    if let Some(r) = &raw.reusable {
        p.check(r.cooldown > 0, "reusable.cooldown must be 1 or more turns");
        p.check(!r.refusal.trim().is_empty(), "reusable.refusal is empty");
    }
    p.finish(|| ConsumableDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        satiety: raw.satiety,
        hydration: raw.hydration,
        heal: raw.heal,
        cures: raw.cures,
        needs,
        effects,
        message: raw.message,
        reusable: raw.reusable,
        tags: raw.tags,
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
        assert_eq!((d.weight, d.heal, d.cures, d.price.base, d.loot_weight), (100, 0, false, 5, 0));
        assert_eq!(d.hydration, 0);
        assert_eq!(d.effects, vec![EffectSpec::new(EffectKind::Boost(Attribute::Wit), 2, 50)]);
    }

    #[test]
    fn rejects_bad_content() {
        assert!(parse(&GOOD.replace("\"wit\"", "\"charisma\"")).is_err());
        assert!(parse(&GOOD.replace("\"boost\"", "\"teleport\"")).is_err());
        assert!(parse(&GOOD.replace("turns = 50", "turns = 0")).unwrap_err().contains("turns"));
        assert!(parse(&GOOD.replace("satiety = 100", "satiety = -1")).is_err());
        assert_eq!(parse(&GOOD.replace("satiety = 100", "hydration = -50")).unwrap().hydration, -50);
    }

    #[test]
    fn reads_needs_messages_and_reuse() {
        let d = parse(&format!(
            "{GOOD}needs = {{ hope = 300, hunger = -50 }}\nmessage = \"Ahh.\"\nreusable = {{ cooldown = 40, refusal = \"Not yet.\" }}"
        ))
        .unwrap();
        assert_eq!(d.needs.len(), 2);
        assert_eq!(d.message.as_deref(), Some("Ahh."));
        assert_eq!(d.reusable.map(|r| r.cooldown), Some(40));
        assert!(parse(&format!("{GOOD}needs = {{ joy = 1 }}")).unwrap_err().contains("joy"));
        assert!(
            parse(&format!("{GOOD}reusable = {{ cooldown = 0, refusal = \"x\" }}")).unwrap_err().contains("cooldown")
        );
    }
}
