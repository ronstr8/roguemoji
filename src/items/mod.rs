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
pub mod weight;

pub use armor::Armor;
pub use ballistics::Ammo;
pub use components::Component;
pub use consumables::Consumable;
pub use guns::{Gun, GunModel};
pub use inventory::Inventory;
pub use melee::MeleeWeapon;
pub use packs::Pack;

pub const CRYSTAL_GLYPH: &str = "💎";
pub const AMMO_GLYPH: &str = "🔸";
pub const GUN_GLYPH: &str = "🔫";

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
            ItemKind::Gun(g) => g.def().name.into(),
            ItemKind::Melee(m) => m.def().name.into(),
            ItemKind::Ammo(a) => a.name().into(),
            ItemKind::Armor(a) => a.def().name.into(),
            ItemKind::Component(c) => c.def().name.into(),
            ItemKind::Consumable(c) => c.def().name.into(),
            ItemKind::Pack(p) => p.def().name.into(),
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

    pub fn glyph(&self) -> &'static str {
        match &self.kind {
            ItemKind::Crystals => CRYSTAL_GLYPH,
            ItemKind::Gun(_) => GUN_GLYPH,
            ItemKind::Melee(m) => m.def().glyph,
            ItemKind::Ammo(_) => AMMO_GLYPH,
            ItemKind::Armor(a) => a.def().glyph,
            ItemKind::Component(c) => c.def().glyph,
            ItemKind::Consumable(c) => c.def().glyph,
            ItemKind::Pack(p) => p.def().glyph,
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

    fn weight_line(&self) -> String {
        if self.qty > 1 {
            format!("Weight {} ({} each)", weight::fmt_lb(self.weight()), weight::fmt_lb(self.unit_weight()))
        } else {
            format!("Weight {}", weight::fmt_lb(self.weight()))
        }
    }

    /// Detailed description lines for the inventory pop-up.
    pub fn describe(&self) -> Vec<String> {
        let mut out = Vec::new();
        match &self.kind {
            ItemKind::Crystals => {
                out.push("Something dies, something crystallizes. Absorb to heal; spend at Autovends.".into());
            }
            ItemKind::Gun(g) => {
                let d = g.def();
                out.push(d.desc.into());
                out.push(format!(
                    "{} {} · {}\" barrel · {} · cap {}",
                    d.cartridge.name(),
                    d.action.name(),
                    d.barrel_in,
                    if d.rifled { "rifled" } else { "smoothbore" },
                    d.capacity
                ));
                out.push(format!(
                    "Skill: {} · Accuracy {:+} · Range {} · Reload {} turn(s)",
                    d.skill.name(),
                    d.accuracy,
                    d.effective_range,
                    d.action.reload_turns()
                ));
                for ammo in Ammo::ALL.iter().filter(|a| g.accepts(**a)) {
                    let p = g.ballistics(*ammo);
                    out.push(format!(
                        "  {:<20} {:>4.0} fps {:>5.0} J  dmg ~{:.1}  pen {:+}",
                        ammo.name(),
                        p.fps,
                        p.joules,
                        p.base_damage,
                        p.pen
                    ));
                }
            }
            ItemKind::Melee(m) => {
                let d = m.def();
                out.push(d.desc.into());
                out.push(format!(
                    "Skill: {} · Damage {}-{} (+Brawn) · Accuracy {:+} · Pen {}",
                    d.skill.name(),
                    d.damage.0,
                    d.damage.1,
                    d.accuracy,
                    d.pen
                ));
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
                out.push(d.desc.into());
                out.push(format!("Armor {} · Dodge penalty {}", d.armor, d.dodge_penalty));
            }
            ItemKind::Component(c) => out.push(c.def().desc.into()),
            ItemKind::Consumable(c) => {
                let d = c.def();
                out.push(d.desc.into());
                let mut parts = vec![format!("satiety +{}", d.satiety)];
                if d.heal > 0 {
                    parts.push(format!("heal {}", d.heal));
                }
                if d.cures {
                    parts.push("cures poison & bleeding".into());
                }
                parts.extend(d.effects.iter().map(|e| e.describe()));
                out.push(parts.join("; "));
            }
            ItemKind::Pack(p) => {
                let d = p.def();
                out.push(d.desc.into());
                out.push(format!("Carry capacity +{}", weight::fmt_lb(d.capacity)));
            }
        }
        out.push(self.weight_line());
        out
    }
}
