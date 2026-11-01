//! Autovend kiosk pop-up.

use super::{footer, paragraph, popup};
use crate::game::Game;
use crate::ui::canvas::Canvas;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game, kiosk: usize, selected: usize) {
    let inner = popup(canvas, 96, 24, "🏧 Autovend");
    let bg = theme::POPUP;
    let Some(k) = game.kiosks.get(kiosk) else { return };
    let list_w = 46.min(inner.w / 2);
    let bottom = inner.y + inner.h.saturating_sub(2);

    canvas.text(inner.x, inner.y, &format!("You carry {} 💎", game.player.crystals), theme::LOOT, bg);
    if k.stock.is_empty() {
        canvas.text(inner.x, inner.y + 2, "SOLD OUT. HAVE A NICE DAY.", theme::DIM, bg);
    }
    for (i, (item, price)) in k.stock.iter().enumerate() {
        let y = inner.y + 2 + i as u16;
        if y >= bottom {
            break;
        }
        let rbg = if i == selected { theme::SELECTED } else { bg };
        let affordable = game.player.crystals >= *price;
        canvas.fill(inner.x, y, list_w, 1, rbg);
        let color = if affordable { theme::TEXT } else { theme::DIM };
        canvas.text_max(
            inner.x,
            y,
            &format!("{:>3}💎  {} {}", price, item.glyph(), item.display_name()),
            list_w,
            color,
            rbg,
        );
    }
    if let Some((item, _)) = k.stock.get(selected) {
        let dx = inner.x + list_w + 3;
        let dw = inner.w.saturating_sub(list_w + 3);
        let mut y = inner.y + 2;
        canvas.text_max(dx, y, &item.display_name(), dw, theme::TITLE, bg);
        y += 2;
        for line in item.describe() {
            y = paragraph(canvas, dx, y, dw, bottom, &line, theme::TEXT);
        }
    }
    footer(canvas, inner, "↑↓ select · Enter buy · Esc leave   (crystals spent here can't heal you)");
}
