//! Food and drink: crafted buffs and scavenged provisions.

use crate::effects::{EffectKind, EffectSpec};
use crate::stats::Attribute;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Consumable {
    // Crafted
    HerbalTea,
    GlowcapBrew,
    CactusJuice,
    GrilledMeat,
    Hardtack,
    BeanChili,
    BoneBroth,
    MutantStew,
    Moonshine,
    StimSoda,
    CryptidTonic,
    // Found
    BottledWater,
    CannedPeaches,
    Painkillers,
}

pub struct ConsumableDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub desc: &'static str,
    /// Hunger restored.
    pub satiety: i32,
    /// Immediate healing.
    pub heal: i32,
    /// Removes poison and bleeding.
    pub cures: bool,
    pub effects: &'static [EffectSpec],
}

const fn boost(a: Attribute, potency: i32, turns: u32) -> EffectSpec {
    EffectSpec::new(EffectKind::Boost(a), potency, turns)
}

impl Consumable {
    pub fn def(self) -> &'static ConsumableDef {
        use Attribute::*;
        use EffectKind::*;
        macro_rules! c {
            ($n:expr, $g:expr, $sat:expr, $heal:expr, $cure:expr, [$($e:expr),*], $d:expr) => {
{
                    const DEF: ConsumableDef = ConsumableDef {
                        name: $n, glyph: $g, satiety: $sat, heal: $heal, cures: $cure, effects: &[$($e),*], desc: $d,
                    };
                    &DEF
                }
            };
        }
        match self {
            Consumable::HerbalTea => c!(
                "Herbal Tea",
                "🍵",
                60,
                2,
                true,
                [EffectSpec::new(Regen, 1, 20)],
                "Calming and clean. Purges toxins and knits small wounds."
            ),
            Consumable::GlowcapBrew => c!(
                "Glowcap Brew",
                "🧉",
                40,
                0,
                false,
                [boost(Brains, 2, 120), boost(Wit, 2, 120)],
                "Shimmering mushroom tea. Thoughts come fast and bright."
            ),
            Consumable::CactusJuice => c!(
                "Cactus Juice",
                "🧃",
                80,
                0,
                false,
                [boost(Reflexes, 2, 120)],
                "Sweetened pulp. Twitchy-quick hydration."
            ),
            Consumable::GrilledMeat => c!(
                "Grilled Meat",
                "🍖",
                350,
                3,
                false,
                [boost(Brawn, 2, 150)],
                "Seared over scrap-wood coals with herbs. Protein for punching."
            ),
            Consumable::Hardtack => {
                c!("Hardtack", "🍘", 300, 0, false, [], "Flour and water baked into a brick. Lasts forever.")
            }
            Consumable::BeanChili => c!(
                "Bean Chili",
                "🥘",
                320,
                2,
                false,
                [boost(Willpower, 2, 150)],
                "Spiced beans. Warms the belly and stiffens the spine."
            ),
            Consumable::BoneBroth => c!(
                "Bone Broth",
                "🍜",
                200,
                4,
                false,
                [boost(Fortitude, 1, 150), EffectSpec::new(Regen, 1, 16)],
                "Long-simmered marrow broth."
            ),
            Consumable::MutantStew => c!(
                "Mutant Stew",
                "🍲",
                300,
                0,
                false,
                [boost(Fortitude, 3, 150), EffectSpec::new(Fortified, 1, 150)],
                "It glows a little. Your skin toughens like hide."
            ),
            Consumable::Moonshine => c!(
                "Moonshine",
                "🍶",
                20,
                0,
                false,
                [boost(Willpower, 3, 100), boost(Reflexes, -1, 100)],
                "Clear grain liquor. Liquid courage, wobbly hands."
            ),
            Consumable::StimSoda => c!(
                "Stim Soda",
                "🥤",
                30,
                0,
                false,
                [boost(Reflexes, 2, 80), EffectSpec::new(Focus, 2, 80)],
                "Sugar syrup spiked with mutagen extract. Everything slows down."
            ),
            Consumable::CryptidTonic => c!(
                "Cryptid Tonic",
                "🫖",
                50,
                10,
                true,
                [
                    boost(Brawn, 2, 200),
                    boost(Reflexes, 2, 200),
                    boost(Fortitude, 2, 200),
                    EffectSpec::new(Regen, 2, 30)
                ],
                "Steeped from the hair of things that shouldn't exist. Primal vigor."
            ),
            Consumable::BottledWater => {
                c!("Bottled Water", "🍼", 60, 1, false, [], "Sealed, clean water. A rare luxury.")
            }
            Consumable::CannedPeaches => c!("Canned Peaches", "🍑", 220, 2, false, [], "Syrupy and perfect."),
            Consumable::Painkillers => c!(
                "Painkillers",
                "💊",
                0,
                6,
                false,
                [EffectSpec::new(Focus, -1, 20)],
                "Expired opioid tablets. Dulls pain and wits alike."
            ),
        }
    }
}
