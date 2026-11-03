//! Building and populating a level.

use super::{FINAL_DEPTH, Game, Kiosk, MAP_HEIGHT, MAP_WIDTH, Tone};
use crate::creatures::{Family, Monster, Species};
use crate::geometry::Pos;
use crate::items::loot;
use crate::world::generator::{self, Biome};
use crate::world::{Feature, Harvestable, Tile};

impl Game {
    pub(super) fn enter_level(&mut self, depth: u32) {
        self.depth = depth;
        self.biome = Biome::for_depth(depth);
        let terrain = generator::generate(&mut self.dice, self.biome, MAP_WIDTH, MAP_HEIGHT);
        self.map = terrain.map;
        self.player.pos = terrain.start;
        self.monsters.clear();
        self.items.clear();
        self.kiosks.clear();

        let start = terrain.start;
        let dist = self.map.distances_from(start);
        let width = self.map.width();
        let dist_of = |p: Pos| dist[(p.y * width + p.x) as usize].unwrap_or(0);

        // The way down (or the Beacon) goes as far from the start as possible.
        let exit = self
            .map
            .walkable_cells()
            .into_iter()
            .filter(|p| !is_liquid(self.map.tile(*p)))
            .max_by_key(|p| dist_of(*p))
            .unwrap_or(start);
        let exit_feature = if depth >= FINAL_DEPTH { Feature::Beacon } else { Feature::Stairs };
        self.map.set_feature(exit, exit_feature);

        // Blocking features only go in open ground so they never seal a passage.
        let mut open: Vec<Pos> = self
            .map
            .walkable_cells()
            .into_iter()
            .filter(|p| {
                !is_liquid(self.map.tile(*p))
                    && dist_of(*p) >= 2
                    && *p != exit
                    && p.neighbors().all(|n| self.map.tile(n).walkable() && self.map.feature(n).is_none())
            })
            .collect();
        self.dice.shuffle(&mut open);

        let (primary, secondary): (&[Harvestable], &[Harvestable]) = match self.biome {
            Biome::Ruins => (&Harvestable::URBAN, &Harvestable::NATURAL),
            Biome::Wastes => (&Harvestable::NATURAL, &Harvestable::URBAN),
        };
        let n_harvest = 14 + self.dice.range(0, 6) as usize;
        for _ in 0..n_harvest {
            let Some(p) = take_isolated(&mut open, &self.map) else { break };
            let pool = if self.dice.chance(0.75) { primary } else { secondary };
            let kind = *self.dice.pick(pool);
            self.map.set_feature(p, Feature::harvest(kind));
        }

        if (depth == 1 || self.dice.chance(0.6))
            && let Some(p) = take_isolated(&mut open, &self.map)
        {
            let stock = loot::kiosk_stock(&mut self.dice, depth);
            self.kiosks.push(Kiosk { stock });
            self.map.set_feature(p, Feature::Kiosk(self.kiosks.len() - 1));
        }

        // Visitor lockboxes all share one stash, so gear left on one depth
        // can be picked up on another.
        if (depth == 1 || self.dice.chance(0.5))
            && depth < FINAL_DEPTH
            && let Some(p) = take_isolated(&mut open, &self.map)
        {
            self.map.set_feature(p, Feature::Lockbox);
        }

        // Loot, favoring building interiors in the ruins.
        let floor: Vec<Pos> = self
            .map
            .walkable_cells()
            .into_iter()
            .filter(|p| !is_liquid(self.map.tile(*p)) && self.map.feature(*p).is_none() && *p != start)
            .collect();
        let n_items = 6 + depth as usize / 2 + self.dice.range(0, 3) as usize;
        for _ in 0..n_items {
            let p = if !terrain.rooms.is_empty() && self.dice.chance(0.6) {
                let room = *self.dice.pick(&terrain.rooms);
                let cells: Vec<Pos> = room.cells().filter(|c| floor.contains(c)).collect();
                if cells.is_empty() { *self.dice.pick(&floor) } else { *self.dice.pick(&cells) }
            } else {
                *self.dice.pick(&floor)
            };
            let item = loot::random_floor_item(&mut self.dice, depth);
            self.drop_at(p, item);
        }

        // Monsters keep their distance from the arrival point.
        let lairs: Vec<Pos> = floor.iter().copied().filter(|p| dist_of(*p) >= 7).collect();
        let n_monsters = 6 + (depth as usize * 3) / 2 + self.dice.range(0, 3) as usize;
        for _ in 0..n_monsters {
            if lairs.is_empty() {
                break;
            }
            let p = *self.dice.pick(&lairs);
            if self.monster_index_at(p).is_some() {
                continue;
            }
            let species = self.pick_species(depth);
            let id = self.next_id();
            self.monsters.push(Monster::new(id, species, p, depth));
        }

        self.log.push(format!("— Depth {}: {} —", depth, self.biome.name()), Tone::Special);
        if depth >= FINAL_DEPTH {
            self.log.push("The air tastes of copper. The Beacon 📡 is near.", Tone::Special);
        }
        self.refresh_fov();
    }

    fn pick_species(&mut self, depth: u32) -> Species {
        let table: Vec<(Species, u32)> = Species::ALL
            .iter()
            .filter(|s| s.def().min_depth <= depth)
            .map(|s| {
                let def = s.def();
                let age = depth - def.min_depth;
                let mut w = def.rarity * 3;
                if age > 4 {
                    w /= 3; // trash mobs thin out in the deep
                }
                if def.family == Family::Cryptid {
                    w += depth / 3;
                }
                (*s, w.max(1))
            })
            .collect();
        self.dice.weighted(&table).unwrap_or(Species::Rat)
    }
}

fn is_liquid(t: Tile) -> bool {
    matches!(t, Tile::Water | Tile::Sludge)
}

/// Pop a candidate cell that still has no feature in its neighborhood.
fn take_isolated(open: &mut Vec<Pos>, map: &crate::world::Map) -> Option<Pos> {
    while let Some(p) = open.pop() {
        if map.feature(p).is_none() && p.neighbors().all(|n| map.feature(n).is_none()) {
            return Some(p);
        }
    }
    None
}
