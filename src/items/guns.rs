//! Common civilian firearms and per-gun state (rounds loaded).

use super::ballistics::{self, Ammo, BallisticProfile, Cartridge, LoadType};
use crate::stats::Skill;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GunModel {
    TargetPistol22,
    SnubRevolver38,
    CompactPistol9,
    ServicePistol9,
    Pistol1911,
    Revolver357,
    Rifle22,
    Carbine9,
    SingleShot20,
    PumpShotgun12,
    LeverAction3030,
    Sks,
    Ar15,
    BoltRifle308,
}

pub struct GunDef {
    pub name: &'static str,
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
    pub desc: &'static str,
}

impl GunModel {
    pub fn def(self) -> &'static GunDef {
        use Action::*;
        use Cartridge as C;
        use Skill::{LongArms, SmallArms};
        macro_rules! gun {
            ($name:expr, $c:expr, $act:expr, $bar:expr, $cap:expr, $acc:expr, $rng:expr, $rifled:expr, $sk:expr, $desc:expr) => {
                &GunDef {
                    name: $name,
                    cartridge: $c,
                    action: $act,
                    barrel_in: $bar,
                    capacity: $cap,
                    accuracy: $acc,
                    effective_range: $rng,
                    rifled: $rifled,
                    skill: $sk,
                    desc: $desc,
                }
            };
        }
        match self {
            GunModel::TargetPistol22 => gun!(
                ".22 Target Pistol",
                C::Lr22,
                SemiAuto,
                5.5,
                10,
                2,
                6,
                true,
                SmallArms,
                "Long slim barrel and adjustable sights. Accurate, quiet, and weak."
            ),
            GunModel::SnubRevolver38 => gun!(
                ".38 Snub Revolver",
                C::Spl38,
                Revolver,
                2.0,
                5,
                -2,
                3,
                true,
                SmallArms,
                "Five shots, two-inch barrel, a heavy trigger. A pocket gun, not a fight gun."
            ),
            GunModel::CompactPistol9 => gun!(
                "9mm Compact Pistol",
                C::Luger9,
                SemiAuto,
                3.5,
                12,
                0,
                5,
                true,
                SmallArms,
                "The most common carry pistol in the old world. Short sight radius."
            ),
            GunModel::ServicePistol9 => gun!(
                "9mm Service Pistol",
                C::Luger9,
                SemiAuto,
                4.5,
                15,
                1,
                6,
                true,
                SmallArms,
                "Full-size polymer duty pistol. Reliable and high capacity."
            ),
            GunModel::Pistol1911 => gun!(
                ".45 1911 Pistol",
                C::Acp45,
                SemiAuto,
                5.0,
                7,
                1,
                6,
                true,
                SmallArms,
                "A century-old steel design with a crisp trigger and a slow, heavy bullet."
            ),
            GunModel::Revolver357 => gun!(
                ".357 Revolver",
                C::Mag357,
                Revolver,
                4.0,
                6,
                1,
                6,
                true,
                SmallArms,
                "Six magnum rounds and a loud bark. Will also chamber .38 Special."
            ),
            GunModel::Rifle22 => gun!(
                ".22 Rifle",
                C::Lr22,
                SemiAuto,
                18.0,
                10,
                3,
                10,
                true,
                LongArms,
                "A farmhouse plinker. Barely kicks, very accurate."
            ),
            GunModel::Carbine9 => gun!(
                "9mm Carbine",
                C::Luger9,
                SemiAuto,
                16.0,
                15,
                2,
                9,
                true,
                LongArms,
                "Pistol-caliber carbine. The long barrel squeezes extra speed from 9mm."
            ),
            GunModel::SingleShot20 => gun!(
                "20ga Single-Shot",
                C::Ga20,
                BreakOpen,
                26.0,
                1,
                0,
                5,
                false,
                LongArms,
                "A hinge, a hammer and one shell. The classic first shotgun."
            ),
            GunModel::PumpShotgun12 => gun!(
                "12ga Pump Shotgun",
                C::Ga12,
                Pump,
                18.5,
                5,
                0,
                5,
                false,
                LongArms,
                "Short-barreled home-defense pump. Smoothbore: buckshot or slugs."
            ),
            GunModel::LeverAction3030 => gun!(
                ".30-30 Lever Rifle",
                C::Win3030,
                Lever,
                20.0,
                6,
                2,
                12,
                true,
                LongArms,
                "The deer rifle of a dozen generations. Tube-fed, quick-handling."
            ),
            GunModel::Sks => gun!(
                "SKS Rifle",
                C::Sov762,
                SemiAuto,
                20.5,
                10,
                2,
                12,
                true,
                LongArms,
                "Surplus semi-auto with a fixed ten-round magazine. Crude but rugged."
            ),
            GunModel::Ar15 => gun!(
                "AR-15 Rifle",
                C::Nato556,
                SemiAuto,
                16.0,
                30,
                3,
                14,
                true,
                LongArms,
                "Light, modular, flat-shooting. Small fast bullets."
            ),
            GunModel::BoltRifle308 => gun!(
                ".308 Bolt Rifle",
                C::Win308,
                Bolt,
                22.0,
                4,
                4,
                18,
                true,
                LongArms,
                "Scoped hunting rifle. Slow to cycle, devastating on target."
            ),
        }
    }
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
    fn snub_is_inaccurate_at_range() {
        let snub = Gun::loaded_with(GunModel::SnubRevolver38, Ammo::Spl38, 5);
        let rifle = Gun::loaded_with(GunModel::BoltRifle308, Ammo::Win308Sp, 4);
        assert!(snub.hit_modifier(Ammo::Spl38, 8) < rifle.hit_modifier(Ammo::Win308Sp, 8));
    }

    #[test]
    fn buckshot_falls_off() {
        assert_eq!(Gun::pellet_fraction(Ammo::Ga12Buck, 1), 1.0);
        assert!(Gun::pellet_fraction(Ammo::Ga12Buck, 6) < 0.5);
        assert_eq!(Gun::pellet_fraction(Ammo::Ga12Slug, 9), 1.0);
    }
}
