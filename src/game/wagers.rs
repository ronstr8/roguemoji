//! Betting at a wager: walking up, staking crystals, and walking away.

use super::{Game, Tone};
use crate::creatures::needs::Need;
use crate::geometry::Pos;
use crate::stats::check::{self, CheckResult};
use crate::world::encounter::Encounter;
use crate::world::feature::Feature;
use crate::world::wager::{Verdict, Wager};

/// The wager you're sitting at, for its pop-up.
#[derive(Clone, Debug)]
pub struct Table {
    pub pos: Pos,
    pub encounter: Encounter,
    /// The scene as rolled when you walked up.
    pub text: String,
    /// The most recent bet, once you've placed one.
    pub last: Option<BetReport>,
    /// Bets placed since you sat down.
    pub bets: u32,
}

/// How one bet went.
#[derive(Clone, Debug)]
pub struct BetReport {
    pub check: CheckResult,
    pub verdict: Verdict,
    /// Crystals gained beyond the stake, or lost (negative).
    pub net: i64,
    /// The result's text as rolled.
    pub text: String,
    pub changes: Vec<(Need, i32)>,
}

impl Table {
    pub fn wager(&self) -> &'static Wager {
        self.encounter.def().wager.as_ref().expect("a table is only opened for a wager")
    }
}

impl Game {
    /// Walk up to a wager: roll its scene and keep it for the pop-up.
    pub(super) fn sit_at_table(&mut self, pos: Pos, encounter: Encounter) {
        let text = encounter.def().text.roll(&mut self.dice);
        self.table = Some(Table { pos, encounter, text, last: None, bets: 0 });
    }

    /// Crystals the house has left at `pos`, for a den.
    pub fn house_purse(&self, pos: Pos) -> u32 {
        match self.map.feature(pos) {
            Some(Feature::Encounter { purse, .. }) => *purse,
            _ => 0,
        }
    }

    /// Why you can't bet right now, or `None` if you can.
    pub fn bet_blocker(&self) -> Option<String> {
        let table = self.table.as_ref()?;
        let wager = table.wager();
        if !table.encounter.def().stays && table.bets > 0 {
            return Some("That was the only bet on offer.".into());
        }
        if !wager.house_covers(self.house_purse(table.pos)) {
            return Some("The house is broke.".into());
        }
        if self.player.crystals < wager.stake {
            return Some(format!("You need {} 💎 to bet.", wager.stake));
        }
        None
    }

    /// Stake crystals and roll. Returns false (changing nothing) if you can't bet.
    pub fn place_bet(&mut self) -> bool {
        let Some(table) = self.table.as_ref().filter(|_| self.bet_blocker().is_none()) else { return false };
        let (pos, encounter, wager) = (table.pos, table.encounter, table.wager());
        let modifier = wager.attribute.map_or(0, |a| self.player.attr_bonus(a));
        let check = check::roll(&mut self.dice, modifier, wager.target);
        let verdict = wager.verdict(check.tier);
        let net = wager.net(verdict);
        self.player.crystals = (self.player.crystals as i64 + net).max(0) as u32;
        if let Some(Feature::Encounter { purse, .. }) = self.map.feature_mut(pos) {
            *purse = wager.settle_purse(*purse, verdict);
        }
        let payout = wager.payout(verdict);
        let changes = self.apply_needs(&payout.needs, false);
        let text = payout.text.roll(&mut self.dice);
        let def = encounter.def();
        let tone = if net < 0 { Tone::Bad } else { Tone::Good };
        self.log.push(format!("{} {}: {} {net:+} 💎", def.glyph, def.name, verdict.name()), tone);
        let report = BetReport { check, verdict, net, text, changes };
        if let Some(t) = self.table.as_mut() {
            t.last = Some(report);
            t.bets += 1;
        }
        self.pass_time(1);
        true
    }

    /// Get up from the table. Walking away without betting logs the decline;
    /// a one-off wager is gone either way.
    pub fn leave_table(&mut self) {
        let Some(table) = self.table.take() else { return };
        if table.bets == 0 {
            let line = table.wager().decline.roll(&mut self.dice);
            self.log.push(line, Tone::Info);
        }
        if !table.encounter.def().stays {
            self.map.remove_feature(table.pos);
        }
    }
}
