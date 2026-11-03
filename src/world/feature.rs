//! Map features: harvestable nodes, Autovend kiosks, Visitor lockboxes,
//! stairs, and the Beacon.

use crate::stats::Skill;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Harvestable {
    WildHerbs,
    GlowcapPatch,
    Cactus,
    WildGrain,
    WaterPump,
    CarWreck,
    Toolbox,
    DeadElectronics,
    SupplyCrate,
    Remains,
}

pub struct HarvestDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub skill: Skill,
    pub difficulty: i32,
    /// `(component id, min, max)` — each entry rolls independently.
    pub yields: &'static [(&'static str, u32, u32)],
    pub uses: u8,
    pub verb: &'static str,
}

impl Harvestable {
    pub const NATURAL: [Harvestable; 5] = [
        Harvestable::WildHerbs,
        Harvestable::GlowcapPatch,
        Harvestable::Cactus,
        Harvestable::WildGrain,
        Harvestable::WaterPump,
    ];
    pub const URBAN: [Harvestable; 5] = [
        Harvestable::CarWreck,
        Harvestable::Toolbox,
        Harvestable::DeadElectronics,
        Harvestable::SupplyCrate,
        Harvestable::Remains,
    ];

    pub const ALL: [Harvestable; 10] = [
        Harvestable::WildHerbs,
        Harvestable::GlowcapPatch,
        Harvestable::Cactus,
        Harvestable::WildGrain,
        Harvestable::WaterPump,
        Harvestable::CarWreck,
        Harvestable::Toolbox,
        Harvestable::DeadElectronics,
        Harvestable::SupplyCrate,
        Harvestable::Remains,
    ];

    pub fn def(self) -> &'static HarvestDef {
        use Skill::{Scavenging, Survival};
        macro_rules! h {
            ($n:expr, $g:expr, $sk:expr, $diff:expr, $uses:expr, $verb:expr, [$(($c:expr, $lo:expr, $hi:expr)),*]) => {
                &HarvestDef { name: $n, glyph: $g, skill: $sk, difficulty: $diff, uses: $uses, verb: $verb,
                    yields: &[$(($c, $lo, $hi)),*] }
            };
        }
        match self {
            Harvestable::WildHerbs => h!("Wild Herbs", "🌿", Survival, 9, 2, "forage", [("herbs", 1, 2)]),
            Harvestable::GlowcapPatch => h!("Glowcap Patch", "🍄", Survival, 11, 2, "pick", [("glowcap", 1, 2)]),
            Harvestable::Cactus => h!("Barrel Cactus", "🌵", Survival, 10, 2, "carve", [("cactus_pulp", 1, 2)]),
            Harvestable::WildGrain => h!("Wild Grain", "🌾", Survival, 9, 2, "thresh", [("grain", 1, 3)]),
            Harvestable::WaterPump => h!("Hand Pump", "🚰", Survival, 8, 3, "pump", [("dirty_water", 1, 2)]),
            Harvestable::CarWreck => h!(
                "Car Wreck",
                "🚗",
                Scavenging,
                11,
                3,
                "strip",
                [("scrap_metal", 1, 2), ("wire", 0, 1), ("battery", 0, 1), ("pipe", 0, 1)]
            ),
            Harvestable::Toolbox => h!(
                "Toolbox",
                "🧰",
                Scavenging,
                9,
                1,
                "rummage through",
                [("nails", 1, 3), ("duct_tape", 0, 2), ("wire", 0, 1)]
            ),
            Harvestable::DeadElectronics => {
                h!(
                    "Dead Electronics",
                    "📺",
                    Scavenging,
                    12,
                    2,
                    "gut",
                    [("circuit", 0, 1), ("wire", 1, 2), ("battery", 0, 1)]
                )
            }
            Harvestable::SupplyCrate => h!(
                "Supply Crate",
                "📦",
                Scavenging,
                10,
                1,
                "pry open",
                [("canned_beans", 0, 2), ("sugar", 0, 2), ("rags", 0, 2), ("duct_tape", 0, 1)]
            ),
            Harvestable::Remains => {
                h!("Old Remains", "💀", Scavenging, 8, 1, "search", [("bone", 1, 2), ("rags", 0, 2)])
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    Harvest {
        kind: Harvestable,
        uses_left: u8,
    },
    /// Index into the level's kiosk list.
    Kiosk(usize),
    /// A Visitor lockbox. Every lockbox opens onto the same shared stash.
    Lockbox,
    Stairs,
    Beacon,
}

impl Feature {
    pub fn harvest(kind: Harvestable) -> Self {
        Feature::Harvest { kind, uses_left: kind.def().uses }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Feature::Harvest { kind, .. } => kind.def().glyph,
            Feature::Kiosk(_) => "🏧",
            Feature::Lockbox => "🛅",
            Feature::Stairs => "🔽",
            Feature::Beacon => "📡",
        }
    }

    /// Harvest nodes, kiosks and lockboxes are bumped, not walked over.
    pub fn blocks(&self) -> bool {
        matches!(self, Feature::Harvest { .. } | Feature::Kiosk(_) | Feature::Lockbox)
    }
}
