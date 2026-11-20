//! Rendering. Views read game state and draw onto a `Canvas`; they never
//! mutate the game.

pub mod canvas;
mod hud;
mod map_view;
pub mod screens;
pub mod terminal;
pub mod theme;

use canvas::Canvas;

use crate::game::Game;
use crate::geometry::Pos;

/// The game's name as drawn on screen, spaced a letter apart.
// ☢ without its variation selector, which the canvas would drop anyway. The
// canvas counts ☢ as one column, but some terminals draw it two wide, so the
// extra space after it keeps it from crowding the j.
pub const TITLE: &str = "r 💀 g u e m ☢  j i";

pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;
const SIDEBAR_WIDTH: u16 = 38;
const LOG_HEIGHT: u16 = 7;

#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Area {
    pub const fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

/// Timed effects on the map.
#[derive(Clone, Copy, Debug)]
pub struct Effects {
    /// A tile to draw an impact burst on.
    pub flash: Option<Pos>,
    /// How strongly encounters glow right now, from 0 to 1.
    pub glow: f32,
    /// Milliseconds since the app started, for blinking faces.
    pub clock_ms: u128,
}

/// Draw the main play view: map, sidebar and message log.
pub fn draw_play(canvas: &mut Canvas, game: &Game, target: Option<Pos>, fx: Effects, status_line: Option<&str>) {
    let (w, h) = (canvas.width(), canvas.height());
    let map_w = w.saturating_sub(SIDEBAR_WIDTH);
    let map_h = h.saturating_sub(LOG_HEIGHT);
    map_view::draw(canvas, Area::new(0, 0, map_w, map_h), game, target, fx);
    hud::draw_sidebar(canvas, Area::new(map_w, 0, SIDEBAR_WIDTH, map_h), game);
    hud::draw_log(canvas, Area::new(0, map_h, w, LOG_HEIGHT), game, status_line);
}

pub fn draw_too_small(canvas: &mut Canvas) {
    let msg = format!("roguemoji needs at least {MIN_WIDTH}x{MIN_HEIGHT}. Please enlarge the terminal.");
    canvas.text(0, 0, &msg, theme::WARN, theme::BG);
}
