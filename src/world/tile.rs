//! Terrain tiles. Each renders as a two-column cell so it lines up with emoji.

use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    /// Concrete and brick of ruined buildings.
    Wall,
    /// Natural rock and packed earth.
    Rock,
    Floor,
    Asphalt,
    Rubble,
    Grass,
    Water,
    /// Glowing toxic runoff: poisons whoever wades in.
    Sludge,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Wall | Tile::Rock)
    }

    pub fn opaque(self) -> bool {
        matches!(self, Tile::Wall | Tile::Rock)
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Tile::Wall => "▓▓",
            Tile::Rock => "▒▒",
            Tile::Floor => "· ",
            Tile::Asphalt => "░░",
            Tile::Rubble => "∴ ",
            Tile::Grass => "„ ",
            Tile::Water => "≈≈",
            Tile::Sludge => "≈≈",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Tile::Wall => Color::Rgb { r: 140, g: 130, b: 120 },
            Tile::Rock => Color::Rgb { r: 120, g: 95, b: 70 },
            Tile::Floor => Color::Rgb { r: 110, g: 110, b: 110 },
            Tile::Asphalt => Color::Rgb { r: 55, g: 55, b: 60 },
            Tile::Rubble => Color::Rgb { r: 150, g: 130, b: 100 },
            Tile::Grass => Color::Rgb { r: 110, g: 150, b: 70 },
            Tile::Water => Color::Rgb { r: 70, g: 120, b: 200 },
            Tile::Sludge => Color::Rgb { r: 140, g: 230, b: 60 },
        }
    }
}
