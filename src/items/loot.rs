//! Depth-scaled loot tables for floor items and Autovend stock.

use super::melee::Origin;
use super::{Ammo, Armor, Component, Consumable, Gun, GunModel, Item, ItemKind, MeleeWeapon, Pack};
use crate::content::Loot;
use crate::dice::Dice;

/// Guns weighted by how common they'd be in an abandoned American town,
/// with heavier rifles appearing deeper.
fn gun_table(depth: u32) -> Vec<(GunModel, u32)> {
    use GunModel::*;
    let d = depth;
    vec![
        (TargetPistol22, 10),
        (SnubRevolver38, 10),
        (CompactPistol9, 12),
        (ServicePistol9, 8 + d),
        (Pistol1911, 5 + d),
        (Revolver357, 3 + d),
        (Rifle22, 10),
        (SingleShot20, 8),
        (PumpShotgun12, 4 + d),
        (Carbine9, 2 + d),
        (LeverAction3030, if d >= 2 { 3 + d } else { 0 }),
        (Sks, if d >= 3 { 2 + d } else { 0 }),
        (Ar15, if d >= 3 { 1 + d } else { 0 }),
        (BoltRifle308, if d >= 4 { d } else { 0 }),
    ]
}

/// Ammo for a gun model: the first load for its cartridge (plus the
/// alternative load sometimes).
pub fn ammo_for(model: GunModel, dice: &mut Dice) -> Ammo {
    let options: Vec<Ammo> =
        Ammo::ALL.iter().copied().filter(|a| model.def().cartridge.accepts(a.def().cartridge)).collect();
    *dice.pick(&options)
}

pub fn random_gun(dice: &mut Dice, depth: u32) -> Gun {
    let model = dice.weighted(&gun_table(depth)).unwrap_or(GunModel::TargetPistol22);
    let ammo = ammo_for(model, dice);
    let cap = model.def().capacity;
    let rounds = dice.range(0, cap as i32) as u32;
    Gun::loaded_with(model, ammo, rounds)
}

fn random_ammo(dice: &mut Dice, depth: u32) -> Item {
    // Ammo found tends to match guns that exist at this depth.
    let model = dice.weighted(&gun_table(depth)).unwrap_or(GunModel::TargetPistol22);
    let ammo = ammo_for(model, dice);
    let box_size = ammo.def().box_size as i32;
    Item::ammo(ammo, dice.range(box_size / 2, box_size) as u32)
}

fn random_consumable(dice: &mut Dice) -> Consumable {
    let table: Vec<(Consumable, u32)> = Consumable::all().map(|c| (c, c.def().loot_weight)).collect();
    // If no food is marked as loot, any will do.
    dice.weighted(&table).unwrap_or_else(|| *dice.pick(&Consumable::all().collect::<Vec<_>>()))
}

fn random_component(dice: &mut Dice) -> Component {
    let table: Vec<(Component, u32)> = Component::all().map(|c| (c, c.def().loot_weight)).collect();
    // If no component is marked as floor loot, any will do.
    dice.weighted(&table).unwrap_or_else(|| *dice.pick(&Component::all().collect::<Vec<_>>()))
}

/// Pick by each item's depth-scaled loot weight, if anything qualifies.
fn pick_loot<T: Copy>(dice: &mut Dice, items: impl Iterator<Item = (T, Loot)>, depth: u32) -> Option<T> {
    let table: Vec<(T, u32)> = items.map(|(x, loot)| (x, loot.weight_at(depth))).collect();
    dice.weighted(&table)
}

fn random_armor(dice: &mut Dice, depth: u32) -> Option<Armor> {
    pick_loot(dice, Armor::all().map(|a| (a, a.def().loot)), depth)
}

fn random_pack(dice: &mut Dice, depth: u32) -> Option<Pack> {
    pick_loot(dice, Pack::all().map(|p| (p, p.def().loot)), depth)
}

/// A melee weapon found lying around: Visitor artifacts (`alien = true`) or
/// everything else.
fn random_melee(dice: &mut Dice, depth: u32, alien: bool) -> Option<MeleeWeapon> {
    let pool = MeleeWeapon::all().filter(|m| (m.def().origin == Origin::Alien) == alien);
    pick_loot(dice, pool.map(|m| (m, m.def().loot)), depth)
}

#[derive(Clone, Copy)]
enum Category {
    Gun,
    Ammo,
    Tool,
    Alien,
    Armor,
    Pack,
    Food,
    Component,
}

/// A random item to leave lying on the floor at `depth`.
pub fn random_floor_item(dice: &mut Dice, depth: u32) -> Item {
    let alien_weight = if depth >= 3 { depth } else { 0 };
    let cat = dice
        .weighted(&[
            (Category::Gun, 6),
            (Category::Ammo, 16),
            (Category::Tool, 8),
            (Category::Alien, alien_weight),
            (Category::Armor, 4),
            (Category::Pack, 3),
            (Category::Food, 12),
            (Category::Component, 14),
        ])
        .unwrap_or(Category::Component);
    let item = match cat {
        Category::Gun => Some(Item::gun(random_gun(dice, depth))),
        Category::Ammo => Some(random_ammo(dice, depth)),
        Category::Tool => random_melee(dice, depth, false).map(|m| Item::one(ItemKind::Melee(m))),
        Category::Alien => random_melee(dice, depth, true).map(|m| Item::one(ItemKind::Melee(m))),
        Category::Armor => random_armor(dice, depth).map(|a| Item::one(ItemKind::Armor(a))),
        Category::Pack => random_pack(dice, depth).map(|p| Item::one(ItemKind::Pack(p))),
        Category::Food => Some(Item::food(random_consumable(dice), 1)),
        Category::Component => None,
    };
    // Nothing of the chosen kind can turn up here (e.g. a mod zeroed every
    // pack's loot weight): leave crafting scraps instead.
    item.unwrap_or_else(|| Item::component(random_component(dice), dice.range(1, 2) as u32))
}

/// Stock for an Autovend kiosk: `(item, price in crystals)`.
pub fn kiosk_stock(dice: &mut Dice, depth: u32) -> Vec<(Item, u32)> {
    let mut stock = Vec::new();
    for _ in 0..3 {
        let item = random_ammo(dice, depth + 1);
        let price = 4 + item.qty / 3;
        stock.push((item, price));
    }
    for _ in 0..2 {
        let food = random_consumable(dice);
        stock.push((Item::food(food, 1), food.def().price));
    }
    stock.push((Item::component(random_component(dice), 2), 4));
    let gun = random_gun(dice, depth + 1);
    let gun_price = 20 + gun.def().barrel_in as u32 + depth * 3;
    stock.push((Item::gun(gun), gun_price));
    if dice.chance(0.5)
        && let Some(a) = random_armor(dice, depth + 1)
    {
        stock.push((Item::one(ItemKind::Armor(a)), 25 + depth * 3));
    }
    if dice.chance(0.35)
        && let Some(p) = random_pack(dice, depth + 1)
    {
        stock.push((Item::one(ItemKind::Pack(p)), 18 + depth * 3));
    }
    if depth >= 4
        && dice.chance(0.3)
        && let Some(m) = random_melee(dice, depth, true)
    {
        stock.push((Item::one(ItemKind::Melee(m)), 60 + depth * 5));
    }
    stock
}
