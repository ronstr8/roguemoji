//! Items: definitions for every kind of object, plus the generic `Item` wrapper.

pub mod armor;
pub mod ballistics;
pub mod components;
pub mod consumables;
pub mod guns;
pub mod inventory;
pub mod loot;
pub mod melee;
pub mod packs;
pub mod tool;
pub mod weight;

pub use armor::Armor;
pub use ballistics::Ammo;
pub use components::Component;
pub use consumables::Consumable;
pub use guns::{Gun, GunModel};
pub use inventory::Inventory;
pub use melee::MeleeWeapon;
pub use packs::Pack;
pub use tool::Tool;

use std::cmp::Ordering;

pub const CRYSTAL_GLYPH: &str = "💎";
pub const AMMO_GLYPH: &str = "🔸";
pub const GUN_GLYPH: &str = "🔫";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ItemKind {
    /// Life crystals: healing and currency.
    Crystals,
    Gun(Gun),
    Melee(MeleeWeapon),
    Ammo(Ammo),
    Armor(Armor),
    Component(Component),
    Consumable(Consumable),
    Pack(Pack),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub qty: u32,
}

impl Item {
    pub fn new(kind: ItemKind, qty: u32) -> Self {
        Self { kind, qty: qty.max(1) }
    }

    pub fn one(kind: ItemKind) -> Self {
        Self::new(kind, 1)
    }

    pub fn crystals(n: u32) -> Self {
        Self::new(ItemKind::Crystals, n)
    }

    pub fn gun(g: Gun) -> Self {
        Self::one(ItemKind::Gun(g))
    }

    pub fn ammo(a: Ammo, n: u32) -> Self {
        Self::new(ItemKind::Ammo(a), n)
    }

    pub fn component(c: Component, n: u32) -> Self {
        Self::new(ItemKind::Component(c), n)
    }

    pub fn food(c: Consumable, n: u32) -> Self {
        Self::new(ItemKind::Consumable(c), n)
    }

    pub fn stackable(&self) -> bool {
        !matches!(self.kind, ItemKind::Gun(_) | ItemKind::Melee(_) | ItemKind::Armor(_) | ItemKind::Pack(_))
    }

    pub fn base_name(&self) -> String {
        match &self.kind {
            ItemKind::Crystals => "Life Crystal".into(),
            ItemKind::Gun(g) => g.def().name.clone(),
            ItemKind::Melee(m) => m.def().name.clone(),
            ItemKind::Ammo(a) => a.name().into(),
            ItemKind::Armor(a) => a.def().name.clone(),
            ItemKind::Component(c) => c.def().name.clone(),
            ItemKind::Consumable(c) => c.def().name.clone(),
            ItemKind::Pack(p) => p.def().name.clone(),
        }
    }

    /// Name with quantity or loaded status, for lists.
    pub fn display_name(&self) -> String {
        match &self.kind {
            ItemKind::Gun(g) => format!("{} [{}]", g.def().name, g.status()),
            _ if self.qty > 1 => format!("{} x{}", self.base_name(), self.qty),
            _ => self.base_name(),
        }
    }

    /// Whether a recipe asking for any `tag` (e.g. `water`) can use this.
    pub fn has_tag(&self, tag: &str) -> bool {
        let tags: &[String] = match &self.kind {
            ItemKind::Component(c) => &c.def().tags,
            ItemKind::Consumable(c) => &c.def().tags,
            _ => &[],
        };
        tags.iter().any(|t| t == tag)
    }

