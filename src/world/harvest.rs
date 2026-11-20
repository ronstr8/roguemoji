//! Harvest nodes: plants, pumps, wrecks, crates and caches you bump to
//! gather from, loaded from `data/harvest/*.toml` (see `data/README.md`).
//!
//! Each node lists the ways it can be opened. A way names the skill or
//! attribute it tests, the tool it needs and what a fumble costs. A hidden
//! node isn't drawn until you spot it or walk into it.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::Biome;
use crate::content::{self, Loot, Problems};
use crate::creatures::Player;
use crate::creatures::needs::Need;
use crate::items::Tool;
use crate::items::loot::Prize;
use crate::stats::check::success_chance;
use crate::stats::{Attribute, Skill};

/// A handle to a loaded harvest node: an index into the harvest table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Harvestable(u16);

#[derive(Debug)]
pub struct HarvestDef {
    /// The file name without `.toml`, e.g. `car_wreck`.
    pub id: String,
    pub name: String,
    pub glyph: String,
    /// The biome it's most common in; it turns up in the other one now and then.
    pub biome: Biome,
    /// How often it's placed, by depth, against the other nodes of its biome.
    pub chance: Loot,
    /// Not drawn until spotted or walked into.
    pub hidden: Option<Hidden>,
    /// Successful harvests before it's picked clean.
    pub uses: u8,
    /// What a success can gather, in order: a partial success stops at the
    /// first that comes up, and the first is the fallback if none do.
    pub yields: Vec<Yield>,
    /// The ways to work it.
    pub open: Vec<Way>,
}

/// The one look you get at a hidden node, when it first comes into view.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hidden {
    pub skill: Skill,
    pub target: i32,
    /// Logged when you spot it; a plain "You spot …" if absent.
    #[serde(default)]
    pub text: Option<String>,
}

/// Something a node can yield, and how many a success rolls.
#[derive(Clone, Copy, Debug)]
pub struct Yield {
    pub prize: Prize,
    pub min: u32,
    pub max: u32,
}

/// One way to work a node: the check it takes, the tool it needs and what an
/// attempt costs.
#[derive(Clone, Debug)]
pub struct Way {
    /// What you do to it, e.g. "pry open": "You pry open the Supply Crate".
    pub verb: String,
    pub test: Test,
    pub difficulty: i32,
    /// Needed on hand, in the pack or wielded; not used up unless it breaks.
    pub tool: Option<Tool>,
    /// Need changes every attempt costs, success or not (digging tires you).
    pub needs: Vec<(Need, i32)>,
    /// Added to the log on a partial success, e.g. what prying spoiled.
    pub partial: Option<String>,
    pub fumble: Fumble,
}

/// What a way's check rolls on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Test {
    /// A skill, which successes train.
    Skill(Skill),
    /// A raw attribute bonus, like Brawn for prying.
    Attribute(Attribute),
}

