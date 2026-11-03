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
    pub ingredients: &'static [(Component, u32)],
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
}

macro_rules! recipe {
    ($out:expr, [$(($c:ident, $n:expr)),*], $tool:expr, $diff:expr) => {
        Recipe { output: $out, ingredients: &[$((Component::$c, $n)),*], consumes_tool: $tool, difficulty: $diff }
    };
}

pub static RECIPES: &[Recipe] = &[
    // Food & drink
    recipe!(Output::Food(Consumable::HerbalTea), [(Herbs, 1), (DirtyWater, 1)], None, 8),
    recipe!(Output::Food(Consumable::Hardtack), [(Grain, 1), (DirtyWater, 1)], None, 8),
    recipe!(Output::Food(Consumable::CactusJuice), [(CactusPulp, 1), (Sugar, 1)], None, 9),
    recipe!(Output::Food(Consumable::GrilledMeat), [(RawMeat, 1), (Herbs, 1)], None, 9),
    recipe!(Output::Food(Consumable::BeanChili), [(CannedBeans, 1), (Herbs, 1)], None, 9),
    recipe!(Output::Food(Consumable::GlowcapBrew), [(Glowcap, 1), (DirtyWater, 1)], None, 11),
    recipe!(Output::Food(Consumable::BoneBroth), [(Bone, 1), (DirtyWater, 1), (Herbs, 1)], None, 11),
    recipe!(Output::Food(Consumable::Moonshine), [(Grain, 2), (Sugar, 1), (DirtyWater, 1)], None, 12),
    recipe!(Output::Food(Consumable::StimSoda), [(Sugar, 1), (Mutagen, 1), (DirtyWater, 1)], None, 13),
    recipe!(Output::Food(Consumable::MutantStew), [(RawMeat, 1), (Mutagen, 1), (DirtyWater, 1)], None, 13),
    recipe!(Output::Food(Consumable::CryptidTonic), [(CryptidHair, 1), (Glowcap, 1), (DirtyWater, 1)], None, 15),
    // Scrapcraft weapons & armor
    recipe!(Output::Melee(MeleeWeapon::NailBat), [(Nails, 2)], Some(MeleeWeapon::BaseballBat), 9),
    recipe!(Output::Melee(MeleeWeapon::PipeSpear), [(Pipe, 1), (DuctTape, 1)], Some(MeleeWeapon::KitchenKnife), 11),
    recipe!(Output::Melee(MeleeWeapon::ScrapCleaver), [(ScrapMetal, 2), (DuctTape, 1)], None, 12),
    recipe!(Output::Melee(MeleeWeapon::SpikedKnuckles), [(ScrapMetal, 1), (Nails, 1)], None, 11),
    recipe!(Output::Melee(MeleeWeapon::ShockBaton), [(Pipe, 1), (Battery, 1), (Wire, 1), (Circuit, 1)], None, 14),
    recipe!(Output::Armor(Armor::ScrapVest), [(ScrapMetal, 3), (Rags, 2), (DuctTape, 1)], None, 13),
    // Carrying gear
    recipe!(Output::Pack(Pack::RagSatchel), [(Rags, 3), (DuctTape, 1)], None, 9),
];
