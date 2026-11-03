//! Game state and the rules that drive it.
//!
//! `Game` owns the world. Its behavior is split by concern across sibling
//! modules, each adding an `impl Game` block:
//! * `level`   — building and populating each depth
//! * `turn`    — the passage of time: effects, hunger, monster turns
//! * `ai`      — monster decision-making and attacks
//! * `actions` — everything the player can do

mod actions;
mod ai;
mod level;
pub mod log;
#[cfg(test)]
mod tests;
mod turn;

pub use actions::{Event, StashSide};
pub use log::{Log, Tone};

use crate::creatures::player::Burden;
use crate::creatures::{Monster, Player};
use crate::dice::Dice;
use crate::geometry::Pos;
use crate::items::{Inventory, Item};
use crate::world::{Biome, Map, Tile, fov};

pub const MAP_WIDTH: i32 = 72;
pub const MAP_HEIGHT: i32 = 40;
pub const FINAL_DEPTH: u32 = 10;
/// Stacks the shared lockbox stash can hold.
pub const STASH_SLOTS: usize = 20;

#[derive(Clone, Debug)]
pub struct FloorItem {
    pub pos: Pos,
    pub item: Item,
}

#[derive(Clone, Debug, Default)]
pub struct Kiosk {
    pub stock: Vec<(Item, u32)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Died(String),
    Won,
}

pub struct Game {
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub items: Vec<FloorItem>,
    pub kiosks: Vec<Kiosk>,
    pub map: Map,
    pub biome: Biome,
    pub depth: u32,
    pub turn: u64,
    pub log: Log,
    pub outcome: Option<Outcome>,
    /// Items left in the Visitor lockbox network, reachable from any lockbox.
    pub stash: Inventory,
    /// Burden as of the last check, to announce changes.
    last_burden: Burden,
    dice: Dice,
    next_id: u32,
}

impl Game {
    pub fn new(player: Player, seed: Option<u64>) -> Self {
        let dice = seed.map_or_else(Dice::from_entropy, Dice::seeded);
        let mut game = Self {
            player,
            monsters: Vec::new(),
            items: Vec::new(),
            kiosks: Vec::new(),
            map: Map::new(1, 1, Tile::Rock),
            biome: Biome::Ruins,
            depth: 0,
            turn: 0,
            log: Log::default(),
            outcome: None,
            stash: Inventory::with_slots(STASH_SLOTS),
            last_burden: Burden::Light,
            dice,
            next_id: 1,
        };
        game.log.push("The Visitors came, took nothing, and left. Everything since has been strange.", Tone::Special);
        game.log.push(
            format!("Their Beacon still hums somewhere below depth {}. Find it.", FINAL_DEPTH - 1),
            Tone::Special,
        );
        game.last_burden = game.player.burden();
        game.enter_level(1);
        game.log.info("Press ? for help.");
        game
    }

    pub fn is_over(&self) -> bool {
        self.outcome.is_some()
    }

    pub fn monster_index_at(&self, p: Pos) -> Option<usize> {
        self.monsters.iter().position(|m| m.pos == p && !m.is_dead())
    }

    pub fn monster(&self, id: u32) -> Option<&Monster> {
        self.monsters.iter().find(|m| m.id == id && !m.is_dead())
    }

    /// Cell is passable and not occupied by any creature.
    pub fn is_free(&self, p: Pos) -> bool {
        self.map.passable(p) && self.player.pos != p && self.monster_index_at(p).is_none()
    }

    pub fn items_at(&self, p: Pos) -> impl Iterator<Item = &FloorItem> {
        self.items.iter().filter(move |fi| fi.pos == p)
    }

    /// Visible living monsters, nearest first.
    pub fn visible_monsters(&self) -> Vec<&Monster> {
        let mut v: Vec<&Monster> =
            self.monsters.iter().filter(|m| !m.is_dead() && self.map.is_visible(m.pos)).collect();
        v.sort_by_key(|m| m.pos.distance(self.player.pos));
        v
    }

    fn refresh_fov(&mut self) {
        fov::compute(&mut self.map, self.player.pos, fov::SIGHT_RADIUS);
        for m in &mut self.monsters {
            if !m.seen && !m.is_dead() && self.map.is_visible(m.pos) {
                m.seen = true;
                let def = m.def();
                self.log.push(
                    format!("You spot a {} {} ({}). {}", def.glyph, def.name, def.family.label(), def.desc),
                    Tone::Info,
                );
            }
        }
    }

    fn next_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    /// Drop an item on the floor, merging stacks on the same cell.
    fn drop_at(&mut self, pos: Pos, item: Item) {
        if item.stackable()
            && let Some(fi) = self.items.iter_mut().find(|fi| fi.pos == pos && fi.item.kind == item.kind)
        {
            fi.item.qty += item.qty;
            return;
        }
        self.items.push(FloorItem { pos, item });
    }

    /// Announce when the player's burden tier changes.
    fn note_burden(&mut self) {
        let now = self.player.burden();
        if now == self.last_burden {
            return;
        }
        let tone = if now > self.last_burden { Tone::Bad } else { Tone::Good };
        let load = format!(
            "{}/{}",
            crate::items::weight::fmt_lb(self.player.carried_weight()),
            crate::items::weight::fmt_lb(self.player.carry_capacity())
        );
        self.log.push(format!("You are now {} ({load}). {}", now.name(), now.describe()), tone);
        self.last_burden = now;
    }

    /// Put an item in the pack, or at the player's feet if the pack is full.
    fn give_player(&mut self, item: Item) {
        if let Err(item) = self.player.inventory.add(item) {
            self.log.push(format!("Your pack is full; {} falls at your feet.", item.display_name()), Tone::Bad);
            let pos = self.player.pos;
            self.drop_at(pos, item);
        }
    }
}
