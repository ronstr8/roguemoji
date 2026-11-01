//! Random number source with tabletop-style helpers.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub struct Dice {
    rng: StdRng,
}

impl Dice {
    pub fn seeded(seed: u64) -> Self {
        Self { rng: StdRng::seed_from_u64(seed) }
    }

    pub fn from_entropy() -> Self {
        Self { rng: StdRng::from_entropy() }
    }

    /// Uniform integer in `lo..=hi` (order-insensitive).
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let (lo, hi) = if lo <= hi { (lo, hi) } else { (hi, lo) };
        self.rng.gen_range(lo..=hi)
    }

    /// Uniform float in `lo..hi`.
    pub fn float(&mut self, lo: f64, hi: f64) -> f64 {
        if hi <= lo {
            return lo;
        }
        self.rng.gen_range(lo..hi)
    }

    /// Roll `n` dice with `sides` faces and sum them.
    pub fn roll(&mut self, n: u32, sides: i32) -> i32 {
        (0..n).map(|_| self.range(1, sides)).sum()
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.rng.gen_bool(p.clamp(0.0, 1.0))
    }

    pub fn index(&mut self, len: usize) -> usize {
        self.rng.gen_range(0..len)
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.index(items.len())]
    }

    /// Pick from `(value, weight)` pairs; `None` if all weights are zero.
    pub fn weighted<T: Copy>(&mut self, items: &[(T, u32)]) -> Option<T> {
        let total: u32 = items.iter().map(|(_, w)| *w).sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.rng.gen_range(0..total);
        for (item, w) in items {
            if roll < *w {
                return Some(*item);
            }
            roll -= w;
        }
        None
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.rng.gen_range(0..=i);
            items.swap(i, j);
        }
    }
}
