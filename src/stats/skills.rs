//! Skills, their governing attributes, ranks and training.

use super::attributes::{Attribute, bonus};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Skill {
    Melee,
    Brawling,
    SmallArms,
    LongArms,
    Dodge,
    Crafting,
    Scavenging,
    Survival,
}

pub const MAX_RANK: u8 = 5;
/// Bonus granted per skill rank.
pub const RANK_BONUS: i32 = 2;

impl Skill {
    pub const ALL: [Skill; 8] = [
        Skill::Melee,
        Skill::Brawling,
        Skill::SmallArms,
        Skill::LongArms,
        Skill::Dodge,
        Skill::Crafting,
        Skill::Scavenging,
        Skill::Survival,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Skill::Melee => "Melee",
            Skill::Brawling => "Brawling",
            Skill::SmallArms => "Small Arms",
            Skill::LongArms => "Long Arms",
            Skill::Dodge => "Dodge",
            Skill::Crafting => "Crafting",
            Skill::Scavenging => "Scavenging",
            Skill::Survival => "Survival",
        }
    }

    /// The two attributes whose bonuses are averaged into this skill.
    pub fn attributes(self) -> (Attribute, Attribute) {
        use Attribute::*;
        match self {
            Skill::Melee => (Brawn, Reflexes),
            Skill::Brawling => (Brawn, Fortitude),
            Skill::SmallArms => (Reflexes, Wit),
            Skill::LongArms => (Reflexes, Brains),
            Skill::Dodge => (Reflexes, Wit),
            Skill::Crafting => (Brains, Wit),
            Skill::Scavenging => (Wit, Brains),
            Skill::Survival => (Fortitude, Wit),
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// How a skill modifier was assembled, for display and for rolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillBreakdown {
    pub rank: u8,
    pub rank_bonus: i32,
    pub attr_bonus: i32,
    pub unskilled: bool,
    /// The attribute bonus was halved for being unskilled.
    pub halved: bool,
}

impl SkillBreakdown {
    pub fn total(&self) -> i32 {
        self.rank_bonus + self.attr_bonus
    }
}

/// Compute a skill modifier from its rank and current attribute scores.
///
/// The attribute bonus is the average of the two governing attributes' bonuses.
/// When unskilled (rank 0) only half of a *positive* attribute bonus applies;
/// penalties always apply in full — talent helps less without training, but
/// weakness hurts just the same.
pub fn breakdown(skill: Skill, rank: u8, score: impl Fn(Attribute) -> i32) -> SkillBreakdown {
    let (a, b) = skill.attributes();
    let averaged = (bonus(score(a)) + bonus(score(b))).div_euclid(2);
    let unskilled = rank == 0;
    let halved = unskilled && averaged > 0;
    let attr_bonus = if halved { averaged / 2 } else { averaged };
    SkillBreakdown { rank, rank_bonus: rank as i32 * RANK_BONUS, attr_bonus, unskilled, halved }
}

/// Ranks plus accumulated practice for every skill.
#[derive(Clone, Debug, Default)]
pub struct SkillSet {
    ranks: [u8; 8],
    xp: [u32; 8],
}

impl SkillSet {
    pub fn rank(&self, s: Skill) -> u8 {
        self.ranks[s.index()]
    }

    pub fn set_rank(&mut self, s: Skill, rank: u8) {
        self.ranks[s.index()] = rank.min(MAX_RANK);
    }

    pub fn xp(&self, s: Skill) -> u32 {
        self.xp[s.index()]
    }

    /// Practice needed to reach the next rank, or `None` at max rank.
    pub fn xp_needed(&self, s: Skill) -> Option<u32> {
        let r = self.rank(s) as u32;
        (r < MAX_RANK as u32).then(|| 6 * (r + 1) * (r + 1))
    }

    /// Add practice; returns `true` if the skill ranked up.
    pub fn train(&mut self, s: Skill, amount: u32) -> bool {
        let Some(needed) = self.xp_needed(s) else { return false };
        let i = s.index();
        self.xp[i] += amount;
        if self.xp[i] >= needed {
            self.xp[i] -= needed;
            self.ranks[i] += 1;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scores(v: i32) -> impl Fn(Attribute) -> i32 {
        move |_| v
    }

    #[test]
    fn unskilled_halves_positive_bonus() {
        let b = breakdown(Skill::Melee, 0, scores(9));
        assert_eq!(b.attr_bonus, 2);
        assert_eq!(b.total(), 2);
        let trained = breakdown(Skill::Melee, 1, scores(9));
        assert_eq!(trained.attr_bonus, 4);
        assert_eq!(trained.total(), 6);
    }

    #[test]
    fn unskilled_penalty_is_full() {
        let b = breakdown(Skill::Melee, 0, scores(3));
        assert_eq!(b.attr_bonus, -2);
    }

    #[test]
    fn training_ranks_up() {
        let mut s = SkillSet::default();
        assert!(!s.train(Skill::Crafting, 5));
        assert!(s.train(Skill::Crafting, 1));
        assert_eq!(s.rank(Skill::Crafting), 1);
    }
}
