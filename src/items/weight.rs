//! Item weights. All weights are in hundredths of a pound so that a single
//! .22 round (about 0.01 lb) still counts.

use super::{Ammo, Armor, Component, Consumable, Gun, GunModel, ItemKind, MeleeWeapon, Pack};

/// Format a weight in hundredths of a pound as e.g. `12.5 lb`. Anything
/// lighter than a tenth of a pound gets a second decimal, so it doesn't
/// read as weightless.
pub fn fmt_lb(w: u32) -> String {
    let lb = w as f64 / 100.0;
    if (1..10).contains(&w) { format!("{lb:.2} lb") } else { format!("{lb:.1} lb") }
}

/// Weight of a single unit of `kind`. A gun's loaded rounds count too.
pub fn unit_weight(kind: &ItemKind) -> u32 {
    match kind {
        // Crystals are light itself; they weigh nothing.
        ItemKind::Crystals => 0,
        ItemKind::Gun(g) => loaded_gun(g),
        ItemKind::Melee(m) => melee(*m),
        ItemKind::Ammo(a) => ammo(*a),
        ItemKind::Armor(a) => armor(*a),
        ItemKind::Component(c) => component(*c),
        ItemKind::Consumable(c) => consumable(*c),
        ItemKind::Pack(p) => pack(*p),
    }
}

/// A gun plus the rounds in it.
pub fn loaded_gun(g: &Gun) -> u32 {
    gun(g.model) + g.ammo.map_or(0, |a| ammo(a) * g.loaded)
}

pub fn gun(m: GunModel) -> u32 {
    m.def().weight
}

pub fn ammo(a: Ammo) -> u32 {
    a.def().cartridge.def().round_weight
}

pub fn melee(x: MeleeWeapon) -> u32 {
    x.def().weight
}

pub fn armor(x: Armor) -> u32 {
    x.def().weight
}

pub fn component(c: Component) -> u32 {
    c.def().weight
}

pub fn consumable(c: Consumable) -> u32 {
    c.def().weight
}

pub fn pack(x: Pack) -> u32 {
    x.def().weight
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_things_keep_a_visible_weight() {
        assert_eq!(fmt_lb(1), "0.01 lb", "a single .22 round isn't weightless");
        assert_eq!(fmt_lb(9), "0.09 lb");
        assert_eq!(fmt_lb(0), "0.0 lb");
        assert_eq!(fmt_lb(10), "0.1 lb");
        assert_eq!(fmt_lb(1250), "12.5 lb");
    }
}
