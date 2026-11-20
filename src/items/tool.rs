//! Tools: items a task needs on hand but doesn't use up, unless it breaks.

use super::{Component, ItemKind, MeleeWeapon};

/// An item named as a tool in data: a component (lockpicks) or a melee weapon
/// you can also work with (a crowbar, a shovel).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Component(Component),
    Melee(MeleeWeapon),
}

impl Tool {
    /// The tool with this item id, if a component or melee weapon has it.
    pub fn lookup(id: &str) -> Option<Tool> {
        Component::lookup(id).map(Tool::Component).or_else(|| MeleeWeapon::lookup(id).map(Tool::Melee))
    }

    /// How one of it sits in a pack.
    pub fn kind(self) -> ItemKind {
        match self {
            Tool::Component(c) => ItemKind::Component(c),
            Tool::Melee(m) => ItemKind::Melee(m),
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Tool::Component(c) => &c.def().glyph,
            Tool::Melee(m) => &m.def().glyph,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Tool::Component(c) => &c.def().name,
            Tool::Melee(m) => &m.def().name,
        }
    }
}
