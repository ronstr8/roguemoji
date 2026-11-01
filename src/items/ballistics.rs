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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cartridge {
    Lr22,
    Spl38,
    Mag357,
    Luger9,
    Acp45,
    Ga20,
    Ga12,
    Nato556,
    Sov762,
    Win3030,
    Win308,
}

impl Cartridge {
    pub fn name(self) -> &'static str {
        match self {
            Cartridge::Lr22 => ".22 LR",
            Cartridge::Spl38 => ".38 Special",
            Cartridge::Mag357 => ".357 Magnum",
            Cartridge::Luger9 => "9mm Luger",
            Cartridge::Acp45 => ".45 ACP",
            Cartridge::Ga20 => "20 gauge",
            Cartridge::Ga12 => "12 gauge",
            Cartridge::Nato556 => "5.56 NATO",
            Cartridge::Sov762 => "7.62x39",
            Cartridge::Win3030 => ".30-30 Win",
            Cartridge::Win308 => ".308 Win",
        }
    }

    /// Whether a chamber for `self` can safely fire `other`.
    /// (.357 Magnum revolvers famously chamber .38 Special.)
    pub fn accepts(self, other: Cartridge) -> bool {
        self == other || (self == Cartridge::Mag357 && other == Cartridge::Spl38)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ammo {
    Lr22,
    Spl38,
    Mag357Jhp,
    Luger9Fmj,
    Luger9Jhp,
    Acp45Fmj,
    Ga20Buck,
    Ga12Buck,
    Ga12Slug,
    Nato556Fmj,
    Sov762Fmj,
    Win3030Sp,
    Win308Sp,
}

pub struct AmmoDef {
    pub cartridge: Cartridge,
    pub load: LoadType,
    pub label: &'static str,
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
}

impl Ammo {
    pub const ALL: [Ammo; 13] = [
        Ammo::Lr22,
        Ammo::Spl38,
        Ammo::Mag357Jhp,
        Ammo::Luger9Fmj,
        Ammo::Luger9Jhp,
        Ammo::Acp45Fmj,
        Ammo::Ga20Buck,
        Ammo::Ga12Buck,
        Ammo::Ga12Slug,
        Ammo::Nato556Fmj,
        Ammo::Sov762Fmj,
        Ammo::Win3030Sp,
        Ammo::Win308Sp,
    ];

    pub fn def(self) -> &'static AmmoDef {
        use Cartridge as C;
        use LoadType as L;
        macro_rules! ammo {
            ($c:expr, $l:expr, $label:expr, $gr:expr, $n:expr, $fps:expr, $bar:expr, $slope:expr, $box:expr) => {
                &AmmoDef {
                    cartridge: $c,
                    load: $l,
                    label: $label,
                    grains: $gr,
                    projectiles: $n,
                    ref_fps: $fps,
                    ref_barrel_in: $bar,
                    fps_per_inch: $slope,
                    box_size: $box,
                }
            };
        }
        match self {
            Ammo::Lr22 => ammo!(C::Lr22, L::LeadRoundNose, ".22 LR 40gr", 40.0, 1, 1235.0, 18.0, 14.0, 20),
            Ammo::Spl38 => ammo!(C::Spl38, L::LeadRoundNose, ".38 Spl 158gr", 158.0, 1, 800.0, 4.0, 25.0, 12),
            Ammo::Mag357Jhp => ammo!(C::Mag357, L::HollowPoint, ".357 Mag 158gr JHP", 158.0, 1, 1240.0, 4.0, 40.0, 12),
            Ammo::Luger9Fmj => ammo!(C::Luger9, L::Fmj, "9mm 115gr FMJ", 115.0, 1, 1180.0, 4.0, 18.0, 15),
            Ammo::Luger9Jhp => ammo!(C::Luger9, L::HollowPoint, "9mm 124gr JHP", 124.0, 1, 1150.0, 4.0, 18.0, 10),
            Ammo::Acp45Fmj => ammo!(C::Acp45, L::Fmj, ".45 ACP 230gr FMJ", 230.0, 1, 850.0, 5.0, 15.0, 10),
            Ammo::Ga20Buck => ammo!(C::Ga20, L::Buckshot, "20ga #3 Buck", 400.0, 20, 1200.0, 26.0, 8.0, 5),
            Ammo::Ga12Buck => ammo!(C::Ga12, L::Buckshot, "12ga 00 Buck", 484.0, 9, 1325.0, 28.0, 8.0, 5),
            Ammo::Ga12Slug => ammo!(C::Ga12, L::Slug, "12ga 1oz Slug", 437.5, 1, 1560.0, 28.0, 8.0, 5),
            Ammo::Nato556Fmj => ammo!(C::Nato556, L::Fmj, "5.56 55gr FMJ", 55.0, 1, 3240.0, 20.0, 30.0, 20),
            Ammo::Sov762Fmj => ammo!(C::Sov762, L::Fmj, "7.62x39 123gr FMJ", 123.0, 1, 2350.0, 20.0, 20.0, 10),
            Ammo::Win3030Sp => ammo!(C::Win3030, L::SoftPoint, ".30-30 150gr SP", 150.0, 1, 2390.0, 24.0, 25.0, 6),
            Ammo::Win308Sp => ammo!(C::Win308, L::SoftPoint, ".308 150gr SP", 150.0, 1, 2820.0, 24.0, 25.0, 5),
        }
    }

    pub fn name(self) -> &'static str {
        self.def().label
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
        let pistol = profile(Ammo::Luger9Fmj, 4.0);
        let carbine = profile(Ammo::Luger9Fmj, 16.0);
        assert!(carbine.fps > pistol.fps);
        assert!(carbine.base_damage > pistol.base_damage);
    }

    #[test]
    fn damage_ordering_is_realistic() {
        let d = |a, b| profile(a, b).base_damage;
        assert!(d(Ammo::Lr22, 4.5) < d(Ammo::Spl38, 2.0));
        assert!(d(Ammo::Spl38, 2.0) < d(Ammo::Luger9Fmj, 4.0));
        assert!(d(Ammo::Luger9Fmj, 4.0) < d(Ammo::Mag357Jhp, 4.0));
        assert!(d(Ammo::Mag357Jhp, 4.0) < d(Ammo::Nato556Fmj, 16.0));
        assert!(d(Ammo::Nato556Fmj, 16.0) < d(Ammo::Win308Sp, 22.0));
    }

    #[test]
    fn magnum_accepts_special() {
        assert!(Cartridge::Mag357.accepts(Cartridge::Spl38));
        assert!(!Cartridge::Spl38.accepts(Cartridge::Mag357));
    }
}
