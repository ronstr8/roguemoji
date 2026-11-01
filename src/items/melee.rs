//! Melee weapons: scavenged tools, scrap-crafted improvisations, and rare
//! artifacts left behind by the visitors.

use crate::stats::Skill;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Special {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeleeWeapon {
    KitchenKnife,
    ClawHammer,
    Crowbar,
    PipeWrench,
    BaseballBat,
    Machete,
    Shovel,
    FireAxe,
    BrassKnuckles,
    NailBat,
    PipeSpear,
    ScrapCleaver,
    ShockBaton,
    SpikedKnuckles,
    PhaseEdge,
    GravitonMaul,
    HumFilament,
    SiphonGauntlet,
}

pub struct MeleeDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub origin: Origin,
    pub skill: Skill,
    pub damage: (i32, i32),
    pub accuracy: i32,
    pub pen: i32,
    pub special: Special,
    pub desc: &'static str,
}

/// Bare hands, used when nothing is equipped.
pub const FISTS: MeleeDef = MeleeDef {
    name: "Fists",
    glyph: "👊",
    origin: Origin::Tool,
    skill: Skill::Brawling,
    damage: (1, 3),
    accuracy: 1,
    pen: 0,
    special: Special::None,
    desc: "Your own two hands.",
};

impl MeleeWeapon {
    pub const TOOLS: [MeleeWeapon; 9] = [
        MeleeWeapon::KitchenKnife,
        MeleeWeapon::ClawHammer,
        MeleeWeapon::Crowbar,
        MeleeWeapon::PipeWrench,
        MeleeWeapon::BaseballBat,
        MeleeWeapon::Machete,
        MeleeWeapon::Shovel,
        MeleeWeapon::FireAxe,
        MeleeWeapon::BrassKnuckles,
    ];

    pub const ALIEN: [MeleeWeapon; 4] =
        [MeleeWeapon::PhaseEdge, MeleeWeapon::GravitonMaul, MeleeWeapon::HumFilament, MeleeWeapon::SiphonGauntlet];

    pub fn def(self) -> &'static MeleeDef {
        use Origin::*;
        use Skill::{Brawling, Melee};
        use Special as S;
        macro_rules! w {
            ($name:expr, $g:expr, $o:expr, $sk:expr, $d:expr, $acc:expr, $pen:expr, $sp:expr, $desc:expr) => {
                &MeleeDef {
                    name: $name,
                    glyph: $g,
                    origin: $o,
                    skill: $sk,
                    damage: $d,
                    accuracy: $acc,
                    pen: $pen,
                    special: $sp,
                    desc: $desc,
                }
            };
        }
        match self {
            MeleeWeapon::KitchenKnife => w!(
                "Kitchen Knife",
                "🔪",
                Tool,
                Melee,
                (1, 4),
                2,
                0,
                S::Bleed,
                "An eight-inch chef's knife. Quick and nasty."
            ),
            MeleeWeapon::ClawHammer => {
                w!("Claw Hammer", "🔨", Tool, Melee, (2, 4), 1, 1, S::None, "Sixteen ounces of drop-forged steel.")
            }
            MeleeWeapon::Crowbar => {
                w!("Crowbar", "🦯", Tool, Melee, (2, 5), 0, 1, S::None, "Opens doors, crates and skulls.")
            }
            MeleeWeapon::PipeWrench => {
                w!("Pipe Wrench", "🔧", Tool, Melee, (2, 6), -1, 1, S::Stun, "A plumber's heavy friend.")
            }
            MeleeWeapon::BaseballBat => {
                w!("Baseball Bat", "🏏", Tool, Melee, (2, 5), 1, 0, S::Stun, "Ash wood, taped grip.")
            }
            MeleeWeapon::Machete => {
                w!("Machete", "🔪", Tool, Melee, (2, 6), 1, 0, S::Bleed, "Brush-clearing blade, eighteen inches.")
            }
            MeleeWeapon::Shovel => {
                w!("Shovel", "🧹", Tool, Melee, (2, 6), -1, 0, S::Knockback, "Long reach and a flat steel face.")
            }
            MeleeWeapon::FireAxe => {
                w!("Fire Axe", "🪓", Tool, Melee, (3, 8), -2, 2, S::Bleed, "Pick-head axe from a station wall.")
            }
            MeleeWeapon::BrassKnuckles => {
                w!("Brass Knuckles", "💍", Tool, Brawling, (2, 4), 1, 0, S::None, "Makes a fist into a hammer.")
            }
            MeleeWeapon::NailBat => w!(
                "Nail Bat",
                "🏏",
                Crafted,
                Melee,
                (3, 6),
                0,
                0,
                S::Bleed,
                "A bat studded with a box of framing nails."
            ),
            MeleeWeapon::PipeSpear => w!(
                "Pipe Spear",
                "🔱",
                Crafted,
                Melee,
                (3, 7),
                2,
                1,
                S::None,
                "A kitchen knife lashed to a length of pipe. Reach and point."
            ),
            MeleeWeapon::ScrapCleaver => w!(
                "Scrap Cleaver",
                "🔪",
                Crafted,
                Melee,
                (3, 8),
                -1,
                1,
                S::Bleed,
                "A car-hood blade with a taped handle."
            ),
            MeleeWeapon::ShockBaton => w!(
                "Shock Baton",
                "⚡",
                Crafted,
                Melee,
                (2, 5),
                1,
                0,
                S::Stun,
                "Pipe, battery, exposed wire. Bad idea, good results."
            ),
            MeleeWeapon::SpikedKnuckles => w!(
                "Spiked Knuckles",
                "💍",
                Crafted,
                Brawling,
                (3, 5),
                1,
                1,
                S::Bleed,
                "Scrap knuckles with nails ground to points."
            ),
            MeleeWeapon::PhaseEdge => w!(
                "Phase Edge",
                "💠",
                Alien,
                Melee,
                (5, 10),
                2,
                0,
                S::IgnoreArmor,
                "A blade that is only mostly here. It slides through armor like fog."
            ),
            MeleeWeapon::GravitonMaul => w!(
                "Graviton Maul",
                "💠",
                Alien,
                Melee,
                (6, 12),
                0,
                2,
                S::Knockback,
                "Weightless in your hands, mountainous on impact."
            ),
            MeleeWeapon::HumFilament => w!(
                "Hum Filament",
                "💠",
                Alien,
                Melee,
                (4, 8),
                3,
                1,
                S::Bleed,
                "A singing wire that stiffens into an edge when swung."
            ),
            MeleeWeapon::SiphonGauntlet => w!(
                "Siphon Gauntlet",
                "💠",
                Alien,
                Brawling,
                (4, 8),
                2,
                1,
                S::Drain,
                "A glove of warm glass. Wounds you deal flow back into you."
            ),
        }
    }
}
