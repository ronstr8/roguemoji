//! Needs that drain every turn: hunger, thirst, rest and hope, loaded from
//! `data/needs/*.toml` (see `data/README.md`).
//!
//! A need is a meter from 0 to `max` that falls by `drain` each turn, faster
//! in a fight or under a heavy load if its file says so. Its stages name
//! ranges of the meter and say what being there costs: attribute modifiers,
//! and HP lost every few turns once it runs dry. A need with `sleep` (rest)
//! knocks you out at 0 and refills while you sleep instead.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::content::{self, Problems};
use crate::effects::{EffectSpec, RawEffect};
use crate::stats::Attribute;

/// A handle to a loaded need: an index into the need table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Need(u16);

/// The needs the game's code refers to by id.
pub const HUNGER: &str = "hunger";
pub const THIRST: &str = "thirst";
pub const REST: &str = "rest";
pub const HOPE: &str = "hope";

#[derive(Debug)]
pub struct NeedDef {
    /// The file name without `.toml`, e.g. `thirst`.
    pub id: String,
    /// What the meter measures, shown beside its value: `Hydration` for thirst.
    pub name: String,
    /// The meter when full.
    pub max: i32,
    /// The meter at the start of a game.
    pub start: i32,
    /// Lost each turn.
    pub drain: i32,
    /// Drain is multiplied by this on a turn you fight (attack or are attacked).
    pub fighting: i32,
    /// Whether drain scales with burden, as movement cost does (×1.5, ×2).
    pub burden: bool,
    /// How sleep works, for the need that sleeping restores.
    pub sleep: Option<SleepDef>,
    /// Highest first; the first stage covers the top of the meter.
    pub stages: Vec<Stage>,
}

