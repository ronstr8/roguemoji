//! Crafting recipes. Every craft is a Crafting check against the recipe's
//! difficulty; the tier decides the outcome:
//!
//! * Fumble — ingredients ruined.
//! * Fail — nothing made, ingredients kept (you just wasted time).
//! * Partial / Success — the item is made.
//! * Critical — a bonus portion (food) or a refund of one ingredient (gear).

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
}

pub struct Recipe {
    pub output: Output,
    /// `(component id, count)`; ids must exist in `data/components/`.
    pub ingredients: &'static [(&'static str, u32)],
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
        self.ingredients.iter().map(|(id, n)| (Component::named(id), *n))
    }
}

macro_rules! recipe {
    ($out:expr, [$(($c:literal, $n:expr)),*], $tool:expr, $diff:expr) => {
        Recipe { output: $out, ingredients: &[$(($c, $n)),*], consumes_tool: $tool, difficulty: $diff }
    };
}

pub static RECIPES: &[Recipe] = &[
    // Food & drink
    recipe!(Output::Food(Consumable::HerbalTea), [("herbs", 1), ("dirty_water", 1)], None, 8),
    recipe!(Output::Food(Consumable::Hardtack), [("grain", 1), ("dirty_water", 1)], None, 8),
    recipe!(Output::Food(Consumable::CactusJuice), [("cactus_pulp", 1), ("sugar", 1)], None, 9),
    recipe!(Output::Food(Consumable::GrilledMeat), [("raw_meat", 1), ("herbs", 1)], None, 9),
    recipe!(Output::Food(Consumable::BeanChili), [("canned_beans", 1), ("herbs", 1)], None, 9),
    recipe!(Output::Food(Consumable::GlowcapBrew), [("glowcap", 1), ("dirty_water", 1)], None, 11),
    recipe!(Output::Food(Consumable::BoneBroth), [("bone", 1), ("dirty_water", 1), ("herbs", 1)], None, 11),
    recipe!(Output::Food(Consumable::Moonshine), [("grain", 2), ("sugar", 1), ("dirty_water", 1)], None, 12),
    recipe!(Output::Food(Consumable::StimSoda), [("sugar", 1), ("mutagen", 1), ("dirty_water", 1)], None, 13),
    recipe!(Output::Food(Consumable::MutantStew), [("raw_meat", 1), ("mutagen", 1), ("dirty_water", 1)], None, 13),
    recipe!(
        Output::Food(Consumable::CryptidTonic),
        [("cryptid_hair", 1), ("glowcap", 1), ("dirty_water", 1)],
        None,
        15
    ),
    // Scrapcraft weapons & armor
    recipe!(Output::Melee(MeleeWeapon::NailBat), [("nails", 2)], Some(MeleeWeapon::BaseballBat), 9),
    recipe!(
        Output::Melee(MeleeWeapon::PipeSpear),
        [("pipe", 1), ("duct_tape", 1)],
        Some(MeleeWeapon::KitchenKnife),
        11
    ),
    recipe!(Output::Melee(MeleeWeapon::ScrapCleaver), [("scrap_metal", 2), ("duct_tape", 1)], None, 12),
    recipe!(Output::Melee(MeleeWeapon::SpikedKnuckles), [("scrap_metal", 1), ("nails", 1)], None, 11),
    recipe!(
        Output::Melee(MeleeWeapon::ShockBaton),
        [("pipe", 1), ("battery", 1), ("wire", 1), ("circuit", 1)],
        None,
        14
    ),
    recipe!(Output::Armor(Armor::ScrapVest), [("scrap_metal", 3), ("rags", 2), ("duct_tape", 1)], None, 13),
    // Carrying gear
    recipe!(Output::Pack(Pack::RagSatchel), [("rags", 3), ("duct_tape", 1)], None, 9),
];
