//! The bestiary, loaded from `data/monsters/*.toml` (see `data/README.md`).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::content;
use crate::effects::{EffectKind, EffectSpec};
use crate::items::Component;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Animal,
    Mutant,
    Cryptid,
}

impl Family {
    pub fn label(self) -> &'static str {
        match self {
            Family::Animal => "Animal",
            Family::Mutant => "Mutant",
            Family::Cryptid => "Cryptid",
        }
    }
}

/// What a monster's solid hit inflicts beyond damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnHit {
    None,
    Inflict(EffectSpec),
    /// Willpower check (target 12) or be stunned by sheer terror.
    Dread,
}

/// A handle to a loaded species: an index into the bestiary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Species(u16);

#[derive(Debug)]
pub struct SpeciesDef {
    /// The file name without `.toml`, e.g. `feral_dog`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    pub family: Family,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub damage: (i32, i32),
    pub armor: i32,
    /// Energy per turn; 100 is normal speed.
    pub speed: i32,
    pub crystals: (u32, u32),
    pub min_depth: u32,
    pub max_depth: Option<u32>,
    /// Relative spawn weight before depth adjustments.
    pub weight: u32,
    pub on_hit: OnHit,
    pub drops: Vec<(Component, f64)>,
    pub desc: String,
}

// ── File format ──

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpecies {
    name: String,
    glyph: String,
    family: Family,
    description: String,
    hp: i32,
    attack: i32,
    defense: i32,
    damage: (i32, i32),
    #[serde(default)]
    armor: i32,
    #[serde(default = "default_speed")]
    speed: i32,
    crystals: (u32, u32),
    #[serde(default)]
    on_hit: Option<RawOnHit>,
    #[serde(default)]
    drops: Vec<RawDrop>,
    spawn: RawSpawn,
}

fn default_speed() -> i32 {
    100
}

#[derive(Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case", deny_unknown_fields)]
enum RawOnHit {
    Poison { potency: i32, turns: u32 },
    Bleed { potency: i32, turns: u32 },
    Stun { turns: u32 },
    Dread,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDrop {
    item: Component,
    chance: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpawn {
    min_depth: u32,
    #[serde(default)]
    max_depth: Option<u32>,
    weight: u32,
}

impl RawSpecies {
    fn into_def(self, id: String) -> Result<SpeciesDef, Vec<String>> {
        let mut errs = Vec::new();
        let mut check = |ok: bool, msg: &str| {
            if !ok {
                errs.push(msg.to_string());
            }
        };
        check(!self.name.trim().is_empty(), "name is empty");
        check(!self.glyph.trim().is_empty(), "glyph is empty");
        check(self.hp > 0, "hp must be positive");
        check(self.speed > 0, "speed must be positive");
        check(self.damage.0 >= 0 && self.damage.0 <= self.damage.1, "damage must be [min, max] with 0 <= min <= max");
        check(self.crystals.0 <= self.crystals.1, "crystals must be [min, max] with min <= max");
        check(self.armor >= 0, "armor cannot be negative");
        check(self.spawn.min_depth >= 1, "spawn.min_depth must be at least 1");
        check(self.spawn.max_depth.is_none_or(|m| m >= self.spawn.min_depth), "spawn.max_depth is below min_depth");
        for d in &self.drops {
            check((0.0..=1.0).contains(&d.chance), "drop chance must be between 0 and 1");
        }
        let on_hit = match self.on_hit {
            None => OnHit::None,
            Some(RawOnHit::Poison { potency, turns }) => {
                OnHit::Inflict(EffectSpec::new(EffectKind::Poison, potency, turns))
            }
            Some(RawOnHit::Bleed { potency, turns }) => {
                OnHit::Inflict(EffectSpec::new(EffectKind::Bleed, potency, turns))
            }
            Some(RawOnHit::Stun { turns }) => OnHit::Inflict(EffectSpec::new(EffectKind::Stun, 1, turns)),
            Some(RawOnHit::Dread) => OnHit::Dread,
        };
        if !errs.is_empty() {
            return Err(errs);
        }
        Ok(SpeciesDef {
            id,
            name: self.name,
            glyph: self.glyph,
            family: self.family,
            hp: self.hp,
            attack: self.attack,
            defense: self.defense,
            damage: self.damage,
            armor: self.armor,
            speed: self.speed,
            crystals: self.crystals,
            min_depth: self.spawn.min_depth,
            max_depth: self.spawn.max_depth,
            weight: self.spawn.weight,
            on_hit,
            drops: self.drops.into_iter().map(|d| (d.item, d.chance)).collect(),
            desc: self.description,
        })
    }
}

// ── Registry ──

static BESTIARY: OnceLock<Vec<SpeciesDef>> = OnceLock::new();

fn load() -> Result<Vec<SpeciesDef>, String> {
    let mut defs = Vec::new();
    let mut errors = Vec::new();
    for (src, raw) in content::load::<RawSpecies>("monsters")? {
        match raw.into_def(src.id) {
            Ok(def) => defs.push(def),
            Err(errs) => errors.extend(errs.into_iter().map(|e| format!("{}: {e}", src.origin))),
        }
    }
    if defs.is_empty() && errors.is_empty() {
        errors.push("no monsters defined (data/monsters/*.toml)".into());
    }
    if defs.len() > u16::MAX as usize {
        errors.push("too many monster definitions".into());
    }
    if errors.is_empty() { Ok(defs) } else { Err(errors.join("\n")) }
}

/// Load the bestiary now, reporting bad content files as an error.
pub fn init() -> Result<(), String> {
    if BESTIARY.get().is_none() {
        let _ = BESTIARY.set(load()?);
    }
    Ok(())
}

fn bestiary() -> &'static [SpeciesDef] {
    BESTIARY.get_or_init(|| load().unwrap_or_else(|e| panic!("invalid monster data:\n{e}")))
}

impl Species {
    pub fn all() -> impl Iterator<Item = Species> {
        (0..bestiary().len() as u16).map(Species)
    }

