//! Shared colors.

use crossterm::style::Color;

use crate::creatures::player::Burden;
use crate::game::Tone;
use crate::world::encounter::Mood;

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
/// Encounter glows at full strength, behind the glyph: emoji ignore the
/// foreground color, so the tint has to go in the background.
pub const HOPE_GLOW: Color = rgb(40, 150, 90);
pub const CHANCE_GLOW: Color = rgb(150, 135, 70);
pub const HORROR_GLOW: Color = rgb(165, 60, 60);
/// A color given as red, green and blue in a data file.
pub fn rgb_of((r, g, b): crate::color::Rgb) -> Color {
    rgb(r, g, b)
}

/// The blinding flash of a portal jump.
pub const WHITEOUT: Color = rgb(255, 255, 255);

pub fn burden(b: Burden) -> Color {
    match b {
        Burden::Light => TEXT,
        Burden::Burdened => WARN,
        Burden::Strained | Burden::Overloaded => BAD,
    }
}

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

/// The background behind an encounter, at `strength` from 0 (bare floor) to 1.
pub fn glow(mood: Mood, strength: f32) -> Color {
    let full = match mood {
        Mood::Hope => HOPE_GLOW,
        Mood::Chance => CHANCE_GLOW,
        Mood::Horror => HORROR_GLOW,
    };
    blend(BG, full, strength)
}

/// Mix two colors: `t` of 0 is all `a`, 1 is all `b`.
fn blend(a: Color, b: Color, t: f32) -> Color {
    let (Color::Rgb { r: ar, g: ag, b: ab }, Color::Rgb { r: br, g: bg, b: bb }) = (a, b) else { return b };
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t.clamp(0.0, 1.0)).round() as u8;
    rgb(mix(ar, br), mix(ag, bg), mix(ab, bb))
}
