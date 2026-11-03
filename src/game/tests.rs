//! Whole-game soak tests: generate every depth and play randomly.

use super::*;
use crate::crafting::recipes;
use crate::creatures::Background;
use crate::creatures::player::{BASE_CAPACITY, Burden, CAPACITY_PER_BRAWN};
use crate::effects::{EffectKind, EffectSpec};
use crate::geometry::Dir;
use crate::items::{Component, ItemKind, Pack};
use crate::stats::Attribute;
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
                15 => {
                    if d.chance(0.5) {
                        g.craft(d.index(recipes().len()))
                    } else {
                        let from = if d.chance(0.5) { StashSide::Pack } else { StashSide::Lockbox };
                        g.stash_transfer(from, 0);
                    }
                }
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

/// Find a cell next to the player that is open floor, for movement tests.
fn open_step(g: &Game) -> Dir {
    *Dir::ALL
        .iter()
        .find(|d| {
            let t = g.player.pos.step(**d);
            g.is_free(t) && g.map.feature(t).is_none()
        })
        .expect("an open neighbor")
}

/// Load the pack with scrap until it weighs at least `lb` hundredths of a pound.
fn load_to(g: &mut Game, target: u32) {
    while g.player.carried_weight() < target {
        g.player.inventory.add(Item::component(Component::named("scrap_metal"), 1)).unwrap();
    }
}

#[test]
fn every_background_starts_light() {
    for bg in Background::ALL {
        let p = bg.create("Tester".into());
        assert_eq!(p.burden(), Burden::Light, "{} starts burdened", p.background);
    }
}

#[test]
fn brawn_and_packs_raise_capacity() {
    let mut g = new_game(1);
    let before = g.player.carry_capacity();
    g.player.effects.add(EffectSpec::new(EffectKind::Boost(Attribute::Brawn), 2, 50), "test");
    assert_eq!(g.player.carry_capacity(), before + 2 * CAPACITY_PER_BRAWN as u32);
    g.player.pack = Some(Pack::named("hiking_pack"));
    assert_eq!(
        g.player.carry_capacity(),
        before + 2 * CAPACITY_PER_BRAWN as u32 + Pack::named("hiking_pack").def().capacity
    );
}

#[test]
fn burden_tiers() {
    let cap = BASE_CAPACITY;
    assert_eq!(Burden::of(cap, cap), Burden::Light);
    assert_eq!(Burden::of(cap + 1, cap), Burden::Burdened);
    assert_eq!(Burden::of(cap * 3 / 2, cap), Burden::Burdened);
    assert_eq!(Burden::of(cap * 2, cap), Burden::Strained);
    assert_eq!(Burden::of(cap * 2 + 1, cap), Burden::Overloaded);
}

#[test]
fn heavy_loads_slow_and_then_stop_movement() {
    let mut g = new_game(3);
    g.monsters.clear();
    let cap = g.player.carry_capacity();

    load_to(&mut g, cap * 2 - 300);
    assert_eq!(g.player.burden(), Burden::Strained);
    let t = g.turn;
    let _ = g.move_player(open_step(&g));
    assert_eq!(g.turn - t, 2, "strained steps cost two turns");

    load_to(&mut g, cap * 2 + 1);
    assert_eq!(g.player.burden(), Burden::Overloaded);
    let (t, pos) = (g.turn, g.player.pos);
    let _ = g.move_player(open_step(&g));
    assert_eq!((g.turn, g.player.pos), (t, pos), "overloaded players can't move");
}

#[test]
fn burdened_steps_alternate_one_and_two_turns() {
    let mut g = new_game(4);
    g.monsters.clear();
    let cap = g.player.carry_capacity();
    load_to(&mut g, cap + 1);
    assert_eq!(g.player.burden(), Burden::Burdened);
    let t = g.turn;
    for _ in 0..2 {
        let d = open_step(&g);
        let _ = g.move_player(d);
    }
    assert_eq!(g.turn - t, 3, "two burdened steps cost three turns");
}

#[test]
fn stash_carries_over_between_depths() {
    let mut g = new_game(5);
    g.player.inventory.add(Item::one(ItemKind::Pack(Pack::named("frame_pack")))).unwrap();
    let idx = g.player.inventory.len() - 1;
    g.stash_transfer(StashSide::Pack, idx);
    assert_eq!(g.stash.len(), 1);
    g.enter_level(2);
    g.stash_transfer(StashSide::Lockbox, 0);
    assert!(g.stash.is_empty());
    assert!(g.player.inventory.items().iter().any(|i| i.kind == ItemKind::Pack(Pack::named("frame_pack"))));
}

#[test]
fn equipping_a_pack_swaps_the_old_one_back() {
    let mut g = new_game(6);
    g.player.inventory.add(Item::one(ItemKind::Pack(Pack::named("rag_satchel")))).unwrap();
    g.use_item(g.player.inventory.len() - 1);
    assert_eq!(g.player.pack, Some(Pack::named("rag_satchel")));
    g.player.inventory.add(Item::one(ItemKind::Pack(Pack::named("hiking_pack")))).unwrap();
    g.use_item(g.player.inventory.len() - 1);
    assert_eq!(g.player.pack, Some(Pack::named("hiking_pack")));
    assert!(g.player.inventory.items().iter().any(|i| i.kind == ItemKind::Pack(Pack::named("rag_satchel"))));
}

#[test]
fn selling_pays_crystals_and_takes_the_item() {
    use crate::items::{Armor, Item, MeleeWeapon};
    let mut g = new_game(3);
    let vest = Item::one(ItemKind::Armor(Armor::named("kevlar_vest")));
    let price = crate::items::loot::sell_price(&vest, g.depth).unwrap();
    g.player.inventory.add(vest.clone()).unwrap();
    let before = g.player.crystals;
    let idx = g.player.inventory.items().iter().position(|i| *i == vest).unwrap();
    g.sell(idx);
    assert_eq!(g.player.crystals, before + price);
    assert!(!g.player.inventory.items().contains(&vest));

    // Unpriced items stay put.
    let bar = Item::one(ItemKind::Melee(MeleeWeapon::named("crowbar")));
    g.player.inventory.add(bar.clone()).unwrap();
    let idx = g.player.inventory.items().iter().position(|i| *i == bar).unwrap();
    g.sell(idx);
    assert_eq!(g.player.crystals, before + price);
    assert!(g.player.inventory.items().contains(&bar));
}
