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
        let gear = self.starting_gear();
        p.melee = gear.wield.map(MeleeWeapon::named);
        p.armor = gear.armor.map(Armor::named);
        if let Some((gun, ammo, rounds)) = gear.gun {
            p.gun = Some(Gun::loaded_with(GunModel::named(gun), Ammo::named(ammo), rounds));
        }
        for (id, n) in gear.ammo {
            give(&mut p, Item::ammo(Ammo::named(id), *n));
        }
        for id in gear.spare {
            give(&mut p, Item::one(ItemKind::Melee(MeleeWeapon::named(id))));
        }
        for (id, n) in self.starting_components() {
            give(&mut p, Item::component(Component::named(id), *n));
        }
        for (id, n) in self.starting_food() {
            give(&mut p, Item::food(Consumable::named(id), *n));
        }
        p.crystals = 5;
        p
    }

    /// Weapons, ammo and armor in the starting kit, by id.
    pub fn starting_gear(self) -> StartingGear {
        match self {
            Background::Scavenger => StartingGear {
                gun: Some(("target_pistol_22", "22lr", 10)),
                ammo: &[("22lr", 20)],
                wield: Some("crowbar"),
                spare: &[],
                armor: None,
            },
            Background::ExDeputy => StartingGear {
                gun: Some(("service_pistol_9mm", "9mm_jhp", 15)),
                ammo: &[("9mm_fmj", 15)],
                wield: None,
                spare: &[],
                armor: Some("kevlar_vest"),
            },
            Background::Mechanic => StartingGear {
                gun: Some(("snub_revolver_38", "38_special", 5)),
                ammo: &[("38_special", 10)],
                wield: Some("pipe_wrench"),
                spare: &["baseball_bat"],
                armor: None,
            },
            Background::Homesteader => StartingGear {
                gun: Some(("single_shot_20ga", "20ga_buck", 1)),
                ammo: &[("20ga_buck", 8)],
                wield: Some("kitchen_knife"),
                spare: &[],
                armor: Some("leather_jacket"),
            },
        }
    }

    /// Food and drink in the starting kit, by id.
    pub fn starting_food(self) -> &'static [(&'static str, u32)] {
        match self {
            Background::Scavenger => &[("canned_peaches", 1)],
            Background::ExDeputy => &[("bottled_water", 1)],
            Background::Mechanic => &[],
            Background::Homesteader => &[("hardtack", 1)],
        }
    }

    /// Crafting components in the starting kit, by id.
    pub fn starting_components(self) -> &'static [(&'static str, u32)] {
        match self {
            Background::Scavenger => &[("duct_tape", 1)],
            Background::ExDeputy => &[],
            Background::Mechanic => &[("nails", 2), ("scrap_metal", 2), ("duct_tape", 1)],
            Background::Homesteader => &[("herbs", 2), ("dirty_water", 2)],
        }
    }
}

/// Gear a background starts with, by content id.
#[derive(Clone, Copy)]
pub struct StartingGear {
    /// `(gun, ammo loaded, rounds loaded)`.
    pub gun: Option<(&'static str, &'static str, u32)>,
    /// Spare ammunition: `(ammo, rounds)`.
    pub ammo: &'static [(&'static str, u32)],
    /// The melee weapon in hand.
    pub wield: Option<&'static str>,
    /// Extra melee weapons in the pack.
    pub spare: &'static [&'static str],
    pub armor: Option<&'static str>,
}

impl StartingGear {
    pub fn melee_ids(self) -> impl Iterator<Item = &'static str> {
        self.wield.into_iter().chain(self.spare.iter().copied())
    }

    pub fn ammo_ids(self) -> impl Iterator<Item = &'static str> {
        self.gun.map(|(_, ammo, _)| ammo).into_iter().chain(self.ammo.iter().map(|(id, _)| *id))
    }
}
