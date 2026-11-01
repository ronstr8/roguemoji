//! The bestiary: common animals, mutated creatures, and cryptids.

use crate::effects::{EffectKind, EffectSpec};
use crate::items::Component;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Species {
    Rat,
    Raccoon,
    Rattlesnake,
    FeralDog,
    Coyote,
    Boar,
    BlackBear,
    GiantRoach,
    Bloodflies,
    TwinToad,
    GlowGhoul,
    RadMonitor,
    StingScorpion,
    BlightMass,
    RadBrute,
    Chupacabra,
    DoverDemon,
    Mothman,
    JerseyDevil,
    Bigfoot,
    Wendigo,
}

pub struct SpeciesDef {
    pub name: &'static str,
    pub glyph: &'static str,
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
    pub rarity: u32,
    pub on_hit: OnHit,
    pub drops: &'static [(Component, f64)],
    pub desc: &'static str,
}

const fn poison(p: i32, t: u32) -> OnHit {
    OnHit::Inflict(EffectSpec::new(EffectKind::Poison, p, t))
}

const fn bleed(p: i32, t: u32) -> OnHit {
    OnHit::Inflict(EffectSpec::new(EffectKind::Bleed, p, t))
}

impl Species {
    pub const ALL: [Species; 21] = [
        Species::Rat,
        Species::Raccoon,
        Species::Rattlesnake,
        Species::FeralDog,
        Species::Coyote,
        Species::Boar,
        Species::BlackBear,
        Species::GiantRoach,
        Species::Bloodflies,
        Species::TwinToad,
        Species::GlowGhoul,
        Species::RadMonitor,
        Species::StingScorpion,
        Species::BlightMass,
        Species::RadBrute,
        Species::Chupacabra,
        Species::DoverDemon,
        Species::Mothman,
        Species::JerseyDevil,
        Species::Bigfoot,
        Species::Wendigo,
    ];

