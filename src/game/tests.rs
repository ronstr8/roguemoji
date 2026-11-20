//! Whole-game soak tests: generate every depth and play randomly.

use super::*;
use crate::crafting::recipes;
use crate::creatures::Background;
use crate::creatures::player::{BASE_CAPACITY, Burden, CAPACITY_PER_BRAWN};
use crate::effects::{EffectKind, EffectSpec};
use crate::geometry::Dir;
use crate::items::Item;
use crate::items::{Component, ItemKind, Pack};
use crate::stats::Attribute;
use crate::world::{Cover, Feature, Tile};

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
                .find(|p| matches!(g.map.feature(*p), Some(Feature::Portal | Feature::Beacon)))
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
            // Asleep, turns pass on their own, as in the app.
            if g.player.is_asleep() {
                g.sleep_turn();
                continue;
            }
            // Keep the tester alive so deep content gets exercised.
            g.player.hp = g.player.max_hp();
            match d.range(0, 19) {
                0..=10 => {
                    let _ = g.move_player(*d.pick(&Dir::ALL));
                }
                11 => g.pickup(),
                12 => {
                    let reach = g.player.gun.as_ref().and_then(|gun| gun.reach()).unwrap_or(i32::MAX);
                    for t in g.targets() {
                        let dist = g.monster(t).unwrap().pos.distance(g.player.pos);
                        assert!(dist <= reach, "target at {dist} tiles, gun reaches {reach}");
                    }
                    if let Some(t) = g.shot_targets().first() {
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
fn lighter_limit_is_the_top_of_the_next_level() {
    let cap = 3001; // odd, so 1.5x rounds
    assert_eq!(Burden::Light.lighter(cap), None);
    for b in [Burden::Burdened, Burden::Strained, Burden::Overloaded] {
        let (lighter, limit) = b.lighter(cap).unwrap();
        assert_eq!(Burden::of(limit, cap), lighter, "{b:?}");
        assert_eq!(Burden::of(limit + 1, cap), b, "{b:?}");
    }
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
    let bar = Item::one(ItemKind::Melee(MeleeWeapon::named("claw_hammer")));
    g.player.inventory.add(bar.clone()).unwrap();
    let idx = g.player.inventory.items().iter().position(|i| *i == bar).unwrap();
    g.sell(idx);
    assert_eq!(g.player.crystals, before + price);
    assert!(g.player.inventory.items().contains(&bar));
}

/// A game with no monsters, so passing time only drains needs.
fn quiet_game(seed: u64) -> Game {
    let mut g = new_game(seed);
    g.monsters.clear();
    g
}

fn add_and_use(g: &mut Game, item: Item) {
    g.player.inventory.add(item.clone()).unwrap();
    let idx = g.player.inventory.items().iter().position(|i| *i == item).unwrap();
    g.use_item(idx);
}

#[test]
fn thirst_drains_faster_than_hunger() {
    let mut g = quiet_game(1);
    let (hunger, thirst) = (g.player.hunger.value, g.player.thirst.value);
    g.pass_time(50);
    let hunger_lost = hunger - g.player.hunger.value;
    let thirst_lost = thirst - g.player.thirst.value;
    assert!(hunger_lost > 0 && thirst_lost > hunger_lost, "hunger -{hunger_lost}, thirst -{thirst_lost}");
}

#[test]
fn parched_dulls_the_mind_and_hurts() {
    let mut g = quiet_game(2);
    let brains = g.player.attr(Attribute::Brains);
    g.player.thirst.value = 0;
    assert!(g.player.attr(Attribute::Brains) < brains);
    let hp = g.player.hp;
    g.pass_time(30);
    assert!(g.player.hp < hp, "going without water costs HP");
}

#[test]
fn water_quenches_thirst_and_moonshine_dries_you_out() {
    use crate::items::Consumable;
    let mut g = quiet_game(3);
    g.player.thirst.value = 100;
    let hunger = g.player.hunger.value;
    add_and_use(&mut g, Item::food(Consumable::named("bottled_water"), 1));
    assert!(g.player.thirst.value > 400);
    assert!(g.player.hunger.value < hunger, "water doesn't feed you; a turn passed");
    let thirst = g.player.thirst.value;
    add_and_use(&mut g, Item::food(Consumable::named("moonshine"), 1));
    assert!(g.player.thirst.value < thirst - g.player.thirst.def().drain);
}

#[test]
fn dirty_water_can_be_drunk_from_the_pack() {
    let mut g = quiet_game(4);
    g.player.thirst.value = 100;
    let water = Item::component(Component::named("dirty_water"), 1);
    add_and_use(&mut g, water.clone());
    assert!(g.player.thirst.value > 400);
    assert!(!g.player.inventory.items().contains(&water));
    // Components without `drink` stay put.
    let scrap = Item::component(Component::named("scrap_metal"), 1);
    add_and_use(&mut g, scrap.clone());
    assert!(g.player.inventory.items().contains(&scrap));
}

#[test]
fn drinking_needs_water_nearby_and_sometimes_sickens() {
    let mut g = quiet_game(5);
    let pos = g.player.pos;
    for p in std::iter::once(pos).chain(pos.neighbors()) {
        if g.map.tile(p) == Tile::Water {
            g.map.set_tile(p, Tile::Floor);
        }
    }
    let turn = g.turn;
    g.drink_here();
    assert_eq!(g.turn, turn, "no water, no time spent");

    g.map.set_tile(pos.step(Dir::E), Tile::Water);
    let mut sick = 0;
    for _ in 0..40 {
        g.player.thirst.value = 100;
        g.player.effects = Default::default();
        g.drink_here();
        assert!(g.player.thirst.value > 400);
        sick += usize::from(g.player.effects.has(EffectKind::Poison));
    }
    assert!(sick > 0 && sick < 40, "{sick} of 40 drinks made you sick");
}

#[test]
fn unequipping_moves_gear_into_the_pack() {
    use crate::creatures::player::Slot;
    let mut g = quiet_game(6);
    let gun = g.player.equipped(Slot::Gun).expect("starts with a gun");
    g.unequip(Slot::Gun);
    assert!(g.player.gun.is_none());
    assert!(g.player.inventory.items().contains(&gun), "the gun keeps its loaded rounds");
    g.unequip(Slot::Gun);
    assert_eq!(g.player.inventory.items().iter().filter(|i| **i == gun).count(), 1, "an empty slot gives nothing");
}

/// Sleep until awake, as the app does, up to `cap` turns. Returns turns slept.
fn sleep_it_off(g: &mut Game, cap: u32) -> u32 {
    let mut turns = 0;
    while g.player.is_asleep() && turns < cap {
        g.sleep_turn();
        turns += 1;
    }
    turns
}

/// Put a monster next to the player, where it can see and reach them.
/// Returns the direction it's in.
fn monster_beside(g: &mut Game) -> Dir {
    let mut fresh = new_game(1);
    let dir = open_step(g);
    let mut m = fresh.monsters.remove(0);
    m.pos = g.player.pos.step(dir);
    g.monsters.push(m);
    g.refresh_fov();
    dir
}

fn logged(g: &Game, text: &str) -> bool {
    g.log.recent(200).iter().any(|m| m.text.contains(text))
}

#[test]
fn running_out_of_rest_knocks_you_out_until_wake_at() {
    use crate::creatures::player::Sleep;
    let mut g = quiet_game(7);
    g.player.rest.value = 1;
    g.wait();
    assert_eq!(g.player.sleep, Some(Sleep::Collapse));
    let slept = sleep_it_off(&mut g, 500);
    assert!(!g.player.is_asleep(), "woke within {slept} turns");
    assert!(g.player.rest.value >= g.player.sleep_def().wake_at);
    assert!(g.player.rest.value < g.player.rest.def().max, "a collapse doesn't leave you fully rested");
}

#[test]
fn a_nap_rests_you_fully_and_heals_a_little() {
    use crate::creatures::player::Sleep;
    let mut g = quiet_game(8);
    g.player.rest.value = 500;
    g.player.hp = g.player.max_hp() - 5;
    let (hp, thirst) = (g.player.hp, g.player.thirst.value);
    g.nap();
    assert_eq!(g.player.sleep, Some(Sleep::Nap));
    sleep_it_off(&mut g, 500);
    assert!(!g.player.is_asleep());
    assert_eq!(g.player.rest.value, g.player.rest.def().max);
    assert!(g.player.hp > hp, "sleep heals");
    assert!(g.player.thirst.value < thirst, "time asleep still costs water");
}

#[test]
fn naps_need_quiet_and_end_on_a_key_or_a_monster() {
    let mut g = quiet_game(9);
    g.player.rest.value = g.player.rest.def().max;
    g.nap();
    assert!(!g.player.is_asleep(), "fully rested: no nap");
    g.player.rest.value = 300;
    g.nap();
    g.rouse();
    assert!(!g.player.is_asleep(), "a key ends a nap");

    g.nap();
    let _ = monster_beside(&mut g);
    g.sleep_turn();
    assert!(!g.player.is_asleep(), "a monster in view ends a nap");
    g.nap();
    assert!(!g.player.is_asleep(), "can't nap with a monster in sight");
}

#[test]
fn a_hit_wakes_even_a_collapse() {
    let mut g = quiet_game(10);
    g.player.rest.value = 1;
    g.wait();
    let _ = monster_beside(&mut g);
    g.rouse();
    assert!(g.player.is_asleep(), "keys don't wake you from a collapse");
    sleep_it_off(&mut g, 200);
    assert!(!g.player.is_asleep());
    assert!(logged(&g, "Pain jolts you awake!"), "a monster beside you lands a hit long before you'd come to");
}

#[test]
fn fighting_tires_you_faster_than_waiting() {
    let mut g = quiet_game(11);
    g.player.rest.value = 500;
    g.wait();
    let calm = 500 - g.player.rest.value;
    let dir = monster_beside(&mut g);
    g.player.rest.value = 500;
    let _ = g.move_player(dir);
    let fighting = 500 - g.player.rest.value;
    assert!(fighting > calm, "fighting -{fighting}, waiting -{calm}");
}

/// Put an encounter on the open cell next to the player; returns its direction.
fn encounter_beside(g: &mut Game, id: &str) -> Dir {
    use crate::world::encounter::Encounter;
    let dir = open_step(g);
    g.map.set_feature(g.player.pos.step(dir), Feature::encounter(Encounter::named(id)));
    dir
}

#[test]
fn bumping_a_harvest_node_gathers_until_its_picked_clean() {
    use crate::world::Harvestable;
    let mut g = quiet_game(13);
    let dir = open_step(&g);
    let spot = g.player.pos.step(dir);
    let toolbox = Harvestable::named("toolbox");
    g.map.set_feature(spot, Feature::harvest(toolbox));
    let held = |g: &Game| -> u32 {
        use crate::items::loot::Prize;
        let parts = toolbox.def().yields.iter().filter_map(|y| match y.prize {
            Prize::Component(c) => Some(c),
            _ => None,
        });
        parts.map(|c| g.player.inventory.count_component(c)).sum()
    };
    let before = held(&g);
    for _ in 0..50 {
        if g.map.feature(spot).is_none() {
            break;
        }
        g.player.hp = g.player.max_hp();
        assert_eq!(g.move_player(dir), Event::None);
        assert_ne!(g.player.pos, spot, "you bump a node, not walk onto it");
    }
    assert!(g.map.feature(spot).is_none(), "a one-use toolbox is picked clean");
    assert!(held(&g) > before, "and its components are in your pack");
}

/// A one-use lock beside the player that only lockpicks open, and always
/// fumbles or always succeeds depending on `difficulty`.
fn lock_beside(g: &mut Game, difficulty: i32) -> (Pos, crate::world::harvest::HarvestDef) {
    let dir = open_step(g);
    let spot = g.player.pos.step(dir);
    let def = crate::world::harvest::parse(&format!(
        r#"
        name = "Strongbox"
        glyph = "🔒"
        biome = "ruins"
        chance = {{ weight = 1 }}
        uses = 1
        yields = [{{ item = "nails", min = 1, max = 1 }}]
        [[open]]
        verb = "pick"
        skill = "lockpicking"
        difficulty = {difficulty}
        tool = "lockpicks"
        fumble = {{ hp = 0, breaks = true }}
        "#
    ))
    .unwrap();
    (spot, def)
}

#[test]
fn a_way_that_needs_a_tool_waits_for_one() {
    let mut g = quiet_game(14);
    let picks = Item::component(Component::named("lockpicks"), 1);
    while g.player.inventory.consume(&picks.kind, 1) {}
    let (spot, def) = lock_beside(&mut g, 1);
    let turn = g.turn;
    assert_eq!(g.open_node(spot, &def), Event::None);
    assert_eq!(g.turn, turn, "no tool, no time spent");
    assert!(g.log.recent(1)[0].text.contains("won't open for you. It takes 🪛 Lockpicks."));

    g.player.inventory.add(picks.clone()).unwrap();
    let nails = g.player.inventory.count_component(Component::named("nails"));
    assert_eq!(g.open_node(spot, &def), Event::None, "one way open, so it's used straight away");
    assert!(g.turn > turn);
    assert!(g.player.inventory.count_component(Component::named("nails")) > nails, "picked it");
    assert_eq!(g.player.inventory.count(&picks.kind), 1, "and the picks aren't used up");
}

#[test]
fn a_wielded_tool_counts_and_breaks_before_a_spare() {
    use crate::items::{MeleeWeapon, Tool};
    let mut g = new_game(0);
    let bar = MeleeWeapon::named("crowbar");
    let tool = Tool::Melee(bar);
    g.player.melee = Some(bar);
    g.player.inventory.add(Item::one(ItemKind::Melee(bar))).unwrap();
    assert!(g.player.has_tool(tool));
    assert!(g.player.break_tool(tool));
    assert_eq!(g.player.melee, None, "the one in hand breaks");
    assert!(g.player.has_tool(tool), "the spare is still in the pack");
    assert!(g.player.break_tool(tool));
    assert!(!g.player.has_tool(tool) && !g.player.break_tool(tool));
}

#[test]
fn a_fumble_can_break_the_tool() {
    let mut g = quiet_game(15);
    let picks = Item::component(Component::named("lockpicks"), 1);
    g.player.inventory.add(Item::component(Component::named("lockpicks"), 5)).unwrap();
    let before = g.player.inventory.count(&picks.kind);
    let (spot, def) = lock_beside(&mut g, 20);
    let mut broke = 0;
    for _ in 0..40 {
        let had = g.player.inventory.count(&picks.kind);
        g.harvest(spot, &def, 0);
        broke += had - g.player.inventory.count(&picks.kind);
    }
    assert!(broke > 0, "a hard lock fumbles now and then, snapping picks");
    assert_eq!(g.player.inventory.count(&picks.kind), before - broke);
}

/// Put a stock cache on the open cell next to the player, hidden and not yet
/// looked at; returns its direction and position.
fn cache_beside(g: &mut Game, id: &str) -> (Dir, Pos) {
    use crate::world::Harvestable;
    let dir = open_step(g);
    let pos = g.player.pos.step(dir);
    g.map.set_feature(pos, Feature::harvest(Harvestable::named(id)));
    (dir, pos)
}

fn cover_at(g: &Game, pos: Pos) -> Option<Cover> {
    match g.map.feature(pos) {
        Some(Feature::Harvest { cover, .. }) => Some(*cover),
        _ => None,
    }
}

#[test]
fn a_hidden_cache_is_neither_drawn_nor_in_the_way() {
    let mut g = quiet_game(16);
    let (_, pos) = cache_beside(&mut g, "military_crate");
    assert_eq!(cover_at(&g, pos), Some(Cover::Hidden));
    assert!(g.map.shown_feature(pos).is_none());
    assert!(g.map.passable(pos), "you could walk right over it");
}

#[test]
fn a_hidden_cache_gets_one_look_when_it_comes_into_view() {
    let mut spotted = 0;
    for seed in 0..40 {
        let mut g = quiet_game(seed);
        let (_, pos) = cache_beside(&mut g, "military_crate");
        g.map.mark_visible(pos);
        g.look_for_hidden();
        let first = cover_at(&g, pos).unwrap();
        assert_ne!(first, Cover::Hidden, "seed {seed}: the look is taken");
        for _ in 0..10 {
            g.look_for_hidden();
        }
        assert_eq!(cover_at(&g, pos), Some(first), "seed {seed}: and never retaken");
        spotted += usize::from(first == Cover::Found);
    }
    assert!(spotted > 0 && spotted < 40, "spotted {spotted} of 40");
}

#[test]
fn walking_onto_a_hidden_cache_finds_it_and_says_what_it_takes() {
    let mut g = quiet_game(4);
    g.player.melee = None;
    let picks = Item::component(Component::named("lockpicks"), 1);
    while g.player.inventory.consume(&picks.kind, 1) {}
    let (dir, pos) = cache_beside(&mut g, "military_crate");
    g.map.set_cover(pos, Cover::Overlooked);
    let (start, turn) = (g.player.pos, g.turn);
    assert_eq!(g.move_player(dir), Event::None);
    assert_eq!(g.player.pos, start, "the find stops you");
    assert_eq!(g.turn, turn, "and a locked crate with no tools takes no time");
    assert_eq!(cover_at(&g, pos), Some(Cover::Found));
    let log: Vec<&str> = g.log.recent(2).iter().map(|m| m.text.as_str()).collect();
    assert!(log[0].contains("You stumble onto a 🪖 Military Crate."), "{log:?}");
    assert!(log[1].contains("It takes 🪛 Lockpicks or 🦯 Crowbar."), "{log:?}");
    assert!(!g.map.passable(pos), "found, it's in the way like any crate");
}

#[test]
fn two_ways_in_means_choosing_one() {
    let mut g = quiet_game(8);
    assert!(g.player.melee.is_some_and(|m| m.def().id == "crowbar"), "a Scavenger");
    let (dir, pos) = cache_beside(&mut g, "military_crate");
    g.map.set_cover(pos, Cover::Found);
    assert_eq!(g.move_player(dir), Event::ChooseWay(pos));
    let turn = g.turn;
    g.harvest_at(pos, 1);
    assert!(g.turn > turn, "prying takes a turn");
}

#[test]
fn digging_up_a_cache_is_tiring() {
    let mut g = quiet_game(17);
    g.player.inventory.add(Item::one(ItemKind::Melee(crate::items::MeleeWeapon::named("shovel")))).unwrap();
    let (_, pos) = cache_beside(&mut g, "buried_cache");
    let def = g.node_at(pos).unwrap();
    let before = g.player.rest.value;
    g.harvest(pos, def, 0);
    assert!(before - g.player.rest.value >= 150, "rest {before} -> {}", g.player.rest.value);
}

#[test]
fn levels_hide_caches_now_and_then() {
    let mut caches = 0;
    for seed in 0..6 {
        let mut g = new_game(seed);
        for depth in 1..=FINAL_DEPTH {
            g.enter_level(depth);
            caches += g.map.cells().filter(|p| g.node_at(*p).is_some_and(|d| d.hidden.is_some())).count();
        }
    }
    assert!((5..200).contains(&caches), "{caches} caches in 60 levels");
}

#[test]
fn walking_into_an_encounter_runs_it_once() {
    let mut g = quiet_game(12);
    g.player.hope.value = 1000;
    let dir = encounter_beside(&mut g, "friendly_stranger");
    let target = g.player.pos.step(dir);
    assert_eq!(g.move_player(dir), Event::OpenEncounter);
    assert_eq!(g.player.pos, target, "you step into it");
    assert!(g.map.feature(target).is_none(), "and it's gone");
    assert!(g.player.hope.value > 1000);
    let report = g.last_encounter.as_ref().expect("a report for the pop-up");
    assert!(!report.gifts.is_empty());
    let water = &report.gifts[0];
    assert!(g.player.inventory.items().iter().any(|i| i.kind == water.kind), "the gift is in your pack");
}

#[test]
fn encounter_text_is_rolled_per_visit() {
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..20 {
        let mut g = quiet_game(seed);
        let dir = encounter_beside(&mut g, "friendly_stranger");
        let _ = g.move_player(dir);
        let text = g.last_encounter.as_ref().unwrap().text.clone();
        assert!(!text.contains(['{', '}', '|', '@']), "no markup left in {text:?}");
        seen.insert(text);
    }
    assert!(seen.len() > 5, "the scene varies between visits");
}

#[test]
fn a_sanctum_stays_and_is_bumped() {
    let mut g = quiet_game(13);
    g.player.hope.value = 1000;
    let dir = encounter_beside(&mut g, "peaceful_sanctum");
    let (pos, turn) = (g.player.pos, g.turn);
    assert_eq!(g.move_player(dir), Event::None, "no pop-up, just the log");
    assert_eq!(g.player.pos, pos, "bumped, not entered");
    assert!(g.map.feature(pos.step(dir)).is_some(), "still there");
    assert_eq!(g.turn, turn + 1, "sitting a while takes a turn");
    // Net of a turn's drain, it still helps.
    assert!(g.player.hope.value > 1000);
}

#[test]
fn resisting_a_horror_halves_the_loss() {
    use crate::stats::Attribute;
    use crate::world::encounter::Encounter;
    let full = Encounter::named("witness_horrors").def().needs[0].1;
    let mut seen = (false, false);
    for seed in 0..40 {
        let mut g = quiet_game(seed);
        let push = if seed % 2 == 0 { 10 } else { -10 };
        g.player.effects.add(EffectSpec::new(EffectKind::Boost(Attribute::Willpower), push, 100), "test");
        let dir = encounter_beside(&mut g, "witness_horrors");
        let _ = g.move_player(dir);
        let report = g.last_encounter.as_ref().unwrap();
        let delta = report.changes[0].1;
        if report.resisted() {
            assert_eq!(delta, full / 2);
            seen.0 = true;
        } else {
            assert_eq!(delta, full);
            seen.1 = true;
        }
    }
    assert_eq!(seen, (true, true), "both outcomes happen");
}

#[test]
fn despair_saps_willpower_and_crystal_healing() {
    let mut g = quiet_game(14);
    let (will, heal) = (g.player.attr(crate::stats::Attribute::Willpower), g.player.crystal_heal());
    g.player.hope.value = 0;
    assert!(g.player.attr(crate::stats::Attribute::Willpower) < will);
    assert!(g.player.crystal_heal() <= heal);
    let hp = g.player.hp;
    g.pass_time(50);
    assert_eq!(g.player.hp, hp, "despair never costs HP");
}

#[test]
fn levels_have_encounters() {
    let mut g = new_game(15);
    let mut total = 0;
    for depth in 1..=FINAL_DEPTH {
        g.enter_level(depth);
        total += g.map.cells().filter(|p| matches!(g.map.feature(*p), Some(Feature::Encounter { .. }))).count();
    }
    assert!(total >= FINAL_DEPTH as usize * 2, "{total} encounters over {FINAL_DEPTH} levels");
}

fn table_beside(g: &mut Game, id: &str, crystals: u32) -> Pos {
    g.player.crystals = crystals;
    let dir = encounter_beside(g, id);
    assert_eq!(g.move_player(dir), Event::OpenWager);
    g.table.as_ref().expect("a table to sit at").pos
}

#[test]
fn a_den_takes_bets_and_keeps_the_crystals_in_play() {
    let mut g = quiet_game(30);
    let pos = table_beside(&mut g, "dice_den", 20);
    assert_ne!(g.player.pos, pos, "a den is bumped, not entered");
    let total = g.player.crystals + g.house_purse(pos);
    for _ in 0..3 {
        let turn = g.turn;
        assert!(g.place_bet());
        assert_eq!(g.turn, turn + 1, "each bet takes a turn");
        assert_eq!(g.player.crystals + g.house_purse(pos), total, "crystals only change hands");
    }
    g.leave_table();
    assert!(g.map.feature(pos).is_some(), "the den is still there");
    assert!(g.table.is_none());
}

#[test]
fn a_broke_house_takes_no_bets() {
    let mut g = quiet_game(31);
    let pos = table_beside(&mut g, "dice_den", 20);
    if let Some(Feature::Encounter { purse, .. }) = g.map.feature_mut(pos) {
        *purse = 1;
    }
    assert!(g.bet_blocker().unwrap().contains("broke"));
    assert!(!g.place_bet());
    assert_eq!(g.player.crystals, 20, "nothing staked");
}

#[test]
fn you_need_the_stake_to_bet() {
    let mut g = quiet_game(32);
    table_beside(&mut g, "card_sharp", 1);
    assert!(g.bet_blocker().unwrap().contains("You need"));
    assert!(!g.place_bet());
}

#[test]
fn a_wandering_gambler_takes_one_bet_and_moves_on() {
    let mut g = quiet_game(33);
    let pos = table_beside(&mut g, "card_sharp", 20);
    assert_eq!(g.player.pos, pos, "you step up to a one-off");
    assert!(g.place_bet());
    let net = g.table.as_ref().unwrap().last.as_ref().unwrap().net;
    assert!(net == 6 || net == 12 || net == -4, "won 6 (12 on a critical) or lost the 4 staked: {net}");
    assert_eq!(g.player.crystals as i64, 20 + net);
    assert!(!g.place_bet(), "only one bet");
    g.leave_table();
    assert!(g.map.feature(pos).is_none(), "and they're gone");
}

#[test]
fn walking_away_logs_the_decline_and_a_wanderer_leaves() {
    let mut g = quiet_game(34);
    let pos = table_beside(&mut g, "card_sharp", 20);
    g.leave_table();
    assert!(logged(&g, "Suit yourself"));
    assert!(g.map.feature(pos).is_none());
    assert_eq!(g.player.crystals, 20);
}

#[test]
fn wits_win_more_card_games() {
    let wins = |push: i32| {
        (0..200)
            .filter(|&seed| {
                let mut g = quiet_game(seed);
                g.player.effects.add(EffectSpec::new(EffectKind::Boost(Attribute::Wit), push, 100), "test");
                table_beside(&mut g, "card_sharp", 20);
                g.place_bet();
                g.table.as_ref().unwrap().last.as_ref().unwrap().net > 0
            })
            .count()
    };
    assert!(wins(4) > wins(-4) + 40, "a sharp mind beats the sharp");
}

#[test]
fn a_dealer_keeps_their_pronouns_from_scene_to_result() {
    for seed in 0..40 {
        let mut g = quiet_game(seed);
        table_beside(&mut g, "dice_den", 20);
        let scene = g.table.as_ref().unwrap().text.clone();
        let wrong = if scene.contains("wiry man") { "She " } else { "He " };
        for _ in 0..3 {
            g.place_bet();
            let bet = g.table.as_ref().unwrap().last.as_ref().unwrap();
            assert!(!bet.text.contains(wrong), "seed {seed}: {scene:?} then {:?}", bet.text);
        }
    }
}

#[test]
fn walking_into_a_portal_carries_you_to_the_next_world() {
    let mut g = new_game(7);
    g.monsters.clear();
    let portal = g.map.cells().find(|p| g.map.feature(*p) == Some(&Feature::Portal)).expect("level has a portal");
    let (from, dir) = portal
        .neighbors()
        .filter(|n| g.map.passable(*n))
        .find_map(|n| Dir::ALL.into_iter().find(|d| n.step(*d) == portal).map(|d| (n, d)))
        .expect("portal has an open neighbor");
    g.player.pos = from;

    assert_eq!(g.move_player(dir), Event::Portal);
    assert_eq!(g.depth, 2);
    assert_ne!(g.map.feature(g.player.pos), Some(&Feature::Portal), "you arrive at the new start");
    assert!(g.log.recent(5).iter().any(|m| m.text.starts_with("Spacetime swallows you up")));
}

#[test]
fn a_nap_lifts_hope_and_oversleeping_wears_it_down() {
    let mut g = quiet_game(8);
    g.player.rest.value = 400;
    let before = g.player.hope.value;
    g.nap();
    sleep_it_off(&mut g, 500);
    assert!(g.player.hope.value > before, "a nap nets hope, even after its drain");
    assert!(!logged(&g, "slept too long"));

    let after = g.player.sleep_def().oversleep.as_ref().unwrap().after;
    g.player.rest.value = 100;
    g.nap();
    g.player.slept = after;
    let before = g.player.hope.value;
    g.sleep_turn();
    assert!(logged(&g, "slept too long"));
    assert!(g.player.hope.value < before - 1, "more than the usual drain");
}

#[test]
fn a_collapse_brings_no_relief() {
    let mut g = quiet_game(7);
    g.player.rest.value = 1;
    g.wait();
    let before = g.player.hope.value;
    for _ in 0..10 {
        g.sleep_turn();
    }
    assert!(g.player.hope.value < before, "only the usual drain");
}

#[test]
fn ditch_weed_lifts_hope_and_gives_the_munchies() {
    let mut g = quiet_game(3);
    let (hope, hunger) = (g.player.hope.value, g.player.hunger.value);
    let weed = crate::items::Consumable::named("ditch_weed");
    add_and_use(&mut g, Item::food(weed, 1));
    assert!(g.player.hope.value > hope + 300);
    assert!(g.player.hunger.value < hunger - 90);
    assert!(g.player.attr(Attribute::Wit) < g.player.base.get(Attribute::Wit));
    assert_eq!(g.player.inventory.count(&ItemKind::Consumable(weed)), 0, "smoked");
    assert!(logged(&g, "Hope +350"));
}

#[test]
fn a_reusable_item_is_kept_and_rests_between_uses() {
    let mut g = quiet_game(3);
    let tool = crate::items::Consumable::named("fleshlight_multitool");
    let kind = ItemKind::Consumable(tool);
    g.player.inventory.add(Item::food(tool, 1)).unwrap();
    let idx = || g.player.inventory.items().iter().position(|i| i.kind == kind).unwrap();
    let hope = g.player.hope.value;
    g.use_item(idx());
    assert!(g.player.hope.value > hope + 200);
    assert_eq!(g.player.inventory.count(&kind), 1, "kept");

    let (turn, hope) = (g.turn, g.player.hope.value);
    let idx = g.player.inventory.items().iter().position(|i| i.kind == kind).unwrap();
    g.use_item(idx);
    assert_eq!((g.turn, g.player.hope.value), (turn, hope), "too soon: refused, and no time passes");
    assert!(logged(&g, "not in the mood"));

    g.pass_time(400);
    let hope = g.player.hope.value;
    g.use_item(idx);
    assert!(g.player.hope.value > hope + 200, "ready again after the cooldown");
}

#[test]
fn coffee_takes_any_water_and_spends_the_cheapest_first() {
    let coffee = recipes().iter().position(|r| r.name() == "Hot Coffee").unwrap();
    let mut g = quiet_game(4);
    g.player.inventory = Default::default();
    let bottled = crate::items::Consumable::named("bottled_water");
    let dirty = Component::named("dirty_water");
    g.player.inventory.add(Item::component(Component::named("instant_coffee"), 1)).unwrap();
    g.player.inventory.add(Item::food(bottled, 1)).unwrap();
    assert!(g.can_craft(&recipes()[coffee]), "bottled water will do");
    g.player.inventory.add(Item::component(dirty, 1)).unwrap();
    g.player.skills.set_rank(crate::stats::Skill::Crafting, 10);
    g.craft(coffee);
    assert_eq!(g.player.inventory.count_component(dirty), 0, "the dirty water went in");
    assert_eq!(g.player.inventory.count(&ItemKind::Consumable(bottled)), 1, "the bottled water is kept");
    let log: Vec<String> = g.log.recent(5).iter().map(|m| m.text.clone()).collect();
    assert!(g.player.inventory.count(&ItemKind::Consumable(crate::items::Consumable::named("coffee"))) >= 1, "{log:?}");
}
