//! A living monster on the map.

use super::species::{Species, SpeciesDef};
use crate::effects::Effects;
use crate::geometry::Pos;

#[derive(Clone, Debug)]
pub struct Monster {
    pub id: u32,
    pub species: Species,
    pub pos: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub effects: Effects,
    pub energy: i32,
    /// Has noticed the player.
    pub alert: bool,
    /// The player has laid eyes on it.
    pub seen: bool,
}

impl Monster {
    pub fn new(id: u32, species: Species, pos: Pos, depth: u32) -> Self {
        let def = species.def();
        // Monsters deeper than their home depth are a bit tougher.
        let over = depth.saturating_sub(def.min_depth) as i32;
        let max_hp = def.hp + def.hp * over / 6;
        Self { id, species, pos, hp: max_hp, max_hp, effects: Effects::default(), energy: 0, alert: false, seen: false }
    }

    pub fn def(&self) -> &'static SpeciesDef {
        self.species.def()
    }

    pub fn name(&self) -> &'static str {
        self.def().name
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn defense(&self) -> i32 {
        self.def().defense
    }

    pub fn armor(&self) -> i32 {
        self.def().armor + self.effects.total(crate::effects::EffectKind::Fortified)
    }

    pub fn health_word(&self) -> &'static str {
        let pct = self.hp * 100 / self.max_hp.max(1);
        match pct {
            90.. => "unhurt",
            60..=89 => "wounded",
            30..=59 => "bloodied",
            _ => "near death",
        }
    }
}
