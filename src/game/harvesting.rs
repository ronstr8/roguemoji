//! Harvest nodes and caches: spotting hidden ones, choosing a way in, and
//! what an attempt gathers or costs.

use super::{Event, Game, Tone};
use crate::geometry::Pos;
use crate::items::Tool;
use crate::stats::check::{self, Tier};
use crate::world::harvest::{HarvestDef, Test, Way};
use crate::world::{Cover, Feature};

impl Game {
    /// The harvest node at `pos`, found or not.
    pub fn node_at(&self, pos: Pos) -> Option<&'static HarvestDef> {
        match self.map.feature(pos) {
            Some(Feature::Harvest { kind, .. }) => Some(kind.def()),
            _ => None,
        }
    }

    /// Give each hidden node that just came into view its one look: spotted
    /// now, or overlooked for good.
    pub(super) fn look_for_hidden(&mut self) {
        for pos in self.map.unlooked_in_sight() {
            let Some(def) = self.node_at(pos) else { continue };
            let Some(hidden) = &def.hidden else { continue };
            let skill = self.player.skill(hidden.skill).total();
            if !check::roll(&mut self.dice, skill, hidden.target).tier.succeeded() {
                self.map.set_cover(pos, Cover::Overlooked);
                continue;
            }
            self.map.set_cover(pos, Cover::Found);
            let text = hidden.text.clone().unwrap_or_else(|| format!("You spot a {} {}.", def.glyph, def.name));
            self.log.push(text, Tone::Special);
            self.train(hidden.skill, 1);
        }
    }

    /// Walk into a node: a hidden one is found underfoot, then you try to open it.
    pub(super) fn bump_node(&mut self, pos: Pos) -> Event {
        let Some(def) = self.node_at(pos) else { return Event::None };
        if self.map.feature(pos).is_some_and(|f| f.is_hidden()) {
            self.map.set_cover(pos, Cover::Found);
            self.log.push(format!("You stumble onto a {} {}.", def.glyph, def.name), Tone::Special);
        }
        self.open_node(pos, def)
    }

    /// Try to open a node: with no way you have the tools for, say what it
    /// takes; with one, use it; with several, ask which.
    pub(super) fn open_node(&mut self, pos: Pos, def: &HarvestDef) -> Event {
        match def.ways_for(&self.player)[..] {
            [] => {
                self.log.info(format!("The {} won't open for you. It takes {}.", def.name, tools_needed(def)));
                Event::None
            }
            [way] => {
                self.harvest(pos, def, way);
                Event::None
            }
            _ => Event::ChooseWay(pos),
        }
    }

    /// Work the node at `pos` the way chosen from its pop-up.
    pub fn harvest_at(&mut self, pos: Pos, way: usize) {
        if let Some(def) = self.node_at(pos)
            && def.open.get(way).is_some_and(|w| w.ready(&self.player))
        {
            self.harvest(pos, def, way);
        }
    }

    /// One attempt at a node by its `way`th way, which you have the tools for.
    pub(super) fn harvest(&mut self, pos: Pos, def: &HarvestDef, way: usize) {
        let way = &def.open[way];
        let result = check::roll(&mut self.dice, way.modifier(&self.player), way.difficulty);
        let summary = result.summary();
        let used = match result.tier {
            Tier::Fumble => {
                self.fumble_harvest(def, way);
                true
            }
            Tier::Fail => {
                self.log.info(format!("You try to {} the {} but come up empty. {summary}", way.verb, def.name));
                false
            }
            tier => {
                let names = self.gather(def, tier);
                let spoiled = way.partial.as_ref().filter(|_| tier == Tier::Partial);
                let spoiled = spoiled.map(|t| format!(" {t}.")).unwrap_or_default();
                self.log.push(format!("You {} the {}: {names}.{spoiled} {summary}", way.verb, def.name), Tone::Loot);
                if let Test::Skill(s) = way.test {
                    self.train(s, if tier == Tier::Critical { 2 } else { 1 });
                }
                true
            }
        };
        self.apply_needs(&way.needs, false);
        if used {
            self.wear_node(pos, def);
        }
        self.pass_time(1);
    }

    /// A fumble costs the way's HP and may break its tool.
    fn fumble_harvest(&mut self, def: &HarvestDef, way: &Way) {
        self.player.hp += way.fumble.hp;
        let hurt = if way.fumble.hp < 0 { " and hurt yourself" } else { "" };
        let mut msg = format!("You fumble with the {}{hurt}.", def.name);
        if let Some(tool) = way.tool.filter(|_| way.fumble.breaks)
            && self.player.break_tool(tool)
        {
            msg.push_str(&format!(" Your {} {} breaks.", tool.glyph(), tool.name()));
        }
        self.log.push(msg, Tone::Bad);
    }

    /// Roll a successful harvest's yields into the pack; returns what you got,
    /// for the log. A partial stops at the first yield that comes up, and a
    /// critical adds one to each stack.
    fn gather(&mut self, def: &HarvestDef, tier: Tier) -> String {
        let mut rolled = Vec::new();
        for y in &def.yields {
            let n = self.dice.range(y.min as i32, y.max as i32) as u32;
            if n > 0 {
                let bonus = u32::from(tier == Tier::Critical && y.prize.stacks());
                rolled.push((y.prize, n + bonus));
                if tier == Tier::Partial {
                    break;
                }
            }
        }
        if rolled.is_empty() {
            let first = def.yields[0];
            rolled.push((first.prize, first.min.max(1)));
        }
        let mut names = Vec::new();
        for (prize, n) in rolled {
            for item in prize.make(n, &mut self.dice) {
                names.push(item.display_name());
                self.give_player(item);
            }
        }
        names.join(", ")
    }

    /// Use up one of a node's harvests, removing it when it's picked clean.
    fn wear_node(&mut self, pos: Pos, def: &HarvestDef) {
        if let Some(Feature::Harvest { uses_left, .. }) = self.map.feature_mut(pos) {
            *uses_left = uses_left.saturating_sub(1);
            if *uses_left == 0 {
                self.map.remove_feature(pos);
                self.log.info(format!("The {} is picked clean.", def.name));
            }
        }
    }
}

/// The tools a node's ways name, for the message when you have none of them:
/// "🪛 Lockpicks or 🦯 Crowbar".
fn tools_needed(def: &HarvestDef) -> String {
    let mut tools: Vec<Tool> = Vec::new();
    for tool in def.open.iter().filter_map(|w| w.tool) {
        if !tools.contains(&tool) {
            tools.push(tool);
        }
    }
    let names: Vec<String> = tools.iter().map(|t| format!("{} {}", t.glyph(), t.name())).collect();
    names.join(" or ")
}
