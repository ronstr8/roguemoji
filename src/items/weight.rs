//! Item weights. All weights are in hundredths of a pound so that a single
//! .22 round (about 0.01 lb) still counts.

use super::ballistics::Cartridge;
use super::{Ammo, Armor, Component, Consumable, Gun, GunModel, ItemKind, MeleeWeapon, Pack};

/// Format a weight in hundredths of a pound as e.g. `12.5 lb`.
pub fn fmt_lb(w: u32) -> String {
    format!("{:.1} lb", w as f64 / 100.0)
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
    use GunModel::*;
    match m {
        TargetPistol22 => 250,
        SnubRevolver38 => 140,
        CompactPistol9 => 160,
        ServicePistol9 => 190,
        Pistol1911 => 240,
        Revolver357 => 280,
        Rifle22 => 500,
        Carbine9 => 650,
        SingleShot20 => 550,
        PumpShotgun12 => 750,
        LeverAction3030 => 700,
        Sks => 850,
        Ar15 => 700,
        BoltRifle308 => 900,
    }
}

pub fn ammo(a: Ammo) -> u32 {
    match a.def().cartridge {
        Cartridge::Lr22 => 1,
        Cartridge::Spl38 | Cartridge::Mag357 | Cartridge::Luger9 | Cartridge::Nato556 => 3,
        Cartridge::Acp45 | Cartridge::Sov762 => 4,
        Cartridge::Win3030 => 5,
        Cartridge::Win308 => 6,
        Cartridge::Ga20 => 7,
        Cartridge::Ga12 => 10,
    }
}

pub fn melee(m: MeleeWeapon) -> u32 {
    use MeleeWeapon::*;
    match m {
        KitchenKnife => 50,
        ClawHammer => 150,
        Crowbar => 500,
        PipeWrench => 600,
        BaseballBat => 200,
        Machete => 150,
        Shovel => 550,
        FireAxe => 600,
        BrassKnuckles => 50,
        NailBat => 250,
        PipeSpear => 450,
        ScrapCleaver => 400,
        ShockBaton => 300,
        SpikedKnuckles => 80,
        PhaseEdge => 100,
        // "Weightless in your hands, mountainous on impact."
        GravitonMaul => 0,
        HumFilament => 20,
        SiphonGauntlet => 150,
    }
}

pub fn armor(a: Armor) -> u32 {
    match a {
        Armor::LeatherJacket => 400,
        Armor::ScrapVest => 1400,
        Armor::KevlarVest => 800,
        Armor::RiotGear => 2200,
        Armor::VisitorWeave => 150,
    }
}

pub fn component(c: Component) -> u32 {
    use Component::*;
    match c {
        Herbs | Glowcap | CryptidHair => 10,
        CactusPulp | Bone | Wire | Battery => 50,
        Grain | Mutagen | DuctTape => 30,
        DirtyWater => 200,
        Sugar => 5,
        CannedBeans | RawMeat => 100,
        ScrapMetal => 300,
        Pipe => 400,
        Nails | Circuit | Rags => 20,
    }
}

pub fn consumable(c: Consumable) -> u32 {
    use Consumable::*;
    match c {
        Painkillers => 10,
        Hardtack => 30,
        GrilledMeat => 80,
        HerbalTea | GlowcapBrew | CactusJuice | Moonshine | StimSoda | CryptidTonic | CannedPeaches => 100,
        BeanChili | MutantStew | BoneBroth => 120,
        BottledWater => 200,
    }
}

pub fn pack(p: Pack) -> u32 {
    match p {
        Pack::RagSatchel => 50,
        Pack::SchoolBackpack => 100,
        Pack::HikingPack => 300,
        Pack::FramePack => 500,
        Pack::GravHarness => 0,
    }
}