/// Sleep, for the need it restores: you collapse when the meter hits 0.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SleepDef {
    /// Added to the meter each turn asleep.
    pub recover: i32,
    /// After collapsing, you wake once the meter reaches this.
    pub wake_at: i32,
    /// Regain 1 HP every this many turns asleep; 0 for none.
    #[serde(default)]
    pub heal_every: u32,
    /// A short nap lifts your mood: hope gained each early turn of a nap.
    #[serde(default)]
    pub relief: Option<Relief>,
    /// Sleeping too long, nap or collapse, wears hope down.
    #[serde(default)]
    pub oversleep: Option<Oversleep>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relief {
    /// Hope gained each turn.
    pub hope: i32,
    /// For this many turns from the start of a nap.
    pub turns: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oversleep {
    /// Turns asleep before it starts.
    pub after: u32,
    /// Hope lost each turn after that (negative).
    pub hope: i32,
    /// Logged once, the turn it starts.
    pub message: String,
}

impl SleepDef {
    /// Hope change for the `slept`th turn of a sleep (counting from 1).
    /// Only a nap brings relief; passing out from exhaustion isn't rest.
    pub fn hope_for_turn(&self, slept: u32, napping: bool) -> i32 {
        let relief = self.relief.filter(|r| napping && slept <= r.turns).map_or(0, |r| r.hope);
        let oversleep = self.oversleep.as_ref().filter(|o| slept > o.after).map_or(0, |o| o.hope);
        relief + oversleep
    }

    /// The oversleep message, on the one turn it applies: the first turn too long.
    pub fn overslept_now(&self, slept: u32) -> Option<&str> {
        self.oversleep.as_ref().filter(|o| slept == o.after + 1).map(|o| o.message.as_str())
    }

    fn check(&self, max: i32, p: &mut Problems) {
        p.check(self.recover > 0, "sleep.recover must be positive");
        p.check(self.wake_at > 0 && self.wake_at <= max, "sleep.wake_at must be above 0 and at most max");
        if let Some(r) = self.relief {
            p.check(r.hope > 0 && r.turns > 0, "sleep.relief: hope and turns must be positive");
        }
        if let Some(o) = &self.oversleep {
            p.check(o.hope < 0, "sleep.oversleep.hope must be negative: it's a loss");
            p.check(o.after > 0, "sleep.oversleep.after must be 1 or more turns");
            p.check(!o.message.trim().is_empty(), "sleep.oversleep.message is empty");
        }
    }
}

/// A named range of a need's meter and what being in it does.
#[derive(Debug)]
pub struct Stage {
    pub name: String,
    /// The highest meter value in this stage; `None` for the top stage.
    pub at_most: Option<i32>,
    /// Logged when the meter falls into this stage.
    pub message: Option<String>,
    /// Added to attributes while in this stage.
    pub modifiers: Vec<(Attribute, i32)>,
    /// HP lost while in this stage: 1 every `every` turns, with a message.
    pub hurts: Option<Hurts>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hurts {
    pub every: u32,
    pub message: String,
}

impl Stage {
    /// Whether this stage costs the player something, so the HUD flags it.
    pub fn is_warning(&self) -> bool {
        self.message.is_some() || self.hurts.is_some() || !self.modifiers.is_empty()
    }
}

impl Need {
    /// A change to this need as the log shows it, e.g. `Hope +350`.
    pub fn change_text(self, delta: i32) -> String {
        format!("{} {delta:+}", self.def().name)
    }
}

impl NeedDef {
    /// The stage a meter value falls in.
    pub fn stage(&self, value: i32) -> &Stage {
        &self.stages[self.stage_index(value)]
    }

    /// The position in `stages` of the stage a meter value falls in.
    fn stage_index(&self, value: i32) -> usize {
        self.stages.iter().rposition(|s| s.at_most.is_none_or(|top| value <= top)).unwrap_or(0)
    }
}

/// Water you can drink but probably shouldn't: from a puddle or a pump.
/// Drinking quenches thirst, then a Survival check against `risk` decides
/// whether `sickness` follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnsafeDrink {
    pub hydration: i32,
    pub risk: i32,
    pub sickness: EffectSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUnsafeDrink {
    hydration: i32,
    risk: i32,
    sickness: RawEffect,
}

impl RawUnsafeDrink {
    /// Validate into `p`, prefixing messages with `field`.
    pub fn build(&self, field: &str, p: &mut Problems) -> UnsafeDrink {
        let sickness = self.sickness.spec();
        p.check(self.hydration > 0, format!("{field}.hydration must be positive"));
        p.check(sickness.turns > 0, format!("{field}.sickness turns must be positive"));
        UnsafeDrink { hydration: self.hydration, risk: self.risk, sickness }
    }
}

/// One need's current level.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Meter {
    pub need: Need,
    pub value: i32,
    /// Half a point of drain carried over from a turn whose drain was fractional.
    carry: i32,
}

/// What made this turn harder on the body than standing still.
#[derive(Clone, Copy, Debug)]
pub struct Strain {
    /// Attacked or was attacked.
    pub fought: bool,
    /// The burden's movement cost in half-turns (2 = unburdened).
    pub burden_halves: u32,
}

impl Strain {
    /// A quiet turn under no load.
    #[cfg(test)]
    pub const NONE: Strain = Strain { fought: false, burden_halves: 2 };
}

/// What a change to a meter did.
#[derive(Debug, Default, PartialEq)]
pub struct Change {
    /// The message of a lower stage just entered.
    pub message: Option<&'static str>,
    /// HP lost this turn, with why.
    pub hurt: Option<&'static str>,
}

impl Meter {
    /// A meter at the need's starting level.
    pub fn new(need: Need) -> Self {
        Meter { need, value: need.def().start, carry: 0 }
    }

    pub fn def(&self) -> &'static NeedDef {
        self.need.def()
    }

    pub fn stage(&self) -> &'static Stage {
        self.def().stage(self.value)
    }

    /// How far down the stages the meter is: 0 for the top stage.
    pub fn depth(&self) -> usize {
        self.def().stage_index(self.value)
    }

    pub fn label(&self) -> &'static str {
        &self.stage().name
    }

    /// Add `delta` (negative to lower it), clamped to the meter's range.
    pub fn adjust(&mut self, delta: i32) -> Change {
        let def = self.def();
        let before = def.stage_index(self.value);
        self.value = (self.value + delta).clamp(0, def.max);
        let after = def.stage_index(self.value);
        let message = if after > before { def.stages[after].message.as_deref() } else { None };
        Change { message, hurt: None }
    }

    /// A meter at `value`, for setting up a situation.
    #[cfg(test)]
    pub fn at(need: Need, value: i32) -> Self {
        Meter { need, value, carry: 0 }
    }

    /// One turn passes: drain (scaled by `strain` as the need's file says),
    /// and hurt on the stage's schedule.
    pub fn tick(&mut self, turn: u64, strain: Strain) -> Change {
        let def = self.def();
        let mut halves = if def.burden { strain.burden_halves as i32 } else { 2 };
        if strain.fought {
            halves *= def.fighting;
        }
        let owed = def.drain * halves + self.carry;
        self.carry = owed % 2;
        let mut change = self.adjust(-owed / 2);
        if let Some(h) = &self.stage().hurts
            && turn.is_multiple_of(u64::from(h.every))
        {
            change.hurt = Some(&h.message);
        }
        change
    }

    /// This meter's modifier to an attribute at its current stage.
    pub fn modifier(&self, a: Attribute) -> i32 {
        self.stage().modifiers.iter().filter(|(m, _)| *m == a).map(|(_, n)| n).sum()
    }
}

