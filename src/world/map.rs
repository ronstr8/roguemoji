//! The level grid: tiles, features, and what the player has seen.

use std::collections::HashMap;

use super::feature::Feature;
use super::tile::Tile;
use crate::geometry::Pos;

#[derive(Clone, Debug)]
pub struct Map {
    width: i32,
    height: i32,
    tiles: Vec<Tile>,
    explored: Vec<bool>,
    visible: Vec<bool>,
    features: HashMap<Pos, Feature>,
}

impl Map {
    pub fn new(width: i32, height: i32, fill: Tile) -> Self {
        let n = (width * height) as usize;
        Self {
            width,
            height,
            tiles: vec![fill; n],
            explored: vec![false; n],
            visible: vec![false; n],
            features: HashMap::new(),
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn in_bounds(&self, p: Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    fn idx(&self, p: Pos) -> Option<usize> {
        self.in_bounds(p).then(|| (p.y * self.width + p.x) as usize)
    }

    /// Out-of-bounds reads as rock.
    pub fn tile(&self, p: Pos) -> Tile {
        self.idx(p).map_or(Tile::Rock, |i| self.tiles[i])
    }

    pub fn set_tile(&mut self, p: Pos, t: Tile) {
        if let Some(i) = self.idx(p) {
            self.tiles[i] = t;
        }
    }

    pub fn feature(&self, p: Pos) -> Option<&Feature> {
        self.features.get(&p)
    }

    pub fn feature_mut(&mut self, p: Pos) -> Option<&mut Feature> {
        self.features.get_mut(&p)
    }

    pub fn set_feature(&mut self, p: Pos, f: Feature) {
        self.features.insert(p, f);
    }

    pub fn remove_feature(&mut self, p: Pos) -> Option<Feature> {
        self.features.remove(&p)
    }

    /// Terrain and features allow standing here (ignores creatures).
    pub fn passable(&self, p: Pos) -> bool {
        self.tile(p).walkable() && !self.feature(p).is_some_and(|f| f.blocks())
    }

    pub fn opaque(&self, p: Pos) -> bool {
        self.tile(p).opaque()
    }

    pub fn is_visible(&self, p: Pos) -> bool {
        self.idx(p).is_some_and(|i| self.visible[i])
    }

    pub fn is_explored(&self, p: Pos) -> bool {
        self.idx(p).is_some_and(|i| self.explored[i])
    }

    pub fn clear_visible(&mut self) {
        self.visible.fill(false);
    }

    pub fn mark_visible(&mut self, p: Pos) {
        if let Some(i) = self.idx(p) {
            self.visible[i] = true;
            self.explored[i] = true;
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = Pos> + '_ {
        (0..self.height).flat_map(move |y| (0..self.width).map(move |x| Pos::new(x, y)))
    }

    pub fn walkable_cells(&self) -> Vec<Pos> {
        self.cells().filter(|p| self.tile(*p).walkable()).collect()
    }

    /// Breadth-first step distances from `start` over passable cells.
    pub fn distances_from(&self, start: Pos) -> Vec<Option<u32>> {
        let mut dist = vec![None; (self.width * self.height) as usize];
        let Some(si) = self.idx(start) else { return dist };
        dist[si] = Some(0);
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(p) = queue.pop_front() {
            let d = dist[self.idx(p).unwrap()].unwrap();
            for n in p.neighbors() {
                if let Some(ni) = self.idx(n)
                    && dist[ni].is_none()
                    && self.passable(n)
                {
                    dist[ni] = Some(d + 1);
                    queue.push_back(n);
                }
            }
        }
        dist
    }

    pub fn distance_at(&self, dist: &[Option<u32>], p: Pos) -> Option<u32> {
        self.idx(p).and_then(|i| dist[i])
    }
}
