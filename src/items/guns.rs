//! Common civilian firearms and per-gun state (rounds loaded).
//!
//! Gun models are loaded from `data/guns/*.toml` (see `data/README.md`).
//! Actions stay in code because they carry rules (reload time).

use serde::Deserialize;

use super::ballistics::{self, Ammo, BallisticProfile, Cartridge, LoadType};
use crate::content::{self, Loot, Price, Problems};
use crate::stats::Skill;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    SemiAuto,
    Revolver,
    Pump,
    Lever,
    Bolt,
    BreakOpen,
}

impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Action::SemiAuto => "semi-auto",
            Action::Revolver => "double-action revolver",
            Action::Pump => "pump-action",
            Action::Lever => "lever-action",
            Action::Bolt => "bolt-action",
            Action::BreakOpen => "single-shot break-action",
        }
    }

    /// Turns spent reloading: detachable magazines and single shells are
    /// quick; loose-loading cylinders and tube magazines take longer.
    pub fn reload_turns(self) -> u32 {
        match self {
            Action::SemiAuto | Action::BreakOpen => 1,
            Action::Bolt => 2,
            Action::Revolver | Action::Pump | Action::Lever => 3,
        }
    }
}

/// A handle to a loaded gun model: an index into the gun table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GunModel(u16);

#[derive(Debug)]
pub struct GunDef {
    /// The file name without `.toml`, e.g. `service_pistol_9mm`.
    pub id: String,
    pub name: String,
    pub cartridge: Cartridge,
    pub action: Action,
    pub barrel_in: f64,
    pub capacity: u32,
    /// Inherent mechanical accuracy and sight quality, as a to-hit modifier.
    pub accuracy: i32,
    /// Tiles before range penalties begin.
    pub effective_range: i32,
    pub rifled: bool,
    pub skill: Skill,
    pub desc: String,
    /// Hundredths of a pound, unloaded.
    pub weight: u32,
    pub loot: Loot,
    /// Autovend price; never sold if zero.
    pub price: Price,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGun {
    name: String,
    description: String,
    /// Pounds, unloaded.
    weight: f64,
    cartridge: String,
    action: Action,
    skill: Skill,
    barrel_in: f64,
    capacity: u32,
    #[serde(default)]
    accuracy: i32,
    effective_range: i32,
    rifled: bool,
    #[serde(default)]
    loot: Loot,
    #[serde(default)]
    price: Price,
}

fn build(id: String, raw: RawGun) -> Result<GunDef, Vec<String>> {
    let mut p = Problems::default();
    let cartridge = Cartridge::lookup(&raw.cartridge);
    p.check(cartridge.is_some(), format!("cartridge `{}` is not in data/cartridges/", raw.cartridge));
    if let Some(c) = cartridge {
        p.check(
            Ammo::all().any(|a| c.accepts(a.def().cartridge)),
            format!("no ammo in data/ammo/ fits `{}`", raw.cartridge),
        );
    }
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(raw.weight >= 0.0 && raw.weight.is_finite(), "weight must be zero or more pounds");
    p.check(matches!(raw.skill, Skill::SmallArms | Skill::LongArms), "skill must be small_arms or long_arms");
    p.check(raw.barrel_in > 0.0, "barrel_in must be positive");
    p.check(raw.capacity >= 1, "capacity must be at least 1");
    p.check(raw.effective_range >= 1, "effective_range must be at least 1");
    match cartridge {
        Some(cartridge) => p.finish(|| GunDef {
            id,
            name: raw.name,
            cartridge,
            action: raw.action,
            barrel_in: raw.barrel_in,
            capacity: raw.capacity,
            accuracy: raw.accuracy,
            effective_range: raw.effective_range,
            rifled: raw.rifled,
            skill: raw.skill,
            desc: raw.description,
            weight: (raw.weight * 100.0).round() as u32,
            loot: raw.loot,
            price: raw.price,
        }),
        None => Err(p.0),
    }
}

content::registry!(GunModel, GunDef, "gun", load);

fn load() -> Result<Vec<GunDef>, String> {
    let defs = content::load_defs("guns", build)?;
    let find = |id: &str| defs.iter().find(|d| d.id == id);
    let mut errors = Vec::new();
    for b in crate::creatures::Background::ALL {
        let Some((gun, ammo, _)) = b.starting_gear().gun else { continue };
        match (find(gun), Ammo::lookup(ammo)) {
            (None, _) => errors.push(format!("data/guns/{gun}.toml is missing, but the game uses `{gun}`")),
            (Some(g), Some(a)) if !g.cartridge.accepts(a.def().cartridge) => {
                errors.push(format!("the {} starting kit loads `{ammo}` into `{gun}`, which doesn't fit", b.name()))
            }
            _ => {}
        }
    }
    if errors.is_empty() { Ok(defs) } else { Err(errors.join("\n")) }
}

/// A firearm instance with its current loaded rounds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gun {
    pub model: GunModel,
    pub loaded: u32,
    pub ammo: Option<Ammo>,
}

