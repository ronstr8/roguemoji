//! The player character: stats, equipment, pack and derived values.

use crate::combat::{Momentum, Stance};
use crate::effects::{EffectKind, Effects};
use crate::geometry::Pos;
use crate::items::melee::{FISTS, MeleeDef};
use crate::items::{Armor, Gun, Inventory, MeleeWeapon};
use crate::stats::attributes::{self, MAX_SCORE, MIN_SCORE};
use crate::stats::skills::{self, SkillBreakdown};
use crate::stats::{Attribute, Attributes, Skill, SkillSet};

pub const MAX_SATIETY: i32 = 1000;
pub const HUNGRY_AT: i32 = 200;

#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub background: &'static str,
    pub pos: Pos,
    pub hp: i32,
    pub base: Attributes,
    pub skills: SkillSet,
    pub effects: Effects,
    pub inventory: Inventory,
    pub melee: Option<MeleeWeapon>,
    pub gun: Option<Gun>,
    pub armor: Option<Armor>,
    pub crystals: u32,
    pub satiety: i32,
    pub stance: Stance,
    pub momentum: Momentum,
    pub kills: u32,
}

impl Player {
    pub fn new(name: String, background: &'static str, base: Attributes, skills: SkillSet) -> Self {
        let mut p = Self {
            name,
            background,
            pos: Pos::default(),
            hp: 1,
            base,
            skills,
            effects: Effects::default(),
            inventory: Inventory::default(),
            melee: None,
            gun: None,
            armor: None,
            crystals: 0,
            satiety: MAX_SATIETY * 3 / 4,
            stance: Stance::Balanced,
            momentum: Momentum::default(),
            kills: 0,
        };
        p.hp = p.max_hp();
        p
    }

    /// Effective attribute score including temporary effects.
    pub fn attr(&self, a: Attribute) -> i32 {
        let hungry_penalty = if self.satiety <= 0 && a.is_physical() { 1 } else { 0 };
        (self.base.get(a) + self.effects.attr_mod(a) - hungry_penalty).clamp(MIN_SCORE, MAX_SCORE)
    }

    pub fn attr_bonus(&self, a: Attribute) -> i32 {
        attributes::bonus(self.attr(a))
    }

    pub fn skill(&self, s: Skill) -> SkillBreakdown {
        skills::breakdown(s, self.skills.rank(s), |a| self.attr(a))
    }

    pub fn max_hp(&self) -> i32 {
        14 + self.attr(Attribute::Fortitude) * 3
    }

    pub fn melee_def(&self) -> &'static MeleeDef {
        self.melee.map(|m| m.def()).unwrap_or(&FISTS)
    }

    pub fn armor_value(&self) -> i32 {
        self.armor.map_or(0, |a| a.def().armor) + self.effects.total(EffectKind::Fortified)
    }

    pub fn defense(&self) -> i32 {
        let penalty = self.armor.map_or(0, |a| a.def().dodge_penalty);
        10 + self.skill(Skill::Dodge).total() + self.stance.defense_mod() - penalty
    }

    /// Flat bonus to every to-hit roll from effects.
    pub fn focus(&self) -> i32 {
        self.effects.total(EffectKind::Focus)
    }

    /// Melee damage bonus from Brawn (penalties capped at -2).
    pub fn brawn_damage(&self) -> i32 {
        self.attr_bonus(Attribute::Brawn).max(-2)
    }

    /// HP restored per absorbed crystal; Willpower attunes you to them.
    pub fn crystal_heal(&self) -> i32 {
        (3 + self.attr_bonus(Attribute::Willpower)).max(1)
    }

    pub fn heal(&mut self, n: i32) -> i32 {
        let before = self.hp;
        self.hp = (self.hp + n).min(self.max_hp());
        self.hp - before
    }

    pub fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn hunger_label(&self) -> &'static str {
        match self.satiety {
            s if s <= 0 => "Starving",
            s if s <= HUNGRY_AT => "Hungry",
            s if s >= MAX_SATIETY * 9 / 10 => "Stuffed",
            _ => "Fed",
        }
    }
}
