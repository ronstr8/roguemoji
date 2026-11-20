//! Map features: harvestable nodes, Autovend kiosks, Visitor lockboxes,
//! encounters, portals, and the Beacon.

use super::encounter::{Encounter, Mood};
use super::harvest::Harvestable;

/// Whether a hidden node has been found yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cover {
    /// Drawn on the map like anything else.
    Found,
    /// Hidden, and you haven't had your look at it yet.
    Hidden,
    /// Hidden, and you looked and missed it: only walking into it finds it now.
    Overlooked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    Harvest {
        kind: Harvestable,
        uses_left: u8,
        cover: Cover,
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

    /// A harvest node as placed: full, and hidden if its kind hides.
    pub fn harvest(kind: Harvestable) -> Self {
        let cover = if kind.def().hidden.is_some() { Cover::Hidden } else { Cover::Found };
        Feature::Harvest { kind, uses_left: kind.def().uses, cover }
    }

    /// Not yet found, so not drawn and not in the way.
    pub fn is_hidden(&self) -> bool {
        matches!(self, Feature::Harvest { cover: Cover::Hidden | Cover::Overlooked, .. })
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
    /// not walked over. A hidden node doesn't block until it's found.
    pub fn blocks(&self) -> bool {
        match self {
            Feature::Harvest { .. } => !self.is_hidden(),
            Feature::Kiosk(_) | Feature::Lockbox => true,
            Feature::Encounter { kind, .. } => kind.def().stays,
            Feature::Portal | Feature::Beacon => false,
        }
    }
}
