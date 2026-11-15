//! Shoot pop-up: pick which creature in range to fire at, nearest first.
//!
//! It sits over the sidebar rather than the middle of the screen, and doesn't
//! dim the map, so the highlighted line of fire stays visible.

use super::{footer, panel, scroll_offset};
use crate::game::Game;
use crate::stats::check::success_chance;
use crate::ui::canvas::Canvas;
use crate::ui::{Area, theme};

const WIDTH: u16 = 38;

pub fn draw(canvas: &mut Canvas, game: &Game, targets: &[u32], selected: usize) {
    let w = WIDTH.min(canvas.width());
    let h = (targets.len() as u16 + 5).min(canvas.height().saturating_sub(2));
    let outer = Area::new(canvas.width().saturating_sub(w), 0, w, h);
    let reach = game.player.gun.as_ref().and_then(|g| g.reach()).map(|r| format!(" (range {r})")).unwrap_or_default();
    let inner = panel(canvas, outer, &format!("Shoot{reach}"));
    let bg = theme::POPUP;
    let me = game.player.pos;

    let visible = inner.h.saturating_sub(2) as usize;
    let offset = scroll_offset(selected, visible);
    for (y, (i, id)) in (inner.y..).zip(targets.iter().enumerate().skip(offset).take(visible)) {
        let Some(m) = game.monster(*id) else { continue };
        let rbg = if i == selected { theme::SELECTED } else { bg };
        canvas.fill(inner.x, y, inner.w, 1, rbg);
        let odds = game
            .shot_odds(*id)
            .map(|(modifier, def)| format!("{:>3.0}%", success_chance(modifier, def) * 100.0))
            .unwrap_or_default();
        let right = format!("{:>2}t {odds}", m.pos.distance(me));
        let name_w = inner.w.saturating_sub(right.len() as u16 + 1);
        let name = format!("{} {} ({})", m.def().glyph, m.name(), m.health_word());
        canvas.text_max(inner.x, y, &name, name_w, theme::TEXT, rbg);
        canvas.text(inner.x + inner.w.saturating_sub(right.len() as u16), y, &right, theme::DIM, rbg);
    }
    footer(canvas, inner, "↑↓ pick · Enter/s shoot · Esc cancel");
}
