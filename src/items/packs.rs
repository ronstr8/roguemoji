//! Packs: worn carrying gear that raises how much you can haul before the
//! load starts slowing you down.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pack {
    RagSatchel,
    SchoolBackpack,
    HikingPack,
    FramePack,
    GravHarness,
}

pub struct PackDef {
    pub name: &'static str,
    pub glyph: &'static str,
    /// Extra carrying capacity, in hundredths of a pound.
    pub capacity: u32,
    pub desc: &'static str,
}

impl Pack {
    pub fn def(self) -> &'static PackDef {
        macro_rules! p {
            ($n:expr, $g:expr, $lb:expr, $d:expr) => {
                &PackDef { name: $n, glyph: $g, capacity: $lb * 100, desc: $d }
            };
        }
        match self {
            Pack::RagSatchel => p!("Rag Satchel", "👝", 8, "Knotted rags and tape. It holds, mostly."),
            Pack::SchoolBackpack => p!("School Backpack", "🎒", 15, "Glitter unicorn print. Solid zippers."),
            Pack::HikingPack => p!("Hiking Pack", "🎒", 25, "Hip belt, sternum strap, a dozen pockets."),
            Pack::FramePack => p!("Frame Pack", "🎒", 35, "Aluminum external frame. Built for hauling elk."),
            Pack::GravHarness => {
                p!("Grav Harness", "💠", 50, "Visitor straps that hum. Whatever you clip on weighs less.")
            }
        }
    }
}
