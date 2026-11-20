//! Encounters: things happening on the map that you walk into, loaded from
//! `data/encounters/*.toml` (see `data/README.md`).
//!
//! Most are one-off scenes: stepping in shows their text, changes your needs
//! (Hope, mostly) and may hand you items, and then they're gone. Ones that
//! `stay` are places you bump again and again, like a quiet sanctum.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::content::flavor::Flavor;
use crate::content::{self, Loot, Problems};
use crate::creatures::needs::{self, Need};
use crate::items::{Component, Consumable, Item};
use crate::stats::Attribute;

use super::wager::{RawWager, Wager};

/// A handle to a loaded encounter: an index into the encounter table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Encounter(u16);

#[derive(Debug)]
pub struct EncounterDef {
    /// The file name without `.toml`, e.g. `friendly_stranger`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    /// The scene, shown in a pop-up (or the log, for one that stays); rolled
    /// fresh each time it's shown.
    pub text: Flavor,
    /// How often it turns up, by depth.
    pub chance: Loot,
    /// Stays on the map and is bumped rather than walked into; its text goes
    /// to the log instead of a pop-up.
    pub stays: bool,
    /// Changes to needs, e.g. Hope +200.
    pub needs: Vec<(Need, i32)>,
    pub gives: Vec<Item>,
    pub resist: Option<Resist>,
    /// A bet offered instead of fixed effects.
    pub wager: Option<Wager>,
}

/// What an encounter promises from across the map, which colors its glow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Hope,
    /// A gambler or a den: it could go either way.
    Chance,
    Horror,
}

impl EncounterDef {
    /// A wager is chance; otherwise one that costs Hope is a horror, and
    /// anything else is a hope.
    pub fn mood(&self) -> Mood {
        if self.wager.is_some() {
            return Mood::Chance;
        }
        let hope = Need::named(needs::HOPE);
        let change: i32 = self.needs.iter().filter(|(n, _)| *n == hope).map(|(_, v)| v).sum();
        if change < 0 { Mood::Horror } else { Mood::Hope }
    }
}

/// A check that softens a bad encounter: on success, losses are halved.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resist {
    pub attribute: Attribute,
    pub target: i32,
    /// Shown when the check succeeds.
    pub text: Flavor,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEncounter {
    name: String,
    glyph: String,
    text: Flavor,
    #[serde(default)]
    chance: Loot,
    #[serde(default)]
    stays: bool,
    #[serde(default)]
    needs: BTreeMap<String, i32>,
    #[serde(default)]
    gives: BTreeMap<String, u32>,
    #[serde(default)]
    resist: Option<Resist>,
    #[serde(default)]
    wager: Option<RawWager>,
}

/// An item an encounter can give: any consumable or component, by id.
fn gift(id: &str, qty: u32) -> Option<Item> {
    Consumable::lookup(id)
        .map(|c| Item::food(c, qty))
        .or_else(|| Component::lookup(id).map(|c| Item::component(c, qty)))
}

fn build(id: String, raw: RawEncounter) -> Result<EncounterDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(!raw.text.is_blank(), "text is empty");
    let needs = needs::resolve(raw.needs, &mut p, "needs");
    let mut gives = Vec::new();
    for (item, qty) in raw.gives {
        p.check(qty > 0, format!("gives: `{item}` needs a quantity of 1 or more"));
        match gift(&item, qty) {
            Some(i) => gives.push(i),
            None => p.check(false, format!("gives: `{item}` is not a consumable or component")),
        }
    }
    let wager = raw.wager.map(|w| w.build(raw.stays, &mut p));
    let mut texts = vec![&raw.text];
    texts.extend(raw.resist.as_ref().map(|r| &r.text));
    texts.extend(wager.iter().flat_map(Wager::texts));
    if let Err(e) = Flavor::check_linked(&texts) {
        p.check(false, format!("tags shared between texts: {e}"));
    }
    if wager.is_some() {
        let fixed = !needs.is_empty() || !gives.is_empty() || raw.resist.is_some();
        p.check(!fixed, "a wager's effects go in its win and lose tables, not `needs`, `gives` or `resist`");
    } else {
        p.check(!needs.is_empty() || !gives.is_empty(), "does nothing: give it `needs`, `gives` or a `wager`");
    }
    p.finish(|| EncounterDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        text: raw.text,
        chance: raw.chance,
        stays: raw.stays,
        needs,
        gives,
        resist: raw.resist,
        wager,
    })
}

content::registry!(Encounter, EncounterDef, "encounter", load);

