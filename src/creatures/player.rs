//! The player character: stats, equipment, pack and derived values.

use crate::combat::{Momentum, Stance};
use crate::effects::{EffectKind, Effects};
use crate::geometry::Pos;
use crate::items::melee::{FISTS, MeleeDef};
use crate::items::{Armor, Gun, Inventory, MeleeWeapon, Pack, weight};
use crate::stats::attributes::{self, MAX_SCORE, MIN_SCORE};
use crate::stats::skills::{self, SkillBreakdown};
use crate::stats::{Attribute, Attributes, Skill, SkillSet};

pub const MAX_SATIETY: i32 = 1000;
pub const HUNGRY_AT: i32 = 200;

/// Carry capacity at average Brawn, in hundredths of a pound.
pub const BASE_CAPACITY: u32 = 45_00;
/// Capacity gained (or lost) per point of Brawn above (or below) average.
pub const CAPACITY_PER_BRAWN: i32 = 6_00;
/// No one is so weak they can't carry a little.
pub const MIN_CAPACITY: u32 = 15_00;

/// How weighed down the player is, by carried weight vs. capacity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Burden {
    /// At or under capacity.
    Light,
    /// Up to 1.5× capacity: moving takes 1.5 turns, Defense -1.
    Burdened,
    /// Up to 2× capacity: moving takes 2 turns, Defense -3.
    Strained,
    /// Over 2× capacity: you can't move at all.
    Overloaded,
}

impl Burden {
    pub fn of(carried: u32, capacity: u32) -> Self {
        let cap = capacity.max(1) as u64;
        match carried as u64 * 2 {
            w if w <= cap * 2 => Burden::Light,
            w if w <= cap * 3 => Burden::Burdened,
            w if w <= cap * 4 => Burden::Strained,
            _ => Burden::Overloaded,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Burden::Light => "Light",
            Burden::Burdened => "Burdened",
            Burden::Strained => "Strained",
            Burden::Overloaded => "Overloaded",
        }
    }

    /// Cost of one step, in half-turns.
    pub fn move_cost_halves(self) -> u32 {
        match self {
            Burden::Light => 2,
            Burden::Burdened => 3,
            Burden::Strained | Burden::Overloaded => 4,
        }
    }

    pub fn defense_penalty(self) -> i32 {
        match self {
            Burden::Light => 0,
            Burden::Burdened => 1,
            Burden::Strained | Burden::Overloaded => 3,
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            Burden::Light => "You move freely.",
            Burden::Burdened => "Your load slows you: moving takes longer and dodging is harder.",
            Burden::Strained => "You stagger under the load: every step is a slog and you can barely dodge.",
            Burden::Overloaded => "You can't move under this load. Drop something!",
        }
    }
}

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
    pub pack: Option<Pack>,
    pub crystals: u32,
    pub satiety: i32,
    pub stance: Stance,
    pub momentum: Momentum,
    pub kills: u32,
    /// Leftover half-turns from burdened movement.
    pub move_debt: u32,
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
            pack: None,
            crystals: 0,
            satiety: MAX_SATIETY * 3 / 4,
            stance: Stance::Balanced,
            momentum: Momentum::default(),
            kills: 0,
            move_debt: 0,
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
        10 + self.skill(Skill::Dodge).total() + self.stance.defense_mod() - penalty - self.burden().defense_penalty()
    }

    /// Everything carried: the pack's contents plus all worn and wielded gear.
    pub fn carried_weight(&self) -> u32 {
        self.inventory.weight()
            + self.gun.as_ref().map_or(0, weight::loaded_gun)
            + self.melee.map_or(0, weight::melee)
            + self.armor.map_or(0, weight::armor)
            + self.pack.map_or(0, weight::pack)
    }

    /// Capacity before burden sets in: a base, plus Brawn, plus a worn pack.
    pub fn carry_capacity(&self) -> u32 {
        let brawn = BASE_CAPACITY as i32 + CAPACITY_PER_BRAWN * self.attr_bonus(Attribute::Brawn);
        (brawn.max(MIN_CAPACITY as i32) as u32) + self.pack.map_or(0, |p| p.def().capacity)
    }

    pub fn burden(&self) -> Burden {
        Burden::of(self.carried_weight(), self.carry_capacity())
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