impl Gun {
    pub fn loaded_with(model: GunModel, ammo: Ammo, rounds: u32) -> Self {
        Self { model, loaded: rounds.min(model.def().capacity), ammo: Some(ammo) }
    }

    pub fn def(&self) -> &'static GunDef {
        self.model.def()
    }

    pub fn accepts(&self, ammo: Ammo) -> bool {
        self.def().cartridge.accepts(ammo.def().cartridge)
    }

    pub fn ballistics(&self, ammo: Ammo) -> BallisticProfile {
        ballistics::profile(ammo, self.def().barrel_in)
    }

    /// Effective range for a given load. Buckshot patterns open fast; a slug
    /// from a smoothbore barrel is usable a bit further than buck.
    pub fn effective_range(&self, ammo: Ammo) -> i32 {
        let base = self.def().effective_range;
        match ammo.def().load {
            LoadType::Buckshot => base.min(5),
            LoadType::Slug if !self.def().rifled => base.max(8),
            _ => base,
        }
    }

    /// To-hit modifier at a given distance in tiles.
    pub fn hit_modifier(&self, ammo: Ammo, distance: i32) -> i32 {
        let def = self.def();
        let load = ammo.def().load;
        let mut m = def.accuracy;
        // A spreading pattern is forgiving up close.
        if load == LoadType::Buckshot && distance <= 4 {
            m += 2;
        }
        // Full-bore slugs through a smoothbore are less precise.
        if load == LoadType::Slug && !def.rifled {
            m -= 1;
        }
        // A long gun is awkward to bring to bear on something at arm's length.
        if distance <= 1 && def.skill == Skill::LongArms {
            m -= 2;
        }
        let over = distance - self.effective_range(ammo);
        if over > 0 {
            // Handguns degrade faster past their range than shoulder arms.
            let per_tile = if def.skill == Skill::SmallArms { 2 } else { 1 };
            m -= over * per_tile;
        }
        m
    }

    /// Fraction of a shot's projectiles that land at `distance`.
    pub fn pellet_fraction(ammo: Ammo, distance: i32) -> f64 {
        if ammo.def().projectiles <= 1 {
            return 1.0;
        }
        (1.0 - (distance - 2).max(0) as f64 * 0.14).clamp(0.2, 1.0)
    }

    pub fn status(&self) -> String {
        let def = self.def();
        match self.ammo {
            Some(a) if self.loaded > 0 => {
                format!("{}/{} {}", self.loaded, def.capacity, a.def().load.abbrev())
            }
            _ => format!("0/{}", def.capacity),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_guns_load() {
        load().expect("stock gun data is valid");
        let sks = GunModel::named("sks").def();
        assert_eq!((sks.capacity, sks.loot.weight_at(2), sks.loot.weight_at(3)), (10, 0, 5));
    }

    #[test]
    fn snub_is_inaccurate_at_range() {
        let (spl, w308) = (Ammo::named("38_special"), Ammo::named("308_win_sp"));
        let snub = Gun::loaded_with(GunModel::named("snub_revolver_38"), spl, 5);
        let rifle = Gun::loaded_with(GunModel::named("bolt_rifle_308"), w308, 4);
        assert!(snub.hit_modifier(spl, 8) < rifle.hit_modifier(w308, 8));
    }

    #[test]
    fn buckshot_falls_off() {
        let (buck, slug) = (Ammo::named("12ga_buck"), Ammo::named("12ga_slug"));
        assert_eq!(Gun::pellet_fraction(buck, 1), 1.0);
        assert!(Gun::pellet_fraction(buck, 6) < 0.5);
        assert_eq!(Gun::pellet_fraction(slug, 9), 1.0);
    }
}
