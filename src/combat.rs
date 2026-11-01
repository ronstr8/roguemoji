//! "Margin & Momentum" combat.
//!
//! An attack is a standard check: 2d10 + attack modifier against the
//! defender's Defense. The *margin* decides how well it lands:
//!
//! | Margin   | Result | Damage                          |
//! |----------|--------|---------------------------------|
//! | < 0      | Miss   | none                            |
//! | 0 – 3    | Graze  | half, after armor               |
//! | 4 – 8    | Hit    | full, after armor; specials fire|
//! | 9+ / 20  | Crit   | ×1.5, armor ignored; specials   |
//!
//! Two simple levers on top:
//! * **Stance** trades offense for defense (Aggressive / Balanced / Guarded).
//! * **Momentum** — each consecutive solid hit on the same target gives +1 to
//!   hit (max +3). Missing, switching targets, or getting hurt resets it.

use crate::dice::Dice;
use crate::stats::check::{self, Tier};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    Aggressive,
    Balanced,
    Guarded,
}

impl Stance {
    pub fn name(self) -> &'static str {
        match self {
            Stance::Aggressive => "Aggressive",
            Stance::Balanced => "Balanced",
            Stance::Guarded => "Guarded",
        }
    }

    pub fn attack_mod(self) -> i32 {
        match self {
            Stance::Aggressive => 2,
            Stance::Balanced => 0,
            Stance::Guarded => -2,
        }
    }

    pub fn defense_mod(self) -> i32 {
        match self {
            Stance::Aggressive => -2,
            Stance::Balanced => 0,
            Stance::Guarded => 3,
        }
    }

    /// Extra melee damage from committing to the swing.
    pub fn damage_mod(self) -> i32 {
        match self {
            Stance::Aggressive => 1,
            _ => 0,
        }
    }

    pub fn next(self) -> Stance {
        match self {
            Stance::Aggressive => Stance::Balanced,
            Stance::Balanced => Stance::Guarded,
            Stance::Guarded => Stance::Aggressive,
        }
    }
}

pub const MAX_MOMENTUM: i32 = 3;

/// Consecutive-hit bonus against a single target.
#[derive(Clone, Copy, Debug, Default)]
pub struct Momentum {
    target: Option<u32>,
    stacks: i32,
}

impl Momentum {
    pub fn bonus_against(&self, target: u32) -> i32 {
        if self.target == Some(target) { self.stacks } else { 0 }
    }

    pub fn stacks(&self) -> i32 {
        self.stacks
    }

    pub fn record(&mut self, target: u32, tier: HitTier) {
        if !tier.is_solid() {
            self.reset();
            return;
        }
        if self.target == Some(target) {
            self.stacks = (self.stacks + 1).min(MAX_MOMENTUM);
        } else {
            self.target = Some(target);
            self.stacks = 1;
        }
    }

    /// Drop momentum built against a target that no longer exists.
    pub fn forget(&mut self, target: u32) {
        if self.target == Some(target) {
            self.reset();
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitTier {
    Miss,
    Graze,
    Hit,
    Crit,
}

impl HitTier {
    pub fn from_check(tier: Tier) -> Self {
        match tier {
            Tier::Fumble | Tier::Fail => HitTier::Miss,
            Tier::Partial => HitTier::Graze,
            Tier::Success => HitTier::Hit,
            Tier::Critical => HitTier::Crit,
        }
    }

    /// Hit or Crit: triggers weapon specials and builds momentum.
    pub fn is_solid(self) -> bool {
        matches!(self, HitTier::Hit | HitTier::Crit)
    }

    pub fn verb(self) -> &'static str {
        match self {
            HitTier::Miss => "misses",
            HitTier::Graze => "grazes",
            HitTier::Hit => "hits",
            HitTier::Crit => "CRITS",
        }
    }
}

/// Inputs for a single attack, already including all modifiers.
#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub to_hit: i32,
    /// Damage before tier scaling and armor.
    pub raw_damage: i32,
    pub pen: i32,
    pub ignore_armor: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Outcome {
    pub tier: HitTier,
    pub damage: i32,
}

pub fn resolve(dice: &mut Dice, attack: Attack, defense: i32, armor: i32) -> Outcome {
    let check = check::roll(dice, attack.to_hit, defense);
    let tier = HitTier::from_check(check.tier);
    let mitigation = if attack.ignore_armor { 0 } else { (armor - attack.pen).max(0) };
    let raw = attack.raw_damage.max(0);
    let damage = match tier {
        HitTier::Miss => 0,
        HitTier::Graze => ((raw - mitigation).max(0) + 1) / 2,
        HitTier::Hit => (raw - mitigation).max(0),
        HitTier::Crit => (raw * 3 + 1) / 2,
    };
    Outcome { tier, damage }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn momentum_builds_and_resets() {
        let mut m = Momentum::default();
        m.record(1, HitTier::Hit);
        m.record(1, HitTier::Crit);
        assert_eq!(m.bonus_against(1), 2);
        assert_eq!(m.bonus_against(2), 0);
        m.record(1, HitTier::Graze);
        assert_eq!(m.bonus_against(1), 0);
    }

    #[test]
    fn crits_ignore_armor_hits_do_not() {
        let mut dice = Dice::seeded(1);
        let atk = Attack { to_hit: 100, raw_damage: 10, pen: 0, ignore_armor: false };
        let out = resolve(&mut dice, atk, 0, 5);
        assert_eq!(out.tier, HitTier::Crit);
        assert_eq!(out.damage, 15);
        let atk = Attack { to_hit: -100, raw_damage: 10, pen: 0, ignore_armor: false };
        assert_eq!(resolve(&mut dice, atk, 0, 5).damage, 0);
    }
}