    pub fn glyph(&self) -> &'static str {
        match &self.kind {
            ItemKind::Crystals => CRYSTAL_GLYPH,
            ItemKind::Gun(_) => GUN_GLYPH,
            ItemKind::Melee(m) => m.def().glyph.as_str(),
            ItemKind::Ammo(_) => AMMO_GLYPH,
            ItemKind::Armor(a) => a.def().glyph.as_str(),
            ItemKind::Component(c) => c.def().glyph.as_str(),
            ItemKind::Consumable(c) => c.def().glyph.as_str(),
            ItemKind::Pack(p) => p.def().glyph.as_str(),
        }
    }

    pub fn category(&self) -> &'static str {
        match &self.kind {
            ItemKind::Crystals => "Crystal",
            ItemKind::Gun(g) => match g.def().skill {
                crate::stats::Skill::SmallArms => "Handgun",
                _ => "Long gun",
            },
            ItemKind::Melee(m) => m.def().origin.label(),
            ItemKind::Ammo(_) => "Ammunition",
            ItemKind::Armor(_) => "Armor",
            ItemKind::Component(_) => "Component",
            ItemKind::Consumable(_) => "Food & drink",
            ItemKind::Pack(_) => "Pack",
        }
    }

    /// Weight of one unit (a loaded gun includes its rounds).
    pub fn unit_weight(&self) -> u32 {
        weight::unit_weight(&self.kind)
    }

    /// Weight of the whole stack.
    pub fn weight(&self) -> u32 {
        self.unit_weight() * self.qty
    }

    /// The stack's weight, with the unit weight too when there's more than one.
    fn weight_text(&self) -> String {
        if self.qty > 1 {
            format!("{} ({} each)", weight::fmt_lb(self.weight()), weight::fmt_lb(self.unit_weight()))
        } else {
            weight::fmt_lb(self.weight())
        }
    }

    /// Detailed description lines for the inventory pop-up.
    pub fn describe(&self) -> Vec<String> {
        let mut out = Vec::new();
        match &self.kind {
            ItemKind::Crystals => {
                out.push("Something dies, something crystallizes. Absorb to heal; spend at Autovends.".into());
            }
            ItemKind::Gun(g) => out.push(g.def().desc.to_string()),
            ItemKind::Melee(m) => {
                let d = m.def();
                out.push(d.desc.to_string());
                if !d.special.describe().is_empty() {
                    out.push(d.special.describe().into());
                }
            }
            ItemKind::Ammo(a) => {
                let d = a.def();
                out.push(format!(
                    "{} {} · {} gr{} · {} fps from {}\" test barrel",
                    d.cartridge.name(),
                    d.load.abbrev(),
                    d.grains,
                    if d.projectiles > 1 { format!(" ({} pellets)", d.projectiles) } else { String::new() },
                    d.ref_fps,
                    d.ref_barrel_in
                ));
            }
            ItemKind::Armor(a) => {
                let d = a.def();
                out.push(d.desc.to_string());
            }
            ItemKind::Component(c) => {
                let d = c.def();
                out.push(d.desc.clone());
                if let Some(drink) = d.drink {
                    out.push(format!(
                        "Drinkable: hydration +{}, Survival vs {} or {}",
                        drink.hydration,
                        drink.risk,
                        drink.sickness.describe()
                    ));
                }
            }
            ItemKind::Consumable(c) => out.push(c.def().desc.to_string()),
            ItemKind::Pack(p) => out.push(p.def().desc.to_string()),
        }
        // Items with a stats block list their weight there instead.
        if self.stats(0).is_empty() {
            out.push(format!("Weight {}", self.weight_text()));
        }
        out
    }

    /// Labeled stats shown one per line under the description, e.g.
    /// `Barrel 4.5" rifled`. Empty for items without a stats block.
    /// `brawn` is the wielder's melee damage bonus, folded into Damage.
    pub fn stats(&self, brawn: i32) -> Vec<Stat> {
        let mut out = match &self.kind {
            ItemKind::Gun(g) => {
                let d = g.def();
                vec![
                    Stat::plain("Type", format!("{} {}", d.cartridge.name(), d.action.name())),
                    Stat::plain(
                        "Barrel",
                        format!("{}\" {}", d.barrel_in, if d.rifled { "rifled" } else { "smoothbore" }),
                    ),
                    Stat::higher("Capacity", d.capacity.to_string(), d.capacity as i64),
                    Stat::plain("Skill", d.skill.name().to_string()),
                    Stat::higher("Accuracy", format!("{:+}", d.accuracy), d.accuracy as i64),
                    Stat::lower("Reload Turns", d.action.reload_turns().to_string(), d.action.reload_turns() as i64),
                ]
            }
            ItemKind::Melee(m) => melee_stats(m.def(), brawn),
            ItemKind::Armor(a) => {
                let d = a.def();
                vec![
                    Stat::plain("Armor", d.armor.to_string()),
                    Stat::plain("Dodge Penalty", d.dodge_penalty.to_string()),
                ]
            }
            ItemKind::Consumable(c) => consumable_stats(c.def()),
            ItemKind::Pack(p) => vec![capacity_stat(p.def().capacity)],
            _ => return Vec::new(),
        };
        out.push(Stat::lower("Weight", self.weight_text(), self.weight() as i64));
        out
    }
}

