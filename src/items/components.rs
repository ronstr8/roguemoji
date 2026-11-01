//! Crafting components gathered by harvesting and from creature remains.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Component {
    Herbs,
    Glowcap,
    CactusPulp,
    Grain,
    DirtyWater,
    Sugar,
    CannedBeans,
    RawMeat,
    Bone,
    Mutagen,
    CryptidHair,
    ScrapMetal,
    Pipe,
    Wire,
    DuctTape,
    Nails,
    Battery,
    Circuit,
    Rags,
}

pub struct ComponentDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub desc: &'static str,
}

impl Component {
    pub fn def(self) -> &'static ComponentDef {
        macro_rules! c {
            ($n:expr, $g:expr, $d:expr) => {
                &ComponentDef { name: $n, glyph: $g, desc: $d }
            };
        }
        match self {
            Component::Herbs => c!("Wild Herbs", "🌿", "Yarrow, mint and something you hope is sage."),
            Component::Glowcap => c!("Glowcap", "🍄", "A faintly luminous mushroom. Sharpens the mind."),
            Component::CactusPulp => c!("Cactus Pulp", "🌵", "Wet, bitter, full of electrolytes."),
            Component::Grain => c!("Wild Grain", "🌾", "Feral wheat gone to seed."),
            Component::DirtyWater => c!("Dirty Water", "💧", "Boil it first. Probably."),
            Component::Sugar => c!("Sugar Packets", "🍬", "Diner sugar, still sealed."),
            Component::CannedBeans => c!("Canned Beans", "🥫", "Dented but intact."),
            Component::RawMeat => c!("Raw Meat", "🥩", "Fresh from a recent kill."),
            Component::Bone => c!("Bone", "🦴", "Good for broth, or for crafting."),
            Component::Mutagen => c!("Mutagen Gland", "🧪", "A pulsing gland from a mutated creature."),
            Component::CryptidHair => c!("Cryptid Hair", "🧶", "Coarse, warm, impossibly strong fibers."),
            Component::ScrapMetal => c!("Scrap Metal", "🔩", "Sheet steel, bolts and brackets."),
            Component::Pipe => c!("Steel Pipe", "🦯", "Three feet of threaded pipe."),
            Component::Wire => c!("Copper Wire", "➰", "Stripped from walls and wrecks."),
            Component::DuctTape => c!("Duct Tape", "🩹", "Holds the world together."),
            Component::Nails => c!("Nails", "📌", "A handful of framing nails."),
            Component::Battery => c!("Battery", "🔋", "Still holds a charge."),
            Component::Circuit => c!("Circuit Board", "💾", "Salvaged electronics."),
            Component::Rags => c!("Rags", "🧣", "Cloth scraps for wrapping and padding."),
        }
    }
}
