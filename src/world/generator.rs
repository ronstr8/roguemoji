//! Procedural terrain. Odd depths are ruined town blocks; even depths are
//! open wasteland carved by cellular automata.

use super::map::Map;
use super::tile::Tile;
use crate::dice::Dice;
use crate::geometry::{Pos, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Ruins,
    Wastes,
}

impl Biome {
    pub fn for_depth(depth: u32) -> Self {
        if depth % 2 == 1 { Biome::Ruins } else { Biome::Wastes }
    }

    pub fn name(self) -> &'static str {
        match self {
            Biome::Ruins => "Ruined Township",
            Biome::Wastes => "Open Wastes",
        }
    }
}

pub struct Terrain {
    pub map: Map,
    pub start: Pos,
    /// Building interiors, useful for placing loot.
    pub rooms: Vec<Rect>,
}

pub fn generate(dice: &mut Dice, biome: Biome, width: i32, height: i32) -> Terrain {
    let (mut map, rooms) = match biome {
        Biome::Ruins => ruins(dice, width, height),
        Biome::Wastes => (wastes(dice, width, height), Vec::new()),
    };
    let cells = map.walkable_cells();
    let start = loop {
        let p = *dice.pick(&cells);
        if map.tile(p) != Tile::Sludge && map.tile(p) != Tile::Water {
            break p;
        }
    };
    seal_unreachable(&mut map, start);
    Terrain { map, start, rooms }
}

fn border(map: &mut Map) {
    let (w, h) = (map.width(), map.height());
    for x in 0..w {
        map.set_tile(Pos::new(x, 0), Tile::Rock);
        map.set_tile(Pos::new(x, h - 1), Tile::Rock);
    }
    for y in 0..h {
        map.set_tile(Pos::new(0, y), Tile::Rock);
        map.set_tile(Pos::new(w - 1, y), Tile::Rock);
    }
}

fn ruins(dice: &mut Dice, w: i32, h: i32) -> (Map, Vec<Rect>) {
    let mut map = Map::new(w, h, Tile::Asphalt);
    // Weeds push through the streets.
    for p in map.cells().collect::<Vec<_>>() {
        if dice.chance(0.18) {
            map.set_tile(p, Tile::Grass);
        } else if dice.chance(0.06) {
            map.set_tile(p, Tile::Rubble);
        }
    }
    let mut rooms: Vec<Rect> = Vec::new();
    for _ in 0..200 {
        if rooms.len() >= 14 {
            break;
        }
        let rw = dice.range(5, 12);
        let rh = dice.range(4, 8);
        let r = Rect::new(dice.range(2, w - rw - 3), dice.range(2, h - rh - 3), rw, rh);
        if rooms.iter().any(|o| o.intersects_padded(&r)) {
            continue;
        }
        build(&mut map, dice, r);
        rooms.push(Rect::new(r.x + 1, r.y + 1, r.w - 2, r.h - 2));
    }
    border(&mut map);
    (map, rooms)
}

/// A walled building with doorways and partial collapse.
fn build(map: &mut Map, dice: &mut Dice, r: Rect) {
    let mut perimeter = Vec::new();
    for p in r.cells().collect::<Vec<_>>() {
        let edge = p.x == r.x || p.y == r.y || p.x == r.x + r.w - 1 || p.y == r.y + r.h - 1;
        if edge {
            map.set_tile(p, Tile::Wall);
            let corner = (p.x == r.x || p.x == r.x + r.w - 1) && (p.y == r.y || p.y == r.y + r.h - 1);
            if !corner {
                perimeter.push(p);
            }
        } else {
            map.set_tile(p, if dice.chance(0.1) { Tile::Rubble } else { Tile::Floor });
        }
    }
    let doors = dice.range(1, 3);
    for _ in 0..doors {
        let p = *dice.pick(&perimeter);
        map.set_tile(p, Tile::Floor);
    }
    // Collapsed wall sections.
    for p in perimeter {
        if dice.chance(0.07) {
            map.set_tile(p, Tile::Rubble);
        }
    }
}

fn wastes(dice: &mut Dice, w: i32, h: i32) -> Map {
    let mut map = Map::new(w, h, Tile::Grass);
    let mut rock: Vec<bool> = (0..w * h).map(|_| dice.chance(0.44)).collect();
    let at = |rock: &Vec<bool>, x: i32, y: i32| -> bool {
        if x < 0 || y < 0 || x >= w || y >= h { true } else { rock[(y * w + x) as usize] }
    };
    for _ in 0..5 {
        let mut next = rock.clone();
        for y in 0..h {
            for x in 0..w {
                let n = (-1..=1)
                    .flat_map(|dy| (-1..=1).map(move |dx| (dx, dy)))
                    .filter(|&(dx, dy)| (dx, dy) != (0, 0) && at(&rock, x + dx, y + dy))
                    .count();
                next[(y * w + x) as usize] = n >= 5 || (n == 0 && dice.chance(0.02));
            }
        }
        rock = next;
    }
    for y in 0..h {
        for x in 0..w {
            let p = Pos::new(x, y);
            let t = if rock[(y * w + x) as usize] {
                Tile::Rock
            } else if dice.chance(0.25) {
                Tile::Floor
            } else if dice.chance(0.04) {
                Tile::Rubble
            } else {
                Tile::Grass
            };
            map.set_tile(p, t);
        }
    }
    // Pools of water and toxic runoff.
    for i in 0..dice.range(3, 6) {
        let liquid = if i % 2 == 0 { Tile::Water } else { Tile::Sludge };
        let c = Pos::new(dice.range(3, w - 4), dice.range(3, h - 4));
        let radius = dice.range(1, 3);
        for y in -radius..=radius {
            for x in -radius..=radius {
                let p = c.offset(x, y);
                if x * x + y * y <= radius * radius && map.tile(p).walkable() {
                    map.set_tile(p, liquid);
                }
            }
        }
    }
    border(&mut map);
    map
}

/// Turn any walkable pocket the start can't reach into rock.
fn seal_unreachable(map: &mut Map, start: Pos) {
    let dist = map.distances_from(start);
    for p in map.cells().collect::<Vec<_>>() {
        if map.tile(p).walkable() && map.distance_at(&dist, p).is_none() {
            map.set_tile(p, Tile::Rock);
        }
    }
}
