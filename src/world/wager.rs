//! Wagers: an encounter that offers a bet instead of a fixed effect.
//!
//! You stake crystals (which are also your healing) on a check. The check is
//! against an attribute, or pure luck when the wager names none. A den that
//! `stays` keeps a house purse: winnings come out of it, losses go in, and it
//! closes when it can't cover the biggest payout. A wager that doesn't stay is
//! a one-off: one bet, or walk away, and it's gone.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::content::Problems;
use crate::content::flavor::Flavor;
use crate::creatures::needs::Need;
use crate::stats::Attribute;
use crate::stats::check::Tier;

/// A loaded wager: what it costs, what decides it, and what each result does.
#[derive(Debug)]
pub struct Wager {
    /// Crystals put up for each bet.
    pub stake: u32,
    /// The attribute the check uses; `None` is pure luck.
    pub attribute: Option<Attribute>,
    pub target: i32,
    /// Crystals the house starts with, for a den that stays.
    pub purse: Option<u32>,
    /// Logged when you walk away without betting.
    pub decline: Flavor,
    pub win: Payout,
    pub lose: Payout,
    /// A narrow result that returns the stake; without one, it counts as a win.
    pub push: Option<Payout>,
}

/// What one result does to you.
#[derive(Debug)]
pub struct Payout {
    pub text: Flavor,
    /// Crystals won on top of the stake back. Only a win pays.
    pub crystals: u32,
    pub needs: Vec<(Need, i32)>,
}

/// How a bet came out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// A Critical: the win pays double.
    BigWin,
    Win,
    Push,
    Lose,
}

impl Verdict {
    /// One word for the log and the pop-up.
    pub fn name(self) -> &'static str {
        match self {
            Verdict::BigWin => "Big win",
            Verdict::Win => "Win",
            Verdict::Push => "Push",
            Verdict::Lose => "Loss",
        }
    }
}

impl Wager {
    /// The result a check tier earns.
    pub fn verdict(&self, tier: Tier) -> Verdict {
        match tier {
            Tier::Critical => Verdict::BigWin,
            Tier::Success => Verdict::Win,
            Tier::Partial if self.push.is_some() => Verdict::Push,
            Tier::Partial => Verdict::Win,
            Tier::Fail | Tier::Fumble => Verdict::Lose,
        }
    }

    /// The payout table for a verdict.
    pub fn payout(&self, v: Verdict) -> &Payout {
        match v {
            Verdict::BigWin | Verdict::Win => &self.win,
            Verdict::Push => self.push.as_ref().unwrap_or(&self.win),
            Verdict::Lose => &self.lose,
        }
    }

    /// Crystals you come away with beyond your stake (negative on a loss).
    pub fn net(&self, v: Verdict) -> i64 {
        match v {
            Verdict::BigWin => 2 * self.win.crystals as i64,
            Verdict::Win => self.win.crystals as i64,
            Verdict::Push => 0,
            Verdict::Lose => -(self.stake as i64),
        }
    }

    /// The most one bet can take from the house.
    pub fn max_payout(&self) -> u32 {
        2 * self.win.crystals
    }

    /// Whether a den holding `purse` can still take a bet.
    pub fn house_covers(&self, purse: u32) -> bool {
        self.purse.is_none() || purse >= self.max_payout()
    }

    /// The house's purse after a bet came out `v`.
    pub fn settle_purse(&self, purse: u32, v: Verdict) -> u32 {
        let net = self.net(v);
        if net >= 0 { purse.saturating_sub(net as u32) } else { purse + (-net) as u32 }
    }
}

/// A `[wager]` table as written in an encounter file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawWager {
    stake: u32,
    #[serde(default)]
    attribute: Option<Attribute>,
    target: i32,
    #[serde(default)]
    purse: Option<u32>,
    decline: Flavor,
    win: RawPayout,
    lose: RawPayout,
    #[serde(default)]
    push: Option<RawPayout>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPayout {
    text: Flavor,
    #[serde(default)]
    crystals: u32,
    #[serde(default)]
    needs: BTreeMap<String, i32>,
}

