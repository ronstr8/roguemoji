//! What happens when you walk into an encounter.

use super::{Game, Tone};
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
    /// Run the encounter at `pos`. One that stays logs its text and takes a
    /// turn; any other is removed and its report kept for the pop-up.
    /// Returns whether a pop-up should open.
    pub(super) fn encounter(&mut self, pos: Pos, e: Encounter) -> bool {
        let def = e.def();
        let report = self.apply_encounter(e);
        let line = format!("{} {}: {}", def.glyph, def.name, report.summary());
        let tone = if report.changes.iter().any(|(_, d)| *d < 0) { Tone::Bad } else { Tone::Good };
        if def.stays {
            self.log.push(format!("{} {}", report.text, report.summary()), tone);
            self.pass_time(1);
            return false;
        }
        self.map.remove_feature(pos);
        self.log.push(line, tone);
        self.last_encounter = Some(report);
        true
    }

    /// Roll any resist, change needs and hand over gifts.
    fn apply_encounter(&mut self, e: Encounter) -> EncounterReport {
        let def = e.def();
        let resist =
            def.resist.as_ref().map(|r| check::roll(&mut self.dice, self.player.attr_bonus(r.attribute), r.target));
        let soften = resist.is_some_and(|r| r.tier.succeeded());
        let mut changes = Vec::new();
        for &(need, delta) in &def.needs {
            let delta = if soften && delta < 0 { delta / 2 } else { delta };
            let Some(meter) = self.player.meter_mut(need) else { continue };
            if let Some(msg) = meter.adjust(delta).message {
                self.log.push(msg, Tone::Bad);
            }
            changes.push((need, delta));
        }
        for item in &def.gives {
            self.give_player(item.clone());
        }
        let text = def.text.roll(&mut self.dice);
        let resist_text = def.resist.as_ref().map(|r| r.text.roll(&mut self.dice));
        EncounterReport { encounter: e, text, resist, resist_text, changes, gifts: def.gives.clone() }
    }
}
