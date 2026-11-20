//! Harvest nodes: plants, pumps, wrecks and crates you bump to gather
//! components, loaded from `data/harvest/*.toml` (see `data/README.md`).
//!
//! Each node lists the ways it can be opened. A way names the skill it
//! tests and what a fumble costs.

use serde::Deserialize;

use super::Biome;
use crate::content::{self, Problems};
use crate::items::{Component, Tool};
use crate::stats::Skill;

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
    /// Successful harvests before it's picked clean.
    pub uses: u8,
    /// What a success can gather, in order: a partial success stops at the
    /// first that comes up, and the first is the fallback if none do.
    pub yields: Vec<Yield>,
    /// The ways to work it.
    pub open: Vec<Way>,
}

/// A component a node can yield, and how many a success rolls.
#[derive(Clone, Copy, Debug)]
pub struct Yield {
    pub component: Component,
    pub min: u32,
    pub max: u32,
}

/// One way to work a node: the check it takes, the tool it needs and what a
/// fumble costs.
#[derive(Clone, Debug)]
pub struct Way {
    /// What you do to it, e.g. "pry open": "You pry open the Supply Crate".
    pub verb: String,
    pub skill: Skill,
    pub difficulty: i32,
    /// Needed on hand, in the pack or wielded; not used up unless it breaks.
    pub tool: Option<Tool>,
    pub fumble: Fumble,
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
    skill: Skill,
    difficulty: i32,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    fumble: Fumble,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawHarvest {
    name: String,
    glyph: String,
    biome: Biome,
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
    p.check(raw.uses > 0, "uses must be 1 or more");
    p.check(!raw.yields.is_empty(), "yields is empty: give it at least one component");
    let yields = raw.yields.iter().filter_map(|y| resolve_yield(y, &mut p)).collect();
    // Picking among several ways comes with hidden caches; until then a node has one.
    p.check(raw.open.len() == 1, "open needs exactly one [[open]] way");
    let open = raw.open.into_iter().map(|w| resolve_way(w, &mut p)).collect();
    p.finish(|| HarvestDef { id, name: raw.name, glyph: raw.glyph, biome: raw.biome, uses: raw.uses, yields, open })
}

fn resolve_way(raw: RawWay, p: &mut Problems) -> Way {
    p.check(!raw.verb.trim().is_empty(), "open: verb is empty");
    p.check((1..=20).contains(&raw.difficulty), "open: difficulty must be 1 to 20");
    p.check(raw.fumble.hp <= 0, "open: fumble hp is a loss, so zero or negative");
    p.check(!raw.fumble.breaks || raw.tool.is_some(), "open: fumble breaks a tool, but the way has no `tool`");
    let tool = raw.tool.as_deref().and_then(|id| {
        let tool = Tool::lookup(id);
        p.check(tool.is_some(), format!("open: tool `{id}` is not a component or melee weapon"));
        tool
    });
    Way { verb: raw.verb, skill: raw.skill, difficulty: raw.difficulty, tool, fumble: raw.fumble }
}

fn resolve_yield(y: &RawYield, p: &mut Problems) -> Option<Yield> {
    p.check(y.min <= y.max, format!("yields: `{}` has min above max", y.item));
    p.check(y.max > 0, format!("yields: `{}` never yields anything (max 0)", y.item));
    let component = Component::lookup(&y.item);
    p.check(component.is_some(), format!("yields: `{}` is not a component", y.item));
    Some(Yield { component: component?, min: y.min, max: y.max })
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
    /// The way to work this node. Every node has exactly one for now.
    pub fn way(&self) -> &Way {
        &self.open[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
        name = "Toolbox"
        glyph = "🧰"
        biome = "ruins"
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
            assert!(Harvestable::all().any(|h| h.def().biome == biome), "{biome:?} has something to harvest");
        }
    }

    #[test]
    fn parses_yields_in_order() {
        let d = parse(GOOD).unwrap();
        assert_eq!(d.yields.len(), 2);
        assert_eq!(d.yields[0].component.def().id, "nails");
        assert_eq!((d.way().skill, d.way().difficulty, d.way().fumble.hp), (Skill::Scavenging, 9, -1));
    }

    #[test]
    fn tools_resolve_to_components_or_melee_weapons() {
        let with_tool = |id: &str| parse(&GOOD.replace("difficulty = 9", &format!("difficulty = 9\ntool = \"{id}\"")));
        assert!(matches!(with_tool("lockpicks").unwrap().way().tool, Some(Tool::Component(_))));
        assert!(matches!(with_tool("shovel").unwrap().way().tool, Some(Tool::Melee(_))));
        assert!(parse(GOOD).unwrap().way().tool.is_none());
    }

    #[test]
    fn rejects_bad_nodes() {
        let bad = |from: &str, to: &str| parse(&GOOD.replace(from, to)).unwrap_err();
        assert!(bad("\"nails\"", "\"moonbeams\"").contains("`moonbeams` is not a component"));
        assert!(bad("min = 1, max = 3", "min = 4, max = 3").contains("min above max"));
        assert!(bad("uses = 1", "uses = 0").contains("uses"));
        assert!(bad("difficulty = 9", "difficulty = 0").contains("difficulty"));
        assert!(bad("hp = -1", "hp = 2").contains("fumble hp"));
        assert!(bad("hp = -1", "breaks = true").contains("has no `tool`"));
        assert!(bad("difficulty = 9", "difficulty = 9\ntool = \"wishes\"").contains("tool `wishes`"));
        assert!(
            bad("[[open]]", "[[open]]\nverb = \"kick\"\nskill = \"melee\"\ndifficulty = 5\n[[open]]")
                .contains("exactly one")
        );
        assert!(parse(&GOOD.replace("biome = \"ruins\"", "biome = \"moon\"")).is_err());
        assert!(parse(&GOOD.replace("uses = 1", "uses = 1\ncolor = \"red\"")).is_err());
    }
}
