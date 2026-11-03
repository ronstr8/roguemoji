//! Cartridges, ammunition loads and the physics that turns them into damage.
//!
//! Damage is derived, not hand-tuned: each load has a real-world projectile
//! mass and a reference muzzle velocity at a reference barrel length. A gun's
//! barrel length shifts that velocity (rimfire and pistol rounds gain little,
//! magnums and rifle rounds gain a lot), giving muzzle energy in joules.
//!
//! Game damage scales with the *square root* of energy. Raw energy spans a
//! ~25x range from a .22 pistol to a .308 rifle, but wounding effect does not;
//! the square root compresses that to ~5x, which keeps the ordering faithful
//! while letting a .22 still matter against a rat.
//!
//! Cartridges and loads are content: `data/cartridges/*.toml` and
//! `data/ammo/*.toml` (see `data/README.md`). Load types stay in code because
//! they carry rules.

use serde::Deserialize;

use crate::content::{self, Price, Problems};

// ── Cartridges ──

/// A handle to a loaded cartridge: an index into the cartridge table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Cartridge(u16);

#[derive(Debug)]
pub struct CartridgeDef {
    /// The file name without `.toml`, e.g. `9mm`.
    pub id: String,
    pub name: String,
    /// Weight of one round, in hundredths of a pound.
    pub round_weight: u32,
    /// Other cartridges a chamber for this one can safely fire.
    pub also_fires: Vec<Cartridge>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCartridge {
    name: String,
    /// Pounds per round.
    round_weight: f64,
    #[serde(default)]
    also_fires: Vec<String>,
}

fn build_cartridge(id: String, raw: RawCartridge) -> Result<(CartridgeDef, Vec<String>), Vec<String>> {
    let mut p = Problems::default();
    p.check(!raw.name.trim().is_empty(), "name is empty");
    p.check(raw.round_weight >= 0.0 && raw.round_weight.is_finite(), "round_weight must be zero or more pounds");
    let weight = (raw.round_weight * 100.0).round() as u32;
    p.finish(|| (CartridgeDef { id, name: raw.name, round_weight: weight, also_fires: Vec::new() }, raw.also_fires))
}

fn load_cartridges() -> Result<Vec<CartridgeDef>, String> {
    let loaded = content::load_defs("cartridges", build_cartridge)?;
    // `also_fires` names other cartridges in this same table, so resolve it
    // here rather than through `Cartridge::lookup` (the table isn't ready yet).
    let index = |id: &str| loaded.iter().position(|(d, _)| d.id == id);
    let mut errors = Vec::new();
    let mut resolved = Vec::new();
    for (def, names) in &loaded {
        let mut fires = Vec::new();
        for n in names {
            match index(n) {
                Some(i) => fires.push(Cartridge(i as u16)),
                None => errors.push(format!("cartridge `{}`: also_fires `{n}` is not a cartridge", def.id)),
            }
        }
        resolved.push(fires);
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(loaded
        .into_iter()
        .zip(resolved)
        .map(|((mut d, _), fires)| {
            d.also_fires = fires;
            d
        })
        .collect())
}

mod cartridge_registry {
    use super::*;
    content::registry!(Cartridge, CartridgeDef, "cartridge", load_cartridges);
}
pub use cartridge_registry::init as init_cartridges;

impl Cartridge {
    pub fn name(self) -> &'static str {
        &self.def().name
    }

    /// Whether a chamber for `self` can safely fire `other`.
    /// (.357 Magnum revolvers famously chamber .38 Special.)
    pub fn accepts(self, other: Cartridge) -> bool {
        self == other || self.def().also_fires.contains(&other)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadType {
    LeadRoundNose,
    Fmj,
    HollowPoint,
    SoftPoint,
    Buckshot,
    Slug,
}

impl LoadType {
    pub fn abbrev(self) -> &'static str {
        match self {
            LoadType::LeadRoundNose => "LRN",
            LoadType::Fmj => "FMJ",
            LoadType::HollowPoint => "JHP",
            LoadType::SoftPoint => "SP",
            LoadType::Buckshot => "Buck",
            LoadType::Slug => "Slug",
        }
    }

    /// Expanding bullets wound more; jacketed solids penetrate more.
    pub fn damage_mult(self) -> f64 {
        match self {
            LoadType::LeadRoundNose => 0.95,
            LoadType::Fmj => 1.0,
            LoadType::HollowPoint => 1.25,
            LoadType::SoftPoint => 1.2,
            LoadType::Buckshot => 1.0,
            LoadType::Slug => 1.1,
        }
    }

    pub fn pen_mod(self) -> i32 {
        match self {
            LoadType::Fmj | LoadType::Slug => 1,
            LoadType::HollowPoint | LoadType::Buckshot => -1,
            LoadType::LeadRoundNose | LoadType::SoftPoint => 0,
        }
    }
}

// ── Ammunition ──

/// A handle to a loaded ammunition type: an index into the ammo table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ammo(u16);

#[derive(Debug)]
pub struct AmmoDef {
    /// The file name without `.toml`, e.g. `9mm_fmj`.
    pub id: String,
    pub cartridge: Cartridge,
    pub load: LoadType,
    pub label: String,
    /// Total projectile mass in grains (all pellets for shot).
    pub grains: f64,
    pub projectiles: u32,
    /// Factory-published velocity at the reference barrel length.
    pub ref_fps: f64,
    pub ref_barrel_in: f64,
    /// Approximate velocity change per inch of barrel around the reference.
    pub fps_per_inch: f64,
    /// Rounds per box when found.
    pub box_size: u32,
    /// Autovend price of a full box; never sold if zero.
    pub price: Price,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAmmo {
    label: String,
    cartridge: String,
    load: LoadType,
    box_size: u32,
    grains: f64,
    #[serde(default = "one")]
    projectiles: u32,
    ref_fps: f64,
    ref_barrel_in: f64,
    fps_per_inch: f64,
    #[serde(default)]
    price: Price,
}

fn one() -> u32 {
    1
}

fn build_ammo(id: String, raw: RawAmmo) -> Result<AmmoDef, Vec<String>> {
    let mut p = Problems::default();
    let cartridge = Cartridge::lookup(&raw.cartridge);
    p.check(cartridge.is_some(), format!("cartridge `{}` is not in data/cartridges/", raw.cartridge));
    p.check(!raw.label.trim().is_empty(), "label is empty");
    p.check(raw.box_size >= 1, "box_size must be at least 1");
    p.check(raw.grains > 0.0, "grains must be positive");
    p.check(raw.projectiles >= 1, "projectiles must be at least 1");
    p.check(raw.ref_fps > 0.0, "ref_fps must be positive");
    p.check(raw.ref_barrel_in > 0.0, "ref_barrel_in must be positive");
    p.check(raw.fps_per_inch >= 0.0, "fps_per_inch cannot be negative");
    match cartridge {
        Some(cartridge) => p.finish(|| AmmoDef {
            id,
            cartridge,
            load: raw.load,
            label: raw.label,
            grains: raw.grains,
            projectiles: raw.projectiles,
            ref_fps: raw.ref_fps,
            ref_barrel_in: raw.ref_barrel_in,
            fps_per_inch: raw.fps_per_inch,
            box_size: raw.box_size,
            price: raw.price,
        }),
        None => Err(p.0),
    }
}

fn load_ammo() -> Result<Vec<AmmoDef>, String> {
    let defs = content::load_defs("ammo", build_ammo)?;
    let kits = crate::creatures::Background::ALL.into_iter().flat_map(|b| b.starting_gear().ammo_ids());
    content::require("ammo", |id| defs.iter().any(|d| d.id == id), kits)?;
    Ok(defs)
}

mod ammo_registry {
    use super::*;
    content::registry!(Ammo, AmmoDef, "ammo", load_ammo);
}
pub use ammo_registry::init as init_ammo;

impl Ammo {
    pub fn name(self) -> &'static str {
        &self.def().label
    }
}

/// Everything needed to resolve a shot from a specific ammo/barrel combination.
#[derive(Clone, Copy, Debug)]
pub struct BallisticProfile {
    pub fps: f64,
    pub joules: f64,
    /// Expected damage if every projectile hits.
    pub base_damage: f64,
    pub pen: i32,
    pub projectiles: u32,
}

pub fn muzzle_velocity_fps(ammo: &AmmoDef, barrel_in: f64) -> f64 {
    let v = ammo.ref_fps + (barrel_in - ammo.ref_barrel_in) * ammo.fps_per_inch;
    v.clamp(ammo.ref_fps * 0.6, ammo.ref_fps * 1.25)
}

pub fn muzzle_energy_joules(grains: f64, fps: f64) -> f64 {
    const KG_PER_GRAIN: f64 = 0.000_064_798_91;
    const MPS_PER_FPS: f64 = 0.3048;
    let m = grains * KG_PER_GRAIN;
    let v = fps * MPS_PER_FPS;
    0.5 * m * v * v
}

/// Divisor that maps sqrt(joules) onto the game's hit-point scale.
const DAMAGE_SCALE: f64 = 3.0;

pub fn profile(ammo: Ammo, barrel_in: f64) -> BallisticProfile {
    let def = ammo.def();
    let fps = muzzle_velocity_fps(def, barrel_in);
    let joules = muzzle_energy_joules(def.grains, fps);
    let velocity_pen = if fps >= 2000.0 {
        2
    } else if fps >= 1400.0 {
        1
    } else {
        0
    };
    BallisticProfile {
        fps,
        joules,
        base_damage: joules.sqrt() / DAMAGE_SCALE * def.load.damage_mult(),
        pen: velocity_pen + def.load.pen_mod(),
        projectiles: def.projectiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_matches_published_figures() {
        // 9mm 115gr @ 1180 fps is ~480 J; .308 150gr @ 2820 fps is ~3570 J.
        let nine = muzzle_energy_joules(115.0, 1180.0);
        assert!((nine - 482.0).abs() < 10.0, "{nine}");
        let rifle = muzzle_energy_joules(150.0, 2820.0);
        assert!((rifle - 3586.0).abs() < 40.0, "{rifle}");
    }

    #[test]
    fn longer_barrels_are_faster() {
        let pistol = profile(Ammo::named("9mm_fmj"), 4.0);
        let carbine = profile(Ammo::named("9mm_fmj"), 16.0);
        assert!(carbine.fps > pistol.fps);
        assert!(carbine.base_damage > pistol.base_damage);
    }

    #[test]
    fn damage_ordering_is_realistic() {
        let d = |a, b| profile(a, b).base_damage;
        assert!(d(Ammo::named("22lr"), 4.5) < d(Ammo::named("38_special"), 2.0));
        assert!(d(Ammo::named("38_special"), 2.0) < d(Ammo::named("9mm_fmj"), 4.0));
        assert!(d(Ammo::named("9mm_fmj"), 4.0) < d(Ammo::named("357_magnum_jhp"), 4.0));
        assert!(d(Ammo::named("357_magnum_jhp"), 4.0) < d(Ammo::named("556_nato_fmj"), 16.0));
        assert!(d(Ammo::named("556_nato_fmj"), 16.0) < d(Ammo::named("308_win_sp"), 22.0));
    }

    #[test]
    fn stock_ammo_loads() {
        load_cartridges().expect("stock cartridge data is valid");
        load_ammo().expect("stock ammo data is valid");
        assert_eq!(Ammo::named("12ga_buck").def().projectiles, 9);
    }

    #[test]
    fn magnum_accepts_special() {
        let (mag, spl) = (Cartridge::named("357_magnum"), Cartridge::named("38_special"));
        assert!(mag.accepts(spl));
        assert!(!spl.accepts(mag));
    }
}
