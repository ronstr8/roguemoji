//! Map features: harvestable nodes, Autovend kiosks, Visitor lockboxes,
//! encounters, portals, and the Beacon.

use super::encounter::{Encounter, Mood};
use super::harvest::Harvestable;
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
    /// Something happening here: walk in (or bump, if it stays) to engage.
    /// `purse` is the crystals a gambling den has left (0 for anything else).
    Encounter {
        kind: Encounter,
        purse: u32,
    },
    /// A tear in spacetime that drops you into the next, stranger world.
    Portal,
    Beacon,
}

impl Feature {
    /// An encounter as placed, with a den's house purse full.
    pub fn encounter(kind: Encounter) -> Self {
        let purse = kind.def().wager.as_ref().and_then(|w| w.purse).unwrap_or(0);
        Feature::Encounter { kind, purse }
    }

    pub fn harvest(kind: Harvestable) -> Self {
        Feature::Harvest { kind, uses_left: kind.def().uses }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Feature::Harvest { kind, .. } => kind.def().glyph.as_str(),
            Feature::Kiosk(_) => "🏧",
            Feature::Lockbox => "🛅",
            Feature::Encounter { kind, .. } => kind.def().glyph.as_str(),
            Feature::Portal => "🌀",
            Feature::Beacon => "📡",
        }
    }

    /// The mood of an encounter; `None` for every other feature.
    pub fn mood(&self) -> Option<Mood> {
        match self {
            Feature::Encounter { kind, .. } => Some(kind.def().mood()),
            _ => None,
        }
    }

    /// Harvest nodes, kiosks, lockboxes and encounters that stay are bumped,
    /// not walked over.
    pub fn blocks(&self) -> bool {
        match self {
            Feature::Harvest { .. } | Feature::Kiosk(_) | Feature::Lockbox => true,
            Feature::Encounter { kind, .. } => kind.def().stays,
            Feature::Portal | Feature::Beacon => false,
        }
    }
}