fn load() -> Result<Vec<EncounterDef>, String> {
    content::load_defs("encounters", build)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<EncounterDef, String> {
        let raw: RawEncounter = toml::from_str(text).map_err(|e| e.to_string())?;
        build("t".into(), raw).map_err(|e| e.join("; "))
    }

    const GOOD: &str = r#"
        name = "Kind Soul"
        glyph = "🤝"
        text = "Someone shares a meal."
        chance = { weight = 2 }
        needs = { hope = 150, hunger = 100 }
        gives = { bottled_water = 1, rags = 2 }
        resist = { attribute = "willpower", target = 10, text = "x" }
    "#;

    #[test]
    fn stock_encounters_load() {
        load().expect("stock encounter data is valid");
        assert!(Encounter::all().any(|e| e.def().stays), "there's at least one place to return to");
    }

    #[test]
    fn mood_follows_hope_and_wagers() {
        assert_eq!(parse(GOOD).unwrap().mood(), Mood::Hope);
        assert_eq!(parse(&GOOD.replace("hope = 150", "hope = -150")).unwrap().mood(), Mood::Horror);
        assert_eq!(parse(DEN).unwrap().mood(), Mood::Chance);
        let mood = |id: &str| Encounter::all().find(|e| e.def().id == id).unwrap().def().mood();
        assert_eq!(mood("campfire_circle"), Mood::Hope);
        assert_eq!(mood("card_sharp"), Mood::Chance);
        assert_eq!(mood("witness_horrors"), Mood::Horror);
    }

    #[test]
    fn parses_needs_and_gifts() {
        let d = parse(GOOD).unwrap();
        assert_eq!(d.needs.len(), 2);
        assert_eq!(d.gives.iter().map(|i| i.qty).sum::<u32>(), 3);
        assert!(!d.stays);
        assert_eq!(d.resist.unwrap().attribute, Attribute::Willpower);
    }

    const DEN: &str = r#"
        name = "Den"
        glyph = "🎲"
        text = "Dice."
        stays = true
        [wager]
        stake = 3
        target = 10
        purse = 30
        decline = "No."
        [wager.win]
        text = "Won."
        crystals = 3
        [wager.lose]
        text = "Lost."
    "#;

    #[test]
    fn parses_a_wager() {
        let w = parse(DEN).unwrap().wager.unwrap();
        assert_eq!((w.stake, w.target, w.purse, w.attribute), (3, 10, Some(30), None));
        assert!(w.push.is_none());
    }

    #[test]
    fn rejects_bad_wagers() {
        let bad = |from: &str, to: &str| parse(&DEN.replace(from, to)).unwrap_err();
        assert!(bad("purse = 30", "").contains("needs a house `purse`"));
        assert!(bad("stays = true", "").contains("only for a den"));
        assert!(bad("purse = 30", "purse = 5").contains("cover a big win"));
        assert!(bad("stake = 3", "stake = 0").contains("stake"));
        assert!(bad("target = 10", "target = 25").contains("target"));
        assert!(bad("text = \"Lost.\"", "text = \"Lost.\"\ncrystals = 2").contains("win only"));
        assert!(bad("stays = true", "stays = true\nneeds = { hope = 5 }").contains("win and lose tables"));
        assert!(bad("text = \"Won.\"", "text = \"Won.\"\nneeds = { joy = 5 }").contains("wager.win.needs"));
        let linked = DEN.replace("text = \"Dice.\"", "text = \"{@who A | B}\"");
        let err = parse(&linked.replace("text = \"Won.\"", "text = \"{@who x | y | z}\"")).unwrap_err();
        assert!(err.contains("tags shared between texts"), "{err}");
    }

    #[test]
    fn rejects_bad_content() {
        assert!(parse(&GOOD.replace("hope = 150", "joy = 150")).unwrap_err().contains("joy"));
        assert!(parse(&GOOD.replace("bottled_water", "gold_bar")).unwrap_err().contains("gold_bar"));
        assert!(parse(&GOOD.replace("rags = 2", "rags = 0")).unwrap_err().contains("quantity"));
        assert!(parse(&GOOD.replace("\"willpower\"", "\"luck\"")).is_err());
        let empty = "name = \"x\"\nglyph = \"x\"\ntext = \"x\"\n";
        assert!(parse(empty).unwrap_err().contains("does nothing"));
        assert!(parse(&GOOD.replace("shares a meal", "shares { a meal")).unwrap_err().contains("never closed"));
        assert!(parse(&GOOD.replace("text = \"x\"", "text = \"{ x }\"")).unwrap_err().contains("two or more"));
    }
}