/// What a fumbled attempt costs.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fumble {
    /// HP change, zero or negative.
    #[serde(default)]
    pub hp: i32,
    /// The way's tool breaks.
    #[serde(default)]
    pub breaks: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWay {
    verb: String,
    #[serde(default)]
    skill: Option<Skill>,
    #[serde(default)]
    attribute: Option<Attribute>,
    difficulty: i32,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    needs: BTreeMap<String, i32>,
    #[serde(default)]
    partial: Option<String>,
    #[serde(default)]
    fumble: Fumble,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHarvest {
    name: String,
    glyph: String,
    biome: Biome,
    chance: Loot,
    #[serde(default)]
    hidden: Option<Hidden>,
    uses: u8,
    yields: Vec<RawYield>,
    open: Vec<RawWay>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawYield {
    item: String,
    min: u32,
    max: u32,
}

fn build(id: String, raw: RawHarvest) -> Result<HarvestDef, Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(!raw.glyph.trim().is_empty(), "glyph is empty");
    p.check(raw.chance.weight > 0 || raw.chance.per_depth > 0, "chance: weight or per_depth must be above 0");
    p.check(raw.uses > 0, "uses must be 1 or more");
    if let Some(h) = &raw.hidden {
        p.check((1..=20).contains(&h.target), "hidden: target must be 1 to 20");
    }
    p.check(!raw.yields.is_empty(), "yields is empty: give it at least one item");
    let yields = raw.yields.iter().filter_map(|y| resolve_yield(y, &mut p)).collect();
    p.check(!raw.open.is_empty(), "open needs at least one [[open]] way");
    let open = raw.open.into_iter().filter_map(|w| resolve_way(w, &mut p)).collect();
    p.finish(|| HarvestDef {
        id,
        name: raw.name,
        glyph: raw.glyph,
        biome: raw.biome,
        chance: raw.chance,
        hidden: raw.hidden,
        uses: raw.uses,
        yields,
        open,
    })
}

fn resolve_way(raw: RawWay, p: &mut Problems) -> Option<Way> {
    p.check(!raw.verb.trim().is_empty(), "open: verb is empty");
    p.check((1..=20).contains(&raw.difficulty), "open: difficulty must be 1 to 20");
    p.check(raw.fumble.hp <= 0, "open: fumble hp is a loss, so zero or negative");
    p.check(!raw.fumble.breaks || raw.tool.is_some(), "open: fumble breaks a tool, but the way has no `tool`");
    let tool = raw.tool.as_deref().and_then(|id| {
        let tool = Tool::lookup(id);
        p.check(tool.is_some(), format!("open: tool `{id}` is not a component or melee weapon"));
        tool
    });
    let needs = crate::creatures::needs::resolve(raw.needs, p, "open.needs");
    let test = match (raw.skill, raw.attribute) {
        (Some(s), None) => Test::Skill(s),
        (None, Some(a)) => Test::Attribute(a),
        _ => {
            p.check(false, format!("open: `{}` needs either a skill or an attribute, not both", raw.verb));
            return None;
        }
    };
    let (verb, difficulty, partial, fumble) = (raw.verb, raw.difficulty, raw.partial, raw.fumble);
    Some(Way { verb, test, difficulty, tool, needs, partial, fumble })
}

fn resolve_yield(y: &RawYield, p: &mut Problems) -> Option<Yield> {
    p.check(y.min <= y.max, format!("yields: `{}` has min above max", y.item));
    p.check(y.max > 0, format!("yields: `{}` never yields anything (max 0)", y.item));
    let prize = Prize::lookup(&y.item);
    p.check(prize.is_some(), format!("yields: `{}` is not an item", y.item));
    Some(Yield { prize: prize?, min: y.min, max: y.max })
}

content::registry!(Harvestable, HarvestDef, "harvest node", load);

fn load() -> Result<Vec<HarvestDef>, String> {
    content::load_defs("harvest", build)
}

/// Build a node from TOML text, as if it were a data file; for tests elsewhere.
#[cfg(test)]
pub fn parse(text: &str) -> Result<HarvestDef, String> {
    let raw: RawHarvest = toml::from_str(text).map_err(|e| e.to_string())?;
    build("test".into(), raw).map_err(|e| e.join("; "))
}

impl HarvestDef {
    /// Indexes of the ways `player` has the tools for.
    pub fn ways_for(&self, player: &Player) -> Vec<usize> {
        (0..self.open.len()).filter(|&i| self.open[i].ready(player)).collect()
    }
}

impl Way {
    /// Whether `player` has what this way needs on hand.
    pub fn ready(&self, player: &Player) -> bool {
        self.tool.is_none_or(|t| player.has_tool(t))
    }

    /// The bonus `player` rolls with.
    pub fn modifier(&self, player: &Player) -> i32 {
        match self.test {
            Test::Skill(s) => player.skill(s).total(),
            Test::Attribute(a) => player.attr_bonus(a),
        }
    }

    /// The chance `player` succeeds, from 0 to 1.
    pub fn odds(&self, player: &Player) -> f64 {
        success_chance(self.modifier(player), self.difficulty)
    }

    /// The skill or attribute it tests, for display.
    pub fn test_name(&self) -> &'static str {
        match self.test {
            Test::Skill(s) => s.name(),
            Test::Attribute(a) => a.name(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Component, GunModel};

    const GOOD: &str = r#"
        name = "Toolbox"
        glyph = "🧰"
        biome = "ruins"
        chance = { weight = 10 }
        uses = 1
        yields = [{ item = "nails", min = 1, max = 3 }, { item = "wire", min = 0, max = 1 }]

        [[open]]
        verb = "rummage through"
        skill = "scavenging"
        difficulty = 9
        fumble = { hp = -1 }
    "#;

    #[test]
    fn stock_harvest_nodes_load() {
        load().expect("stock harvest data is valid");
        for biome in [Biome::Ruins, Biome::Wastes] {
            let here = || Harvestable::all().filter(move |h| h.def().biome == biome);
            assert!(here().any(|h| h.def().hidden.is_none()), "{biome:?} has something to harvest");
            assert!(here().any(|h| h.def().hidden.is_some()), "{biome:?} has a hidden cache");
        }
    }

    #[test]
    fn parses_yields_in_order() {
        let d = parse(GOOD).unwrap();
        assert_eq!(d.yields.len(), 2);
        assert_eq!(d.yields[0].prize, Prize::Component(Component::named("nails")));
        let way = &d.open[0];
        assert_eq!((way.test, way.difficulty, way.fumble.hp), (Test::Skill(Skill::Scavenging), 9, -1));
        assert!(d.hidden.is_none() && way.needs.is_empty() && way.partial.is_none());
    }

    #[test]
    fn yields_name_any_item() {
        let d = parse(&GOOD.replace("\"nails\"", "\"ar15\"")).unwrap();
        assert_eq!(d.yields[0].prize, Prize::Gun(GunModel::named("ar15")));
    }

    #[test]
    fn a_cache_hides_and_offers_several_ways() {
        let cache = GOOD.replace("uses = 1", "uses = 1\nhidden = { skill = \"survival\", target = 13 }")
            + "\n[[open]]\nverb = \"pry open\"\nattribute = \"brawn\"\ndifficulty = 14\ntool = \"crowbar\"\nneeds = { rest = -40 }\npartial = \"It cracks\"\n";
        let d = parse(&cache).unwrap();
        assert_eq!(d.hidden.as_ref().map(|h| (h.skill, h.target)), Some((Skill::Survival, 13)));
        let pry = &d.open[1];
        assert_eq!(pry.test, Test::Attribute(Attribute::Brawn));
        assert_eq!(pry.needs.len(), 1);
        assert_eq!(pry.partial.as_deref(), Some("It cracks"));
    }

    #[test]
    fn tools_resolve_to_components_or_melee_weapons() {
        let with_tool = |id: &str| parse(&GOOD.replace("difficulty = 9", &format!("difficulty = 9\ntool = \"{id}\"")));
        assert!(matches!(with_tool("lockpicks").unwrap().open[0].tool, Some(Tool::Component(_))));
        assert!(matches!(with_tool("shovel").unwrap().open[0].tool, Some(Tool::Melee(_))));
        assert!(parse(GOOD).unwrap().open[0].tool.is_none());
    }

    #[test]
    fn rejects_bad_nodes() {
        let bad = |from: &str, to: &str| parse(&GOOD.replace(from, to)).unwrap_err();
        assert!(bad("\"nails\"", "\"moonbeams\"").contains("`moonbeams` is not an item"));
        assert!(bad("min = 1, max = 3", "min = 4, max = 3").contains("min above max"));
        assert!(bad("uses = 1", "uses = 0").contains("uses"));
        assert!(bad("weight = 10", "weight = 0").contains("chance"));
        assert!(bad("uses = 1", "uses = 1\nhidden = { skill = \"survival\", target = 30 }").contains("hidden: target"));
        assert!(bad("difficulty = 9", "difficulty = 0").contains("difficulty"));
        assert!(bad("hp = -1", "hp = 2").contains("fumble hp"));
        assert!(bad("hp = -1", "breaks = true").contains("has no `tool`"));
        assert!(bad("difficulty = 9", "difficulty = 9\ntool = \"wishes\"").contains("tool `wishes`"));
        assert!(bad("difficulty = 9", "difficulty = 9\nattribute = \"brawn\"").contains("not both"));
        assert!(bad("difficulty = 9", "difficulty = 9\nneeds = { joy = 5 }").contains("`joy`"));
        assert!(parse(&GOOD.replace("biome = \"ruins\"", "biome = \"moon\"")).is_err());
        assert!(parse(&GOOD.replace("uses = 1", "uses = 1\ncolor = \"red\"")).is_err());
    }
}