/// Turn need ids into handles, noting any that don't exist.
pub(super) fn resolve_needs(raw: BTreeMap<String, i32>, p: &mut Problems, label: &str) -> Vec<(Need, i32)> {
    let mut needs = Vec::new();
    for (need, n) in raw {
        match Need::lookup(&need) {
            Some(handle) => needs.push((handle, n)),
            None => p.check(false, format!("{label}: `{need}` is not in data/needs/")),
        }
    }
    needs
}

fn payout(raw: RawPayout, p: &mut Problems, label: &str) -> Payout {
    p.check(!raw.text.is_blank(), format!("{label}: text is empty"));
    let needs = resolve_needs(raw.needs, p, &format!("{label}.needs"));
    Payout { text: raw.text, crystals: raw.crystals, needs }
}

impl RawWager {
    /// Check the wager and build it; `stays` says whether it's a den.
    pub(super) fn build(self, stays: bool, p: &mut Problems) -> Wager {
        p.check(self.stake > 0, "wager: stake must be 1 or more");
        p.check((2..=20).contains(&self.target), "wager: target must be 2–20 (it's rolled on 2d10)");
        p.check(self.win.crystals > 0, "wager.win: crystals must be 1 or more, or nobody would bet");
        p.check(self.lose.crystals == 0, "wager.lose: crystals are paid on a win only");
        p.check(!self.decline.is_blank(), "wager: decline is empty");
        let push_pays = self.push.as_ref().is_some_and(|r| r.crystals > 0);
        p.check(!push_pays, "wager.push: a push returns the stake; it can't pay crystals");
        match (stays, self.purse) {
            (true, None) => p.check(false, "wager: a den that stays needs a house `purse`"),
            (false, Some(_)) => p.check(false, "wager: `purse` is only for a den that stays"),
            (true, Some(purse)) => {
                p.check(purse >= 2 * self.win.crystals, "wager: purse must cover a big win (twice win.crystals)")
            }
            (false, None) => {}
        }
        Wager {
            stake: self.stake,
            attribute: self.attribute,
            target: self.target,
            purse: self.purse,
            decline: self.decline,
            win: payout(self.win, p, "wager.win"),
            lose: payout(self.lose, p, "wager.lose"),
            push: self.push.map(|r| payout(r, p, "wager.push")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wager(push: bool, purse: Option<u32>) -> Wager {
        let text = || Flavor::parse("x").unwrap();
        let pay = |crystals| Payout { text: text(), crystals, needs: vec![] };
        Wager {
            stake: 3,
            attribute: None,
            target: 11,
            purse,
            decline: text(),
            win: pay(4),
            lose: pay(0),
            push: push.then(|| pay(0)),
        }
    }

    #[test]
    fn tiers_map_to_verdicts() {
        let w = wager(true, None);
        assert_eq!(w.verdict(Tier::Critical), Verdict::BigWin);
        assert_eq!(w.verdict(Tier::Success), Verdict::Win);
        assert_eq!(w.verdict(Tier::Partial), Verdict::Push);
        assert_eq!(w.verdict(Tier::Fumble), Verdict::Lose);
        assert_eq!(wager(false, None).verdict(Tier::Partial), Verdict::Win, "no push table: a narrow win");
    }

    #[test]
    fn payouts_and_the_house_purse() {
        let w = wager(false, Some(10));
        assert_eq!((w.net(Verdict::BigWin), w.net(Verdict::Win), w.net(Verdict::Lose)), (8, 4, -3));
        assert_eq!(w.settle_purse(10, Verdict::Win), 6);
        assert_eq!(w.settle_purse(10, Verdict::Lose), 13);
        assert!(w.house_covers(8));
        assert!(!w.house_covers(7), "can't cover a big win");
        assert!(wager(false, None).house_covers(0), "a one-off has no purse to run dry");
    }
}