/// What fighting bare-handed is like, to compare a first weapon against.
pub fn fist_stats(brawn: i32) -> Vec<Stat> {
    let mut out = melee_stats(&melee::FISTS, brawn);
    out.push(nothing_carried());
    out
}

/// What wearing no pack is like, to compare a first pack against.
pub fn no_pack_stats() -> Vec<Stat> {
    vec![capacity_stat(0), nothing_carried()]
}

fn nothing_carried() -> Stat {
    Stat::lower("Weight", weight::fmt_lb(0), 0)
}

/// A melee weapon's stats, without weight (fists weigh nothing).
fn melee_stats(d: &melee::MeleeDef, brawn: i32) -> Vec<Stat> {
    let (lo, hi) = d.damage;
    vec![
        Stat::plain("Skill", d.skill.name().to_string()),
        damage_stat(lo, hi, brawn),
        Stat::higher("Accuracy", format!("{:+}", d.accuracy), d.accuracy as i64),
        Stat::higher("Penetration", d.pen.to_string(), d.pen as i64),
    ]
}

fn capacity_stat(capacity: u32) -> Stat {
    Stat::higher("Carry Capacity", format!("+{}", weight::fmt_lb(capacity)), capacity as i64)
}

/// Melee damage as the wielder deals it, `4-7`, noting the weapon's own range
/// and the Brawn bonus (`(2-5, +2 Brawn)`) when Brawn changes it.
fn damage_stat(lo: i32, hi: i32, brawn: i32) -> Stat {
    let dealt = format!("{}-{}", (lo + brawn).max(0), (hi + brawn).max(0));
    let mut stat = Stat::higher("Damage", dealt, (lo + hi) as i64);
    if brawn != 0 {
        stat.note = Some(format!("({lo}-{hi}, {brawn:+} Brawn)"));
    }
    stat
}

/// One line of an item's stats block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stat {
    pub label: &'static str,
    pub text: String,
    /// How the value was worked out, shown after it, e.g. `(2-5, +2 Brawn)`.
    pub note: Option<String>,
    /// The number behind the text and which way is better, for stats worth
    /// comparing between two items.
    pub score: Option<(i64, Better)>,
}

/// Which way a stat improves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Better {
    Higher,
    Lower,
}

impl Stat {
    pub fn plain(label: &'static str, text: String) -> Self {
        Self { label, text, note: None, score: None }
    }

    pub fn higher(label: &'static str, text: String, value: i64) -> Self {
        Self { label, text, note: None, score: Some((value, Better::Higher)) }
    }

    pub fn lower(label: &'static str, text: String, value: i64) -> Self {
        Self { label, text, note: None, score: Some((value, Better::Lower)) }
    }

    /// The value with its note, if any.
    pub fn full_text(&self) -> String {
        match &self.note {
            Some(note) => format!("{} {note}", self.text),
            None => self.text.clone(),
        }
    }

    /// How this stat compares with `other`'s: `Greater` means this one is
    /// better. `None` if either can't be compared.
    pub fn versus(&self, other: &Stat) -> Option<Ordering> {
        let ((a, better), (b, _)) = (self.score?, other.score?);
        Some(if better == Better::Higher { a.cmp(&b) } else { b.cmp(&a) })
    }
}

