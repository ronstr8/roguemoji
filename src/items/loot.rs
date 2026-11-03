//! Depth-scaled loot tables for floor items and Autovend stock.

use super::melee::Origin;
use super::{Ammo, Armor, Component, Consumable, Gun, GunModel, Item, ItemKind, MeleeWeapon, Pack};
use crate::content::{Loot, Price};
use crate::dice::Dice;

/// A gun model found at `depth`, weighted by each gun's `loot`. Falls back
/// to any gun at all if none is marked as loot here.
fn random_model(dice: &mut Dice, depth: u32) -> GunModel {
    pick_loot(dice, GunModel::all().map(|g| (g, g.def().loot)), depth)
        .unwrap_or_else(|| *dice.pick(&GunModel::all().collect::<Vec<_>>()))
}

/// Ammo for a gun model: any load that fits its chamber. Startup validation
/// guarantees there is at least one.
pub fn ammo_for(model: GunModel, dice: &mut Dice) -> Ammo {
    let options: Vec<Ammo> = Ammo::all().filter(|a| model.def().cartridge.accepts(a.def().cartridge)).collect();
    *dice.pick(&options)
}

pub fn random_gun(dice: &mut Dice, depth: u32) -> Gun {
    let model = random_model(dice, depth);
    gun_from(model, dice)
}

/// A gun of `model`, partly loaded with ammo that fits.
fn gun_from(model: GunModel, dice: &mut Dice) -> Gun {
    let ammo = ammo_for(model, dice);
    let cap = model.def().capacity;
    let rounds = dice.range(0, cap as i32) as u32;
    Gun::loaded_with(model, ammo, rounds)
}

fn random_ammo(dice: &mut Dice, depth: u32) -> Item {
    // Ammo found tends to match guns that exist at this depth.
    let model = random_model(dice, depth);
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
///
/// Items are rolled as loot one level deeper, but only from things with a
/// price; prices are the files' `price` at this depth. A slot nothing can
/// fill is left out.
pub fn kiosk_stock(dice: &mut Dice, depth: u32) -> Vec<(Item, u32)> {
    let deeper = depth + 1;
    let mut stock = Vec::new();
    for _ in 0..3 {
        if let Some((ammo, rounds)) = shop_ammo(dice, deeper) {
            let def = ammo.def();
            // A full box costs `price`; partial boxes pay their share.
            let price = (def.price.at(depth) * rounds).div_ceil(def.box_size).max(1);
            stock.push((Item::ammo(ammo, rounds), price));
        }
    }
    for _ in 0..2 {
        let food = shop_pick(dice, Consumable::all().map(|c| (c, c.def().loot_weight, c.def().price)));
        if let Some(food) = food {
            stock.push((Item::food(food, 1), food.def().price.at(depth)));
        }
    }
    if let Some(c) = shop_pick(dice, Component::all().map(|c| (c, c.def().loot_weight, c.def().price))) {
        stock.push((Item::component(c, 2), 2 * c.def().price.at(depth)));
    }
    let guns = GunModel::all().filter(|g| g.def().price.for_sale());
    if let Some(model) = pick_loot(dice, guns.map(|g| (g, g.def().loot)), deeper) {
        stock.push((Item::gun(gun_from(model, dice)), model.def().price.at(depth)));
    }
    if dice.chance(0.5) {
        let armor = Armor::all().filter(|a| a.def().price.for_sale());
        if let Some(a) = pick_loot(dice, armor.map(|a| (a, a.def().loot)), deeper) {
            stock.push((Item::one(ItemKind::Armor(a)), a.def().price.at(depth)));
        }
    }
    if dice.chance(0.35) {
        let packs = Pack::all().filter(|p| p.def().price.for_sale());
        if let Some(p) = pick_loot(dice, packs.map(|p| (p, p.def().loot)), deeper) {
            stock.push((Item::one(ItemKind::Pack(p)), p.def().price.at(depth)));
        }
    }
    if depth >= 4 && dice.chance(0.3) {
        let artifacts = MeleeWeapon::all().filter(|m| m.def().origin == Origin::Alien && m.def().price.for_sale());
        if let Some(m) = pick_loot(dice, artifacts.map(|m| (m, m.def().loot)), depth) {
            stock.push((Item::one(ItemKind::Melee(m)), m.def().price.at(depth)));
        }
    }
    stock
}

/// A box of ammo to sell: a priced load for a gun found at `depth`.
fn shop_ammo(dice: &mut Dice, depth: u32) -> Option<(Ammo, u32)> {
    let model = random_model(dice, depth);
    let fits = |a: &Ammo| model.def().cartridge.accepts(a.def().cartridge) && a.def().price.for_sale();
    let options: Vec<Ammo> = Ammo::all().filter(fits).collect();
    if options.is_empty() {
        return None;
    }
    let ammo = *dice.pick(&options);
    let box_size = ammo.def().box_size as i32;
    Some((ammo, dice.range(box_size / 2, box_size) as u32))
}

/// Something for sale, by loot weight; if nothing for sale is loot, any
/// priced item will do.
fn shop_pick<T: Copy>(dice: &mut Dice, items: impl Iterator<Item = (T, u32, Price)>) -> Option<T> {
    let priced: Vec<(T, u32)> = items.filter(|(_, _, p)| p.for_sale()).map(|(x, w, _)| (x, w)).collect();
    dice.weighted(&priced).or_else(|| (!priced.is_empty()).then(|| dice.pick(&priced).0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kiosks_sell_only_priced_items() {
        let mut dice = Dice::seeded(7);
        for depth in 1..=8 {
            for _ in 0..20 {
                let stock = kiosk_stock(&mut dice, depth);
                assert!(stock.len() >= 6, "ammo, food, components and a gun are always stocked");
                assert!(stock.iter().all(|(_, price)| *price > 0));
            }
        }
    }
}
