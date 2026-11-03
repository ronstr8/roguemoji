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
    let mut offer = |item: Item| {
        if let Some(price) = shop_price(&item, depth) {
            stock.push((item, price));
        }
    };
    for _ in 0..3 {
        if let Some((ammo, rounds)) = shop_ammo(dice, deeper) {
            offer(Item::ammo(ammo, rounds));
        }
    }
    for _ in 0..2 {
        if let Some(food) = shop_pick(dice, Consumable::all().map(|c| (c, c.def().loot_weight, c.def().price))) {
            offer(Item::food(food, 1));
        }
    }
    if let Some(c) = shop_pick(dice, Component::all().map(|c| (c, c.def().loot_weight, c.def().price))) {
        offer(Item::component(c, 2));
    }
    let guns = GunModel::all().filter(|g| g.def().price.for_sale());
    if let Some(model) = pick_loot(dice, guns.map(|g| (g, g.def().loot)), deeper) {
        offer(Item::gun(gun_from(model, dice)));
    }
    if dice.chance(0.5) {
        let armor = Armor::all().filter(|a| a.def().price.for_sale());
        if let Some(a) = pick_loot(dice, armor.map(|a| (a, a.def().loot)), deeper) {
            offer(Item::one(ItemKind::Armor(a)));
        }
    }
    if dice.chance(0.35) {
        let packs = Pack::all().filter(|p| p.def().price.for_sale());
        if let Some(p) = pick_loot(dice, packs.map(|p| (p, p.def().loot)), deeper) {
            offer(Item::one(ItemKind::Pack(p)));
        }
    }
    if depth >= 4 && dice.chance(0.3) {
        let artifacts = MeleeWeapon::all().filter(|m| m.def().origin == Origin::Alien && m.def().price.for_sale());
        if let Some(m) = pick_loot(dice, artifacts.map(|m| (m, m.def().loot)), depth) {
            offer(Item::one(ItemKind::Melee(m)));
        }
    }
    stock
}

/// What an Autovend on `depth` charges for `item` (the whole stack), or
/// `None` if it doesn't deal in it: crystals, or anything without a price.
/// A gun's loaded rounds are included; ammo `price` is per full box.
pub fn shop_price(item: &Item, depth: u32) -> Option<u32> {
    let priced = |p: Price| p.for_sale().then(|| p.at(depth));
    let ammo_price = |a: Ammo, rounds: u32| {
        let def = a.def();
        priced(def.price).map(|box_price| (box_price * rounds).div_ceil(def.box_size).max(1))
    };
    let n = item.qty;
    match &item.kind {
        ItemKind::Crystals => None,
        ItemKind::Gun(g) => {
            let rounds = g.ammo.filter(|_| g.loaded > 0).and_then(|a| ammo_price(a, g.loaded)).unwrap_or(0);
            priced(g.model.def().price).map(|p| p + rounds)
        }
        ItemKind::Ammo(a) => ammo_price(*a, n),
        ItemKind::Melee(m) => priced(m.def().price).map(|p| p * n),
        ItemKind::Armor(a) => priced(a.def().price).map(|p| p * n),
        ItemKind::Pack(p) => priced(p.def().price).map(|p| p * n),
        ItemKind::Component(c) => priced(c.def().price).map(|p| p * n),
        ItemKind::Consumable(c) => priced(c.def().price).map(|p| p * n),
    }
}

/// Autovends buy back at this fraction of their own price.
const SELL_RATE: f64 = 0.5;

/// How much of a pack stack one sale hands over: a whole stack of ammo,
/// otherwise one at a time.
pub fn sell_qty(item: &Item) -> u32 {
    match item.kind {
        ItemKind::Ammo(_) => item.qty,
        _ => 1,
    }
}

/// What an Autovend on `depth` pays for one sale of `item` (see
/// [`sell_qty`]), or `None` if it won't buy it.
pub fn sell_price(item: &Item, depth: u32) -> Option<u32> {
    let sold = Item { kind: item.kind.clone(), qty: sell_qty(item) };
    shop_price(&sold, depth).map(|p| ((p as f64 * SELL_RATE) as u32).max(1))
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
    fn sale_prices() {
        let vest = Item::one(ItemKind::Armor(Armor::named("kevlar_vest")));
        assert_eq!((shop_price(&vest, 2), sell_price(&vest, 2)), (Some(31), Some(15)));
        // Ammo sells by the stack, priced as a share of the box.
        let ammo = Ammo::named("22lr");
        let half_box = Item::ammo(ammo, ammo.def().box_size / 2);
        assert_eq!(sell_qty(&half_box), half_box.qty);
        assert_eq!(sell_price(&half_box, 1), Some(2));
        // Stacks sell one at a time; unpriced things and crystals don't sell.
        let hardtack = Consumable::named("hardtack");
        let one = shop_price(&Item::food(hardtack, 1), 1).unwrap();
        assert_eq!(sell_price(&Item::food(hardtack, 3), 1), Some((one / 2).max(1)));
        assert_eq!(sell_price(&Item::one(ItemKind::Melee(MeleeWeapon::named("crowbar"))), 1), None);
        assert_eq!(sell_price(&Item::crystals(5), 1), None);
        // A loaded gun is worth more than an empty one.
        let model = GunModel::named("service_pistol_9mm");
        let empty = Item::gun(Gun::loaded_with(model, Ammo::named("9mm_fmj"), 0));
        let full = Item::gun(Gun::loaded_with(model, Ammo::named("9mm_fmj"), 15));
        assert_eq!(shop_price(&empty, 1), Some(31));
        assert!(shop_price(&full, 1) > shop_price(&empty, 1));
    }

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