// ── File format ──

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawNeed {
    name: String,
    max: i32,
    start: i32,
    drain: i32,
    #[serde(default = "one")]
    fighting: i32,
    #[serde(default)]
    burden: bool,
    #[serde(default)]
    sleep: Option<SleepDef>,
    stages: Vec<RawStage>,
}

fn one() -> i32 {
    1
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStage {
    name: String,
    #[serde(default)]
    at_most: Option<i32>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    modifiers: BTreeMap<String, i32>,
    #[serde(default)]
    hurts: Option<Hurts>,
}

fn attribute(name: &str) -> Option<Attribute> {
    Attribute::ALL.into_iter().find(|a| a.name().eq_ignore_ascii_case(name))
}

fn build(id: String, raw: RawNeed) -> Result<NeedDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(raw.max > 0, "max must be positive");
    p.check((0..=raw.max).contains(&raw.start), "start must be between 0 and max");
    p.check(raw.drain >= 0, "drain cannot be negative");
    p.check(raw.fighting >= 1, "fighting must be 1 or more");
    if let Some(s) = &raw.sleep {
        s.check(raw.max, &mut p);
    }
    p.check(!raw.stages.is_empty(), "needs at least one stage");
    let stages: Vec<Stage> = raw.stages.into_iter().map(|s| build_stage(s, &mut p)).collect();
    check_thresholds(&stages, raw.max, &mut p);
    p.finish(|| NeedDef {
        id,
        name: raw.name,
        max: raw.max,
        start: raw.start,
        drain: raw.drain,
        fighting: raw.fighting,
        burden: raw.burden,
        sleep: raw.sleep,
        stages,
    })
}

fn build_stage(raw: RawStage, p: &mut Problems) -> Stage {
    let name = raw.name;
    p.check(!name.trim().is_empty(), "a stage's name is empty");
    let mut modifiers = Vec::new();
    for (attr, n) in raw.modifiers {
        match attribute(&attr) {
            Some(a) => modifiers.push((a, n)),
            None => p.check(false, format!("stage {name}: unknown attribute `{attr}` in modifiers")),
        }
    }
    p.check(raw.hurts.as_ref().is_none_or(|h| h.every > 0), format!("stage {name}: hurts.every must be positive"));
    Stage { name, at_most: raw.at_most, message: raw.message, modifiers, hurts: raw.hurts }
}

