//! Melee weapons: scavenged tools, scrap-crafted improvisations, and rare
//! artifacts left behind by the visitors.

//!
//! Loaded from `data/melee/*.toml` (see `data/README.md`).

use std::sync::LazyLock;

use serde::Deserialize;

use crate::content::{self, Loot, Problems};
use crate::stats::Skill;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Tool,
    Crafted,
    Alien,
}

impl Origin {
    pub fn label(self) -> &'static str {
        match self {
            Origin::Tool => "Tool",
            Origin::Crafted => "Scrapcraft",
            Origin::Alien => "Visitor artifact",
        }
    }
}

/// Extra effect on a solid hit (Hit or Crit tier).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Special {
    #[default]
    None,
    Bleed,
    Stun,
    /// Shove the target one tile back and stun it briefly.
    Knockback,
    /// Armor is ignored entirely.
    IgnoreArmor,
    /// Heal a share of the damage dealt.
    Drain,
}

impl Special {
    pub fn describe(self) -> &'static str {
        match self {
            Special::None => "",
            Special::Bleed => "Solid hits cause bleeding.",
            Special::Stun => "Solid hits can stun.",
            Special::Knockback => "Solid hits hurl the target back.",
            Special::IgnoreArmor => "Passes through armor as if it weren't there.",
            Special::Drain => "Drinks a share of the damage as health.",
        }
    }
}

/// A handle to a loaded melee weapon: an index into the weapon table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MeleeWeapon(u16);

#[derive(Debug)]
pub struct MeleeDef {
    /// The file name without `.toml`, e.g. `fire_axe`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub origin: Origin,
    pub skill: Skill,
    pub damage: (i32, i32),
    pub accuracy: i32,
    pub pen: i32,
    pub special: Special,
    pub desc: String,
    /// Hundredths of a pound.
    pub weight: u32,
    pub loot: Loot,
}

/// Bare hands, used when nothing is equipped.
pub static FISTS: LazyLock<MeleeDef> = LazyLock::new(|| MeleeDef {
    id: "fists".into(),
    name: "Fists".into(),
    glyph: "👊".into(),
    origin: Origin::Tool,
    skill: Skill::Brawling,
    damage: (1, 3),
    accuracy: 1,
    pen: 0,
    special: Special::None,
    desc: "Your own two hands.".into(),
    weight: 0,
    loot: Loot::default(),
});

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMelee {
    name: String,
    glyph: String,
    description: String,
    /// Pounds.
    weight: f64,
    origin: Origin,
    skill: Skill,
    damage: (i32, i32),
    #[serde(default)]
    accuracy: i32,
    #[serde(default)]
    penetration: i32,
    #[serde(default)]
    special: Special,
    #[serde(default)]
    loot: Loot,
}

fn build(id: String, raw: RawMelee) -> Result<MeleeDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.check(matches!(raw.skill, Skill::Melee | Skill::Brawling), "skill must be melee or brawling");
    p.check(raw.damage.0 >= 0 && raw.damage.0 <= raw.damage.1, "damage must be [min, max] with 0 <= min <= max");
    p.check(raw.penetration >= 0, "penetration cannot be negative");
    p.finish(|| MeleeDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        origin: raw.origin,
        skill: raw.skill,
        damage: raw.damage,
        accuracy: raw.accuracy,
        pen: raw.penetration,
        special: raw.special,
        desc: raw.description,
        weight: (raw.weight * 100.0).round() as u32,
        loot: raw.loot,
    })
}

content::registry!(MeleeWeapon, MeleeDef, "melee weapon", load);

fn load() -> Result<Vec<MeleeDef>, String> {
    let defs = content::load_defs("melee", build)?;
    let kits = crate::creatures::Background::ALL.into_iter().flat_map(|b| b.starting_gear().melee_ids());
    content::require("melee", |id| defs.iter().any(|d| d.id == id), kits)?;
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_melee_loads() {
        load().expect("stock melee data is valid");
        let axe = MeleeWeapon::named("phase_edge").def();
        assert_eq!((axe.origin, axe.special), (Origin::Alien, Special::IgnoreArmor));
    }

    #[test]
    fn rejects_bad_content() {
        let parse = |text: &str| -> Result<MeleeDef, String> {
            let raw: RawMelee = toml::from_str(text).map_err(|e| e.to_string())?;
            build("t".into(), raw).map_err(|e| e.join("; "))
        };
        let ok = "name = \"Stick\"\nglyph = \"🪵\"\ndescription = \"x\"\nweight = 1\norigin = \"tool\"\nskill = \"melee\"\ndamage = [1, 2]\n";
        let d = parse(ok).unwrap();
        assert_eq!((d.special, d.loot.weight_at(5)), (Special::None, 0));
        assert!(parse(&ok.replace("\"melee\"", "\"survival\"")).unwrap_err().contains("skill"));
        assert!(parse(&ok.replace("\"tool\"", "\"magic\"")).is_err());
        assert!(parse(&format!("{ok}special = \"explode\"\n")).is_err());
    }
}