    #[allow(dead_code)]
    pub fn by_id(id: &str) -> Option<Species> {
        bestiary().iter().position(|d| d.id == id).map(|i| Species(i as u16))
    }

    pub fn def(self) -> &'static SpeciesDef {
        &bestiary()[self.0 as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<SpeciesDef, String> {
        let raw: RawSpecies = toml::from_str(text).map_err(|e| e.to_string())?;
        raw.into_def("test".into()).map_err(|e| e.join("; "))
    }

    const GOOD: &str = r#"
        name = "Test Beast"
        glyph = "🐀"
        family = "mutant"
        description = "For tests."
        hp = 5
        attack = 1
        defense = 9
        damage = [1, 2]
        crystals = [1, 2]
        on_hit = { effect = "poison", potency = 2, turns = 4 }
        drops = [{ item = "raw_meat", chance = 0.5 }]
        [spawn]
        min_depth = 2
        weight = 3
    "#;

    #[test]
    fn stock_bestiary_loads() {
        load().expect("stock monster data is valid");
        let rat = Species::by_id("rat").expect("rat exists");
        assert_eq!(rat.def().name, "Rat");
    }

    #[test]
    fn parses_and_defaults() {
        let d = parse(GOOD).unwrap();
        assert_eq!(d.speed, 100);
        assert_eq!(d.armor, 0);
        assert_eq!(d.on_hit, OnHit::Inflict(EffectSpec::new(EffectKind::Poison, 2, 4)));
        assert_eq!(d.drops, vec![(Component::RawMeat, 0.5)]);
    }

    #[test]
    fn rejects_bad_content() {
        assert!(parse(&GOOD.replace("hp = 5", "hp = 0")).unwrap_err().contains("hp"));
        assert!(parse(&GOOD.replace("raw_meat", "unobtainium")).is_err());
        assert!(parse(&GOOD.replace("attack = 1", "atack = 1")).is_err());
        assert!(parse(&GOOD.replace("damage = [1, 2]", "damage = [3, 2]")).unwrap_err().contains("damage"));
    }
}
