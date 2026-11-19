//! What happens when you walk into an encounter.

use super::{Event, Game, Tone};
use crate::content::flavor::Picks;
use crate::creatures::needs::Need;
use crate::geometry::Pos;
use crate::items::Item;
use crate::stats::check::{self, CheckResult};
use crate::world::encounter::Encounter;

/// What an encounter did, for its pop-up and the log.
#[derive(Clone, Debug)]
pub struct EncounterReport {
    pub encounter: Encounter,
    /// The scene as rolled this time.
    pub text: String,
    /// The resist check, if the encounter has one.
    pub resist: Option<CheckResult>,
    /// The encounter's resist text as rolled, if it has one.
    pub resist_text: Option<String>,
    /// Each need's change as applied (losses halved if resisted).
    pub changes: Vec<(Need, i32)>,
    pub gifts: Vec<Item>,
}

impl EncounterReport {
    /// One line for the log, e.g. `Hope +200 · got Bottled Water`.
    pub fn summary(&self) -> String {
        let mut parts: Vec<String> = self.changes.iter().map(|(n, d)| format!("{} {d:+}", n.def().name)).collect();
        parts.extend(self.gifts.iter().map(|i| format!("got {}", i.display_name())));
        parts.join(" · ")
    }

    pub fn resisted(&self) -> bool {
        self.resist.is_some_and(|r| r.tier.succeeded())
    }
}

impl Game {
    /// Run the encounter at `pos` and say which pop-up, if any, opens. A
    /// wager opens its table. Otherwise one that stays logs its text and
    /// takes a turn, and any other is removed and its report kept.
    pub(super) fn encounter(&mut self, pos: Pos, e: Encounter) -> Event {
        let def = e.def();
        if def.wager.is_some() {
            self.sit_at_table(pos, e);
            return Event::OpenWager;
        }
        let report = self.apply_encounter(e);
        let line = format!("{} {}: {}", def.glyph, def.name, report.summary());
        let tone = if report.changes.iter().any(|(_, d)| *d < 0) { Tone::Bad } else { Tone::Good };
        if def.stays {
            self.log.push(format!("{} {}", report.text, report.summary()), tone);
            self.pass_time(1);
            return Event::None;
        }
        self.map.remove_feature(pos);
        self.log.push(line, tone);
        self.last_encounter = Some(report);
        Event::OpenEncounter
    }

    /// Roll any resist, change needs and hand over gifts.
    fn apply_encounter(&mut self, e: Encounter) -> EncounterReport {
        let def = e.def();
        let resist =
            def.resist.as_ref().map(|r| check::roll(&mut self.dice, self.player.attr_bonus(r.attribute), r.target));
        let soften = resist.is_some_and(|r| r.tier.succeeded());
        let changes = self.apply_needs(&def.needs, soften);
        for item in &def.gives {
            self.give_player(item.clone());
        }
        let mut picks = Picks::default();
        let text = def.text.roll_with(&mut self.dice, &mut picks);
        let resist_text = def.resist.as_ref().map(|r| r.text.roll_with(&mut self.dice, &mut picks));
        EncounterReport { encounter: e, text, resist, resist_text, changes, gifts: def.gives.clone() }
    }

    /// Change needs, halving losses if `soften`; returns each change as applied.
    pub(super) fn apply_needs(&mut self, needs: &[(Need, i32)], soften: bool) -> Vec<(Need, i32)> {
        let mut changes = Vec::new();
        for &(need, delta) in needs {
            let delta = if soften && delta < 0 { delta / 2 } else { delta };
            let Some(meter) = self.player.meter_mut(need) else { continue };
            if let Some(msg) = meter.adjust(delta).message {
                self.log.push(msg, Tone::Bad);
            }
            changes.push((need, delta));
        }
        changes
    }
}