    pub fn def(self) -> &'static SpeciesDef {
        use Component::*;
        use Family::*;
        macro_rules! s {
            ($n:expr, $g:expr, $f:expr, hp $hp:expr, atk $atk:expr, def $def:expr, dmg $dmg:expr, arm $arm:expr,
             spd $spd:expr, xtal $x:expr, depth $d:expr, rare $r:expr, $on:expr, [$($drop:expr),*], $desc:expr) => {
{
                    const DEF: SpeciesDef = SpeciesDef {
                        name: $n, glyph: $g, family: $f, hp: $hp, attack: $atk, defense: $def, damage: $dmg,
                        armor: $arm, speed: $spd, crystals: $x, min_depth: $d, rarity: $r, on_hit: $on,
                        drops: &[$($drop),*], desc: $desc,
                    };
                    &DEF
                }
            };
        }
        match self {
            // ── Animals ──
            Species::Rat => s!("Rat", "🐀", Animal, hp 4, atk 1, def 9, dmg (1, 2), arm 0, spd 100, xtal (1, 2),
                depth 1, rare 10, OnHit::None, [(RawMeat, 0.2)], "A fat, bold city rat."),
            Species::Raccoon => {
                s!("Raccoon", "🦝", Animal, hp 6, atk 2, def 11, dmg (1, 3), arm 0, spd 100, xtal (1, 3),
                depth 1, rare 8, OnHit::None, [(RawMeat, 0.4)], "Masked, clever, and very hungry.")
            }
            Species::Rattlesnake => {
                s!("Rattlesnake", "🐍", Animal, hp 5, atk 3, def 11, dmg (1, 3), arm 0, spd 100, xtal (2, 3),
                depth 1, rare 6, poison(1, 8), [(RawMeat, 0.3)], "Coiled in the warm rubble.")
            }
            Species::FeralDog => {
                s!("Feral Dog", "🐕", Animal, hp 9, atk 3, def 10, dmg (2, 4), arm 0, spd 130, xtal (2, 4),
                depth 1, rare 8, OnHit::None, [(RawMeat, 0.6), (Bone, 0.3)], "Somebody's pet, three winters ago.")
            }
            Species::Coyote => {
                s!("Coyote", "🐺", Animal, hp 12, atk 4, def 11, dmg (2, 5), arm 0, spd 130, xtal (3, 5),
                depth 2, rare 7, OnHit::None, [(RawMeat, 0.6), (Bone, 0.4)], "Lean and patient. Hunts in pairs.")
            }
            Species::Boar => {
                s!("Wild Boar", "🐗", Animal, hp 16, atk 4, def 9, dmg (3, 6), arm 1, spd 100, xtal (4, 6),
                depth 2, rare 6, bleed(1, 3), [(RawMeat, 0.9), (Bone, 0.5)], "Tusks like box-cutters.")
            }
            Species::BlackBear => {
                s!("Black Bear", "🐻", Animal, hp 30, atk 5, def 9, dmg (4, 9), arm 1, spd 100, xtal (8, 12),
                depth 4, rare 4, OnHit::None, [(RawMeat, 1.0), (Bone, 0.7)], "Three hundred pounds of bad mood.")
            }
            // ── Mutants ──
            Species::GiantRoach => {
                s!("Hound Roach", "🪳", Mutant, hp 8, atk 3, def 10, dmg (2, 4), arm 2, spd 110, xtal (2, 4),
                depth 1, rare 8, OnHit::None, [(Mutagen, 0.25)], "A cockroach the size of a beagle.")
            }
            Species::Bloodflies => {
                s!("Bloodfly Swarm", "🦟", Mutant, hp 5, atk 4, def 13, dmg (1, 3), arm 0, spd 140, xtal (2, 3),
                depth 2, rare 6, bleed(1, 4), [(Mutagen, 0.2)], "A whining cloud of thumb-sized mosquitoes.")
            }
            Species::TwinToad => {
                s!("Twin-Headed Toad", "🐸", Mutant, hp 10, atk 3, def 9, dmg (2, 4), arm 0, spd 90, xtal (3, 5),
                depth 2, rare 6, poison(2, 6), [(Mutagen, 0.5)], "Both heads want to eat you.")
            }
            Species::GlowGhoul => {
                s!("Glow Ghoul", "🧟", Mutant, hp 18, atk 5, def 9, dmg (3, 6), arm 0, spd 80, xtal (5, 8),
                depth 3, rare 6, poison(2, 8), [(Mutagen, 0.5), (Rags, 0.5)], "Was a person, once. Now it glows.")
            }
            Species::RadMonitor => {
                s!("Rad Monitor", "🦎", Mutant, hp 22, atk 5, def 11, dmg (3, 7), arm 2, spd 100, xtal (6, 9),
                depth 4, rare 5, poison(1, 10), [(RawMeat, 0.7), (Mutagen, 0.5)], "A lizard grown to crocodile size.")
            }
            Species::StingScorpion => {
                s!("Sting Scorpion", "🦂", Mutant, hp 20, atk 6, def 10, dmg (3, 6), arm 3, spd 100, xtal (6, 9),
                depth 4, rare 5, poison(2, 10), [(Mutagen, 0.6)], "Chitin plates like a car hood.")
            }
            Species::BlightMass => {
                s!("Blight Mass", "🦠", Mutant, hp 28, atk 4, def 7, dmg (2, 5), arm 1, spd 60, xtal (7, 10),
                depth 5, rare 4, poison(3, 10), [(Mutagen, 1.0)], "A crawling heap of infection.")
            }
            Species::RadBrute => {
                s!("Rad Brute", "👹", Mutant, hp 40, atk 6, def 9, dmg (5, 10), arm 2, spd 100, xtal (12, 16),
                depth 6, rare 4, OnHit::None, [(Mutagen, 0.8), (ScrapMetal, 0.5)], "A hulking mutant in welded scrap.")
            }
            // ── Cryptids ──
            Species::Chupacabra => {
                s!("Chupacabra", "🐐", Cryptid, hp 18, atk 6, def 12, dmg (3, 6), arm 1, spd 130, xtal (8, 12),
                depth 3, rare 2, bleed(2, 5), [(CryptidHair, 0.6), (Bone, 0.5)], "Spined, hairless, and drinks blood.")
            }
            Species::DoverDemon => {
                s!("Dover Demon", "👺", Cryptid, hp 20, atk 6, def 15, dmg (3, 6), arm 0, spd 120, xtal (9, 13),
                depth 4, rare 2, OnHit::Dread, [(CryptidHair, 0.5)], "Spindly limbs, huge glowing eyes.")
            }
            Species::Mothman => {
                s!("Mothman", "🦋", Cryptid, hp 26, atk 6, def 14, dmg (3, 7), arm 0, spd 130, xtal (12, 16),
                depth 5, rare 2, OnHit::Dread, [(CryptidHair, 0.8)], "Red eyes in the dark. An omen.")
            }
            Species::JerseyDevil => {
                s!("Jersey Devil", "😈", Cryptid, hp 35, atk 7, def 13, dmg (4, 9), arm 1, spd 120, xtal (14, 18),
                depth 6, rare 2, bleed(2, 5), [(CryptidHair, 0.8), (Bone, 0.5)], "Hooves, wings, and a horse's skull.")
            }
            Species::Bigfoot => {
                s!("Bigfoot", "🦍", Cryptid, hp 55, atk 7, def 10, dmg (6, 12), arm 2, spd 100, xtal (18, 24),
                depth 6, rare 2, OnHit::None, [(CryptidHair, 1.0), (RawMeat, 0.7)], "Eight feet of matted fur and muscle.")
            }
            Species::Wendigo => {
                s!("Wendigo", "🦌", Cryptid, hp 45, atk 8, def 12, dmg (5, 10), arm 1, spd 120, xtal (20, 26),
                depth 7, rare 2, OnHit::Dread, [(CryptidHair, 1.0), (Bone, 1.0)], "Antlered, emaciated, ravenous.")
            }
        }
    }
}
