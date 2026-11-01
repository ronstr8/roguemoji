//! Whole-game soak tests: generate every depth and play randomly.

use super::*;
use crate::crafting::RECIPES;
use crate::creatures::Background;
use crate::geometry::Dir;
use crate::world::Feature;

fn new_game(seed: u64) -> Game {
    let bg = Background::ALL[seed as usize % Background::ALL.len()];
    Game::new(bg.create("Tester".into()), Some(seed))
}

#[test]
fn exit_is_always_reachable() {
    for seed in 0..20 {
        let mut g = new_game(seed);
        for depth in 1..=FINAL_DEPTH {
            g.enter_level(depth);
            let dist = g.map.distances_from(g.player.pos);
            let exit = g
                .map
                .cells()
                .find(|p| matches!(g.map.feature(*p), Some(Feature::Stairs | Feature::Beacon)))
                .expect("level has an exit");
            // The exit isn't blocking, so it is reachable if any neighbor is.
            let reachable = exit.neighbors().any(|n| g.map.distance_at(&dist, n).is_some());
            assert!(reachable, "seed {seed} depth {depth}: exit unreachable");
            assert!(!g.monsters.is_empty(), "seed {seed} depth {depth}: no monsters");
        }
    }
}

#[test]
fn random_play_never_panics() {
    for seed in 0..8 {
        let mut g = new_game(seed);
        let mut d = Dice::seeded(seed + 1000);
        for step in 0..2500u32 {
            if g.is_over() {
                break;
            }
            // Keep the tester alive so deep content gets exercised.
            g.player.hp = g.player.max_hp();
            match d.range(0, 19) {
                0..=10 => {
                    let _ = g.move_player(*d.pick(&Dir::ALL));
                }
                11 => g.pickup(),
                12 => {
                    if let Some(t) = g.targets().first() {
                        g.fire_at(*t);
                    }
                }
                13 => g.reload(),
                14 => {
                    let n = g.player.inventory.len();
                    if n > 0 {
                        g.use_item(d.index(n));
                    }
                }
                15 => g.craft(d.index(RECIPES.len())),
                16 => g.absorb(true),
                17 => g.cycle_stance(),
                _ => {
                    if !g.kiosks.is_empty() && !g.kiosks[0].stock.is_empty() {
                        g.player.crystals += 100;
                        g.buy(0, 0);
                    } else {
                        g.wait();
                    }
                }
            }
            if step % 250 == 249 && g.depth < FINAL_DEPTH {
                g.enter_level(g.depth + 1);
            }
        }
        assert!(g.depth >= 2, "seed {seed}: never left depth 1");
    }
}
