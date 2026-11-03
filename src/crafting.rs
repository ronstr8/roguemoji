//! Crafting recipes. Every craft is a Crafting check against the recipe's
//! difficulty; the tier decides the outcome:
//!
//! * Fumble — ingredients ruined.
//! * Fail — nothing made, ingredients kept (you just wasted time).
//! * Partial / Success — the item is made.
//! * Critical — a bonus portion (food) or a refund of one ingredient (gear).

//!
//! Recipes are loaded from `data/recipes/*.toml` (see `data/README.md`).

use std::sync::OnceLock;

use serde::Deserialize;

use crate::content::{self, Problems};
use crate::items::{Armor, Component, Consumable, Item, ItemKind, MeleeWeapon, Pack};

#[derive(Clone, Copy, Debug)]
pub enum Output {
    Food(Consumable),
    Melee(MeleeWeapon),
    Armor(Armor),
    Pack(Pack),
}

impl Output {
    pub fn item(self) -> Item {
        match self {
            Output::Food(c) => Item::food(c, 1),
            Output::Melee(m) => Item::one(ItemKind::Melee(m)),
            Output::Armor(a) => Item::one(ItemKind::Armor(a)),
            Output::Pack(p) => Item::one(ItemKind::Pack(p)),
        }
    }

    pub fn is_food(self) -> bool {
        matches!(self, Output::Food(_))
    }

    /// Menu grouping: food, then weapons, armor, carrying gear.
    fn group(self) -> u8 {
        match self {
            Output::Food(_) => 0,
            Output::Melee(_) => 1,
            Output::Armor(_) => 2,
            Output::Pack(_) => 3,
        }
    }
}

#[derive(Debug)]
pub struct Recipe {
    /// The file name without `.toml`.
    #[allow(dead_code)]
    pub id: String,
    pub output: Output,
    pub ingredients: Vec<(Component, u32)>,
    /// Extra non-component tool consumed (e.g. a bat to nail).
    pub consumes_tool: Option<MeleeWeapon>,
    pub difficulty: i32,
}

impl Recipe {
    pub fn name(&self) -> String {
        self.output.item().base_name()
    }

    pub fn glyph(&self) -> &'static str {
        self.output.item().glyph()
    }

    pub fn components(&self) -> impl Iterator<Item = (Component, u32)> + '_ {
        self.ingredients.iter().copied()
    }
}

// ── File format ──

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecipe {
    makes: RawOutput,
    difficulty: i32,
    /// `{ component_id = count, … }`, in the order written.
    ingredients: toml::Table,
    #[serde(default)]
    uses_up: Option<MeleeWeapon>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum RawOutput {
    Food(String),
    Melee(MeleeWeapon),
    Armor(Armor),
    Pack(Pack),
}

fn build(id: String, raw: RawRecipe) -> Result<Recipe, Vec<String>> {
    let mut p = Problems::default();
    let output = match raw.makes {
        RawOutput::Food(food) => match Consumable::lookup(&food) {
            Some(c) => Some(Output::Food(c)),
            None => {
                p.check(false, format!("makes food `{food}`, which is not in data/consumables/"));
                None
            }
        },
        RawOutput::Melee(m) => Some(Output::Melee(m)),
        RawOutput::Armor(a) => Some(Output::Armor(a)),
        RawOutput::Pack(k) => Some(Output::Pack(k)),
    };
    p.check(!raw.ingredients.is_empty(), "ingredients is empty");
    let mut ingredients = Vec::new();
    for (name, count) in &raw.ingredients {
        let n = count.as_integer().filter(|n| (1..=99).contains(n));
        p.check(n.is_some(), format!("ingredient `{name}` needs a count from 1 to 99"));
        match Component::lookup(name) {
            Some(c) => ingredients.push((c, n.unwrap_or(1) as u32)),
            None => p.check(false, format!("ingredient `{name}` is not in data/components/")),
        }
    }
    match output {
        Some(output) => {
            p.finish(|| Recipe { id, output, ingredients, consumes_tool: raw.uses_up, difficulty: raw.difficulty })
        }
        None => Err(p.0),
    }
}

static RECIPES: OnceLock<Vec<Recipe>> = OnceLock::new();

fn load() -> Result<Vec<Recipe>, String> {
    let mut recipes = content::load_defs("recipes", build)?;
    recipes.sort_by_cached_key(|r| (r.output.group(), r.difficulty, r.name()));
    Ok(recipes)
}

/// Load recipes now, reporting bad content files as an error.
pub fn init() -> Result<(), String> {
    if RECIPES.get().is_none() {
        let _ = RECIPES.set(load()?);
    }
    Ok(())
}

/// Every recipe, in menu order.
pub fn recipes() -> &'static [Recipe] {
    RECIPES.get_or_init(|| load().unwrap_or_else(|e| panic!("invalid recipe data:\n{e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Recipe, String> {
        let raw: RawRecipe = toml::from_str(text).map_err(|e| e.to_string())?;
        build("t".into(), raw).map_err(|e| e.join("; "))
    }

    #[test]
    fn stock_recipes_load() {
        let all = load().expect("stock recipe data is valid");
        assert!(all[0].output.is_food(), "food sorts first");
        let bat = all.iter().find(|r| r.id == "nail_bat").unwrap();
        assert_eq!(bat.consumes_tool, Some(MeleeWeapon::BaseballBat));
    }

    #[test]
    fn keeps_ingredient_order() {
        let r = parse("makes = { food = \"moonshine\" }\ndifficulty = 1\ningredients = { sugar = 1, grain = 2 }\n")
            .unwrap();
        assert_eq!(r.ingredients, vec![(Component::named("sugar"), 1), (Component::named("grain"), 2)]);
    }

    #[test]
    fn rejects_bad_content() {
        let ok = "makes = { armor = \"scrap_vest\" }\ndifficulty = 9\ningredients = { rags = 1 }\n";
        assert!(parse(ok).is_ok());
        assert!(parse(&ok.replace("scrap_vest", "plate_mail")).is_err());
        assert!(parse(&ok.replace("armor", "food")).unwrap_err().contains("consumables"));
        assert!(parse(&ok.replace("rags = 1", "ragz = 1")).unwrap_err().contains("ragz"));
        assert!(parse(&ok.replace("rags = 1", "rags = 0")).unwrap_err().contains("count"));
        assert!(parse(&ok.replace("{ rags = 1 }", "{}")).unwrap_err().contains("empty"));
    }
}
