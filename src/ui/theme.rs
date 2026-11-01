//! Shared colors.

use crossterm::style::Color;

use crate::game::Tone;

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

pub const BG: Color = rgb(18, 16, 14);
pub const PANEL: Color = rgb(28, 25, 22);
pub const POPUP: Color = rgb(34, 30, 27);
pub const SELECTED: Color = rgb(70, 58, 40);
pub const BORDER: Color = rgb(150, 120, 80);
pub const TITLE: Color = rgb(255, 196, 90);
pub const TEXT: Color = rgb(215, 205, 190);
pub const DIM: Color = rgb(120, 112, 100);
pub const MEMORY: Color = rgb(60, 56, 52);
pub const GOOD: Color = rgb(120, 220, 120);
pub const BAD: Color = rgb(240, 100, 90);
pub const WARN: Color = rgb(240, 200, 90);
pub const LOOT: Color = rgb(120, 200, 240);
pub const SPECIAL: Color = rgb(200, 150, 255);
pub const TARGET_BG: Color = rgb(90, 30, 30);
pub const PATH_BG: Color = rgb(50, 30, 30);

pub fn tone(t: Tone) -> Color {
    match t {
        Tone::Info => TEXT,
        Tone::Good => GOOD,
        Tone::Bad => BAD,
        Tone::Combat => WARN,
        Tone::Loot => LOOT,
        Tone::Special => SPECIAL,
    }
}
