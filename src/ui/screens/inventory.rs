//! Inventory pop-up: equipped gear, the pack, and item details.

use super::{footer, paragraph, popup, scroll_offset};
use crate::game::Game;
use crate::items::GUN_GLYPH;
use crate::items::inventory::MAX_SLOTS;
use crate::ui::canvas::Canvas;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game, selected: usize) {
    let inner = popup(canvas, 104, 32, "Inventory");
    let p = &game.player;
    let list_w = 46.min(inner.w / 2);
    let (x, mut y) = (inner.x, inner.y);
    let bottom = inner.y + inner.h.saturating_sub(2);

    canvas.text(x, y, "Equipped", theme::TITLE, theme::POPUP);
    y += 1;
    let melee = p.melee_def();
    canvas.text_max(x, y, &format!(" {} {}", melee.glyph, melee.name), list_w, theme::TEXT, theme::POPUP);
    y += 1;
    let gun = p.gun.as_ref().map_or("—".to_string(), |g| format!("{} [{}]", g.def().name, g.status()));
    canvas.text_max(x, y, &format!(" {GUN_GLYPH} {gun}"), list_w, theme::TEXT, theme::POPUP);
    y += 1;
    let armor = p.armor.map_or(("  ", "—"), |a| (a.def().glyph, a.def().name));
    canvas.text_max(x, y, &format!(" {} {}", armor.0, armor.1), list_w, theme::TEXT, theme::POPUP);
    y += 2;

    let items = p.inventory.items();
    canvas.text(x, y, &format!("Pack ({}/{})", items.len(), MAX_SLOTS), theme::TITLE, theme::POPUP);
    y += 1;
    if p.inventory.is_empty() {
        canvas.text(x, y, " (empty)", theme::DIM, theme::POPUP);
    }
    let visible = bottom.saturating_sub(y) as usize;
    let offset = scroll_offset(selected, visible);
    for (i, item) in items.iter().enumerate().skip(offset).take(visible) {
        let letter = (b'a' + i as u8) as char;
        let bg = if i == selected { theme::SELECTED } else { theme::POPUP };
        canvas.fill(x, y, list_w, 1, bg);
        let text = format!("{letter}) {} {}", item.glyph(), item.display_name());
        canvas.text_max(x, y, &text, list_w, theme::TEXT, bg);
        y += 1;
    }

    // Detail panel.
    let dx = inner.x + list_w + 3;
    let dw = inner.w.saturating_sub(list_w + 3);
    if let Some(item) = items.get(selected) {
        let mut dy = inner.y;
        canvas.text_max(dx, dy, &format!("{} {}", item.glyph(), item.display_name()), dw, theme::TITLE, theme::POPUP);
        dy += 1;
        canvas.text_max(dx, dy, item.category(), dw, theme::DIM, theme::POPUP);
        dy += 2;
        for line in item.describe() {
            dy = paragraph(canvas, dx, dy, dw, bottom, &line, theme::TEXT);
        }
    }

    footer(canvas, inner, "a-z/↑↓ select · Enter use/equip · d drop · w stow melee · Esc close");
}