/// What eating or drinking does: only the meters it moves and the effects it
/// grants. The effects share one label so they read as a list.
fn consumable_stats(d: &consumables::ConsumableDef) -> Vec<Stat> {
    let mut out = Vec::new();
    if d.satiety != 0 {
        out.push(Stat::plain("Satiety", format!("{:+}", d.satiety)));
    }
    if d.hydration != 0 {
        out.push(Stat::plain("Hydration", format!("{:+}", d.hydration)));
    }
    if d.heal > 0 {
        out.push(Stat::plain("Heal", format!("{} HP", d.heal)));
    }
    if d.cures {
        out.push(Stat::plain("Cures", "poison & bleeding".to_string()));
    }
    for (i, e) in d.effects.iter().enumerate() {
        out.push(Stat::plain(if i == 0 { "Effects" } else { "" }, e.describe()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_compare_in_the_right_direction() {
        let crowbar = Item::one(ItemKind::Melee(MeleeWeapon::named("crowbar")));
        let (a, b) = (crowbar.stats(0), crowbar.stats(0));
        assert!(a.iter().zip(&b).all(|(x, y)| x.versus(y) != Some(Ordering::Less)), "an item ties itself");
        let heavy = Stat::lower("Weight", String::new(), 500);
        let light = Stat::lower("Weight", String::new(), 200);
        assert_eq!(light.versus(&heavy), Some(Ordering::Greater), "lighter is better");
        let skill = Stat::plain("Skill", "Melee".into());
        assert_eq!(skill.versus(&skill), None);
    }

    fn labels(item: &Item) -> Vec<&'static str> {
        item.stats(0).into_iter().map(|s| s.label).collect()
    }

    fn text(item: &Item, brawn: i32, label: &str) -> String {
        item.stats(brawn).into_iter().find(|s| s.label == label).unwrap().full_text()
    }

    #[test]
    fn melee_and_armor_list_their_stats() {
        let crowbar = Item::one(ItemKind::Melee(MeleeWeapon::named("crowbar")));
        assert_eq!(labels(&crowbar), ["Skill", "Damage", "Accuracy", "Penetration", "Weight"]);
        assert_eq!(text(&crowbar, 0, "Damage"), "2-5");
        assert_eq!(text(&crowbar, 2, "Damage"), "4-7 (2-5, +2 Brawn)");
        assert_eq!(text(&crowbar, -2, "Damage"), "0-3 (2-5, -2 Brawn)");
        let vest = Item::one(ItemKind::Armor(Armor::named("kevlar_vest")));
        assert_eq!(labels(&vest), ["Armor", "Dodge Penalty", "Weight"]);
        assert!(vest.describe().iter().all(|l| !l.contains("Weight")), "weight moves to the stats block");
    }

    #[test]
    fn consumables_list_only_what_they_do() {
        let hardtack = Item::food(Consumable::named("hardtack"), 4);
        assert_eq!(labels(&hardtack), ["Satiety", "Weight"]);
        let weight = &hardtack.stats(0)[1].text;
        assert!(weight.contains(" each)"), "a stack shows the unit weight too: {weight}");
        let tonic = Item::food(Consumable::named("cryptid_tonic"), 1);
        assert_eq!(labels(&tonic), ["Satiety", "Hydration", "Heal", "Cures", "Effects", "", "", "", "Weight"]);
        assert_eq!(text(&tonic, 0, "Effects"), "+2 Brawn for 200 turns");
        assert_eq!(tonic.describe().len(), 1, "just the prose stays above the block");
    }

    #[test]
    fn packs_list_their_capacity() {
        let pack = Item::one(ItemKind::Pack(Pack::named("hiking_pack")));
        assert_eq!(labels(&pack), ["Carry Capacity", "Weight"]);
        assert!(pack.stats(0)[0].text.starts_with('+'));
    }
}
