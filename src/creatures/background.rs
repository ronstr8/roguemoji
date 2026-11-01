//! Starting backgrounds: who you were before the world ended.

use super::player::Player;
use crate::items::{Ammo, Armor, Component, Consumable, Gun, GunModel, Item, ItemKind, MeleeWeapon};
use crate::stats::{Attributes, Skill, SkillSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Background {
    Scavenger,
    ExDeputy,
    Mechanic,
    Homesteader,
}

impl Background {
    pub const ALL: [Background; 4] =
        [Background::Scavenger, Background::ExDeputy, Background::Mechanic, Background::Homesteader];

    pub fn name(self) -> &'static str {
        match self {
            Background::Scavenger => "Scavenger",
            Background::ExDeputy => "Ex-Deputy",
            Background::Mechanic => "Mechanic",
            Background::Homesteader => "Homesteader",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Background::Scavenger => "🎒",
            Background::ExDeputy => "⭐",
            Background::Mechanic => "🔧",
            Background::Homesteader => "🌽",
        }
    }

    pub fn pitch(self) -> &'static str {
        match self {
            Background::Scavenger => "Quick hands, quicker eyes. Strips any wreck in seconds.",
            Background::ExDeputy => "Badge long gone, training intact. Steady with a sidearm.",
            Background::Mechanic => "Builds weapons out of anything. Swings a mean wrench.",
            Background::Homesteader => "Hunted and foraged before it was cool. Tough as boot leather.",
        }
    }

    /// Attributes in order: Brawn, Brains, Reflexes, Wit, Fortitude, Willpower.
    fn attributes(self) -> Attributes {
        match self {
            Background::Scavenger => Attributes::new(4, 5, 7, 7, 5, 5),
            Background::ExDeputy => Attributes::new(6, 5, 7, 5, 5, 6),
            Background::Mechanic => Attributes::new(6, 7, 5, 6, 5, 4),
            Background::Homesteader => Attributes::new(6, 4, 5, 5, 7, 6),
        }
    }

    fn skills(self) -> SkillSet {
        let mut s = SkillSet::default();
        let ranks: &[(Skill, u8)] = match self {
            Background::Scavenger => {
                &[(Skill::Scavenging, 2), (Skill::Dodge, 1), (Skill::SmallArms, 1), (Skill::Melee, 1)]
            }
            Background::ExDeputy => {
                &[(Skill::SmallArms, 2), (Skill::Brawling, 1), (Skill::LongArms, 1), (Skill::Dodge, 1)]
            }
            Background::Mechanic => &[(Skill::Crafting, 2), (Skill::Melee, 2), (Skill::Scavenging, 1)],
            Background::Homesteader => &[(Skill::LongArms, 2), (Skill::Survival, 2), (Skill::Crafting, 1)],
        };
        for (skill, rank) in ranks {
            s.set_rank(*skill, *rank);
        }
        s
    }

    pub fn create(self, name: String) -> Player {
        let mut p = Player::new(name, self.name(), self.attributes(), self.skills());
        let give = |p: &mut Player, item: Item| {
            let _ = p.inventory.add(item);
        };
        match self {
            Background::Scavenger => {
                p.melee = Some(MeleeWeapon::Crowbar);
                p.gun = Some(Gun::loaded_with(GunModel::TargetPistol22, Ammo::Lr22, 10));
                give(&mut p, Item::ammo(Ammo::Lr22, 20));
                give(&mut p, Item::component(Component::DuctTape, 1));
                give(&mut p, Item::food(Consumable::CannedPeaches, 1));
            }
            Background::ExDeputy => {
                p.gun = Some(Gun::loaded_with(GunModel::ServicePistol9, Ammo::Luger9Jhp, 15));
                p.armor = Some(Armor::KevlarVest);
                give(&mut p, Item::ammo(Ammo::Luger9Fmj, 15));
                give(&mut p, Item::food(Consumable::BottledWater, 1));
            }
            Background::Mechanic => {
                p.melee = Some(MeleeWeapon::PipeWrench);
                p.gun = Some(Gun::loaded_with(GunModel::SnubRevolver38, Ammo::Spl38, 5));
                give(&mut p, Item::ammo(Ammo::Spl38, 10));
                give(&mut p, Item::one(ItemKind::Melee(MeleeWeapon::BaseballBat)));
                give(&mut p, Item::component(Component::Nails, 2));
                give(&mut p, Item::component(Component::ScrapMetal, 2));
                give(&mut p, Item::component(Component::DuctTape, 1));
            }
            Background::Homesteader => {
                p.melee = Some(MeleeWeapon::KitchenKnife);
                p.gun = Some(Gun::loaded_with(GunModel::SingleShot20, Ammo::Ga20Buck, 1));
                p.armor = Some(Armor::LeatherJacket);
                give(&mut p, Item::ammo(Ammo::Ga20Buck, 8));
                give(&mut p, Item::component(Component::Herbs, 2));
                give(&mut p, Item::component(Component::DirtyWater, 2));
                give(&mut p, Item::food(Consumable::Hardtack, 1));
            }
        }
        p.crystals = 5;
        p
    }
}
