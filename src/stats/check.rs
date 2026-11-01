//! The universal success roll: 2d10 + modifier against a target number.
//!
//! Every check — attacks, harvesting, crafting — reports a *margin*
//! (total minus target) which is bucketed into a [`Tier`]. Systems read the
//! tier rather than a bare pass/fail, so better rolls always mean better results.

use crate::dice::Dice;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    /// Natural 2, or missed by 8 or more.
    Fumble,
    Fail,
    /// Made it by 0–3.
    Partial,
    /// Made it by 4–8.
    Success,
    /// Made it by 9+, or natural 20.
    Critical,
}

impl Tier {
    pub fn succeeded(self) -> bool {
        self >= Tier::Partial
    }

    pub fn from_margin(raw: i32, margin: i32) -> Tier {
        if raw == 2 {
            Tier::Fumble
        } else if raw == 20 || margin >= 9 {
            Tier::Critical
        } else if margin <= -8 {
            Tier::Fumble
        } else if margin < 0 {
            Tier::Fail
        } else if margin <= 3 {
            Tier::Partial
        } else {
            Tier::Success
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CheckResult {
    pub raw: i32,
    pub modifier: i32,
    pub target: i32,
    pub tier: Tier,
}

impl CheckResult {
    pub fn total(&self) -> i32 {
        self.raw + self.modifier
    }

    /// Compact roll summary for the log, e.g. `[2d10 13+2=15 vs 11]`.
    pub fn summary(&self) -> String {
        format!("[2d10 {}{:+}={} vs {}]", self.raw, self.modifier, self.total(), self.target)
    }
}

pub fn roll(dice: &mut Dice, modifier: i32, target: i32) -> CheckResult {
    let raw = dice.roll(2, 10);
    let tier = Tier::from_margin(raw, raw + modifier - target);
    CheckResult { raw, modifier, target, tier }
}

/// Exact probability that a check with `modifier` vs `target` succeeds
/// (Partial or better), from the 2d10 distribution.
pub fn success_chance(modifier: i32, target: i32) -> f64 {
    let mut wins = 0;
    for a in 1..=10 {
        for b in 1..=10 {
            let raw = a + b;
            if Tier::from_margin(raw, raw + modifier - target).succeeded() {
                wins += 1;
            }
        }
    }
    wins as f64 / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chance_bounds() {
        assert_eq!(success_chance(100, 0), 0.99); // natural 2 always fumbles
        assert_eq!(success_chance(-100, 0), 0.01); // natural 20 always crits
        assert!((success_chance(0, 11) - 0.55).abs() < 1e-9);
    }

    #[test]
    fn tiers_follow_margin() {
        assert_eq!(Tier::from_margin(10, -9), Tier::Fumble);
        assert_eq!(Tier::from_margin(10, -1), Tier::Fail);
        assert_eq!(Tier::from_margin(10, 0), Tier::Partial);
        assert_eq!(Tier::from_margin(10, 5), Tier::Success);
        assert_eq!(Tier::from_margin(10, 9), Tier::Critical);
        assert_eq!(Tier::from_margin(20, -3), Tier::Critical);
        assert_eq!(Tier::from_margin(2, 5), Tier::Fumble);
    }
}