/// The first stage is the top (no `at_most`); the rest step strictly down
/// from below `max` to zero or more.
fn check_thresholds(stages: &[Stage], max: i32, p: &mut Problems) {
    let Some((top, rest)) = stages.split_first() else { return };
    p.check(top.at_most.is_none(), "the first stage is the top of the meter and takes no at_most");
    let mut ceiling = max;
    for s in rest {
        match s.at_most {
            Some(n) if (0..ceiling).contains(&n) => ceiling = n,
            Some(_) => p.check(false, format!("stage {}: at_most must be below the stage above it and >= 0", s.name)),
            None => p.check(false, format!("stage {}: every stage after the first needs at_most", s.name)),
        }
    }
}

/// Turn need ids into handles, noting any that don't exist.
pub fn resolve(raw: BTreeMap<String, i32>, p: &mut Problems, label: &str) -> Vec<(Need, i32)> {
    let mut needs = Vec::new();
    for (need, n) in raw {
        match Need::lookup(&need) {
            Some(handle) => needs.push((handle, n)),
            None => p.check(false, format!("{label}: `{need}` is not in data/needs/")),
        }
    }
    needs
}

content::registry!(Need, NeedDef, "need", load);

fn load() -> Result<Vec<NeedDef>, String> {
    let defs = content::load_defs("needs", build)?;
    content::require("needs", |id| defs.iter().any(|d| d.id == id), [HUNGER, THIRST, REST, HOPE])?;
    if defs.iter().any(|d| d.id == REST && d.sleep.is_none()) {
        return Err(format!("data/needs/{REST}.toml needs `sleep`: it says how sleeping restores rest"));
    }
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<NeedDef, String> {
        let raw: RawNeed = toml::from_str(text).map_err(|e| e.to_string())?;
        build("t".into(), raw).map_err(|e| e.join("; "))
    }

    const GOOD: &str = r#"
        name = "Test"
        max = 100
        start = 80
        drain = 2
        [[stages]]
        name = "Fine"
        [[stages]]
        name = "Low"
        at_most = 40
        message = "Getting low."
        modifiers = { wit = -1 }
        [[stages]]
        name = "Out"
        at_most = 0
        hurts = { every = 5, message = "Ouch." }
    "#;

    #[test]
    fn stock_needs_load() {
        load().expect("stock need data is valid");
        assert_eq!(Need::named(THIRST).def().name, "Hydration");
        assert!(Need::named(REST).def().sleep.is_some());
    }

    #[test]
    fn stages_cover_the_meter() {
        let d = parse(GOOD).unwrap();
        let name = |v| d.stage(v).name.as_str();
        assert_eq!((name(100), name(41), name(40), name(1), name(0)), ("Fine", "Fine", "Low", "Low", "Out"));
        assert!(!d.stage(50).is_warning());
        assert!(d.stage(10).is_warning());
    }

    #[test]
    fn rejects_bad_content() {
        assert!(parse(&GOOD.replace("at_most = 40", "at_most = 0")).unwrap_err().contains("below"));
        assert!(parse(&GOOD.replace("at_most = 40\n", "")).unwrap_err().contains("needs at_most"));
        assert!(parse(&GOOD.replace("name = \"Fine\"", "name = \"Fine\"\nat_most = 90")).unwrap_err().contains("top"));
        assert!(parse(&GOOD.replace("wit", "charisma")).unwrap_err().contains("charisma"));
        assert!(parse(&GOOD.replace("every = 5", "every = 0")).unwrap_err().contains("every"));
        assert!(parse(&GOOD.replace("start = 80", "start = 101")).unwrap_err().contains("start"));
        assert!(parse(&GOOD.replace("drain = 2", "drain = 2\nfighting = 0")).unwrap_err().contains("fighting"));
        let sleep = |s: &str| GOOD.replace("drain = 2", &format!("drain = 2\nsleep = {{ {s} }}"));
        assert!(parse(&sleep("recover = 5, wake_at = 50")).is_ok());
        assert!(parse(&sleep("recover = 0, wake_at = 50")).unwrap_err().contains("recover"));
        assert!(parse(&sleep("recover = 5, wake_at = 101")).unwrap_err().contains("wake_at"));
    }

    #[test]
    fn thirst_announces_each_lower_stage_once_and_hurts_when_dry() {
        let need = Need::named(THIRST);
        let thirsty = need.def().stages[1].at_most.unwrap();
        let mut m = Meter::at(need, thirsty + 1);
        let drain = need.def().drain;
        let first = m.adjust(-drain);
        assert!(first.message.is_some(), "falling into Thirsty is announced");
        assert_eq!(m.adjust(-drain).message, None, "staying there is not");
        m.value = 0;
        let every = m.stage().hurts.as_ref().expect("dry thirst hurts").every;
        assert!(m.tick(u64::from(every), Strain::NONE).hurt.is_some());
        assert!(m.tick(u64::from(every) + 1, Strain::NONE).hurt.is_none());
        assert_eq!(m.adjust(10).message, None, "rising is never announced");
    }

    #[test]
    fn modifiers_apply_only_in_their_stage() {
        let need = Need::named(THIRST);
        let full = Meter::at(need, need.def().max);
        let dry = Meter::at(need, 0);
        assert_eq!(full.modifier(Attribute::Brains), 0);
        assert!(dry.modifier(Attribute::Brains) < 0);
    }

    #[test]
    fn rest_drains_faster_in_a_fight_and_under_a_load() {
        let need = Need::named(REST);
        let lost = |strain: Strain| {
            let mut m = Meter::at(need, need.def().max);
            for turn in 1..=10 {
                m.tick(turn, strain);
            }
            need.def().max - m.value
        };
        let calm = lost(Strain::NONE);
        assert!(calm > 0);
        assert_eq!(lost(Strain { fought: true, ..Strain::NONE }), calm * need.def().fighting);
        assert_eq!(lost(Strain { burden_halves: 3, ..Strain::NONE }), calm * 3 / 2, "half points carry over");
    }

    #[test]
    fn hunger_ignores_fighting_and_burden() {
        let need = Need::named(HUNGER);
        let mut calm = Meter::at(need, 500);
        let mut strained = Meter::at(need, 500);
        calm.tick(1, Strain::NONE);
        strained.tick(1, Strain { fought: true, burden_halves: 4 });
        assert_eq!(calm.value, strained.value);
    }

    #[test]
    fn sleep_lifts_hope_early_in_a_nap_and_drains_it_late() {
        let sleep: SleepDef = toml::from_str(
            "recover = 10\nwake_at = 400\nrelief = { hope = 4, turns = 30 }\noversleep = { after = 50, hope = -3, message = \"Too long.\" }",
        )
        .unwrap();
        assert_eq!(sleep.hope_for_turn(1, true), 4);
        assert_eq!(sleep.hope_for_turn(30, true), 4);
        assert_eq!(sleep.hope_for_turn(31, true), 0);
        assert_eq!(sleep.hope_for_turn(1, false), 0, "a collapse brings no relief");
        assert_eq!(sleep.hope_for_turn(50, false), 0);
        assert_eq!(sleep.hope_for_turn(51, false), -3, "but oversleeping counts either way");
        assert_eq!(sleep.overslept_now(51), Some("Too long."));
        assert_eq!(sleep.overslept_now(52), None, "said once");
    }

    #[test]
    fn rejects_backwards_sleep_moods() {
        let with = |extra: &str| {
            parse(&format!(
                "name = \"R\"\nmax = 100\nstart = 50\ndrain = 1\nsleep = {{ recover = 1, wake_at = 50, {extra} }}\n[[stages]]\nname = \"A\""
            ))
        };
        assert!(with("relief = { hope = 4, turns = 30 }").is_ok());
        assert!(with("relief = { hope = -4, turns = 30 }").unwrap_err().contains("relief"));
        assert!(with("oversleep = { after = 5, hope = 3, message = \"x\" }").unwrap_err().contains("negative"));
    }
}
