//! Wearable protection. Armor subtracts from incoming damage, reduced by the
//! attack's penetration; heavier pieces hamper dodging.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Armor {
    LeatherJacket,
    ScrapVest,
    KevlarVest,
    RiotGear,
    VisitorWeave,
}

pub struct ArmorDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub armor: i32,
    pub dodge_penalty: i32,
    pub desc: &'static str,
}

impl Armor {
    pub fn def(self) -> &'static ArmorDef {
        macro_rules! a {
            ($n:expr, $g:expr, $ar:expr, $dp:expr, $d:expr) => {
                &ArmorDef { name: $n, glyph: $g, armor: $ar, dodge_penalty: $dp, desc: $d }
            };
        }
        match self {
            Armor::LeatherJacket => a!("Leather Jacket", "🧥", 1, 0, "Scuffed motorcycle leather."),
            Armor::ScrapVest => a!("Scrap Plate Vest", "🦺", 2, 1, "Car-door panels strapped over padding."),
            Armor::KevlarVest => {
                a!("Kevlar Vest", "🦺", 2, 0, "Soft body armor. Stops teeth almost as well as bullets.")
            }
            Armor::RiotGear => a!("Riot Gear", "🪖", 4, 2, "Full police riot kit. Heavy and hot."),
            Armor::VisitorWeave => {
                a!("Visitor Weave", "💠", 3, 0, "A shimmering mesh that hardens at the moment of impact.")
            }
        }
    }
}
