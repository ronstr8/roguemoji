//! Visitor lockbox pop-up: shuffle items between the pack and the shared stash.

use super::{footer, popup, scroll_offset};
use crate::game::{Game, StashSide};
use crate::items::Item;
use crate::items::weight::fmt_lb;
use crate::ui::canvas::Canvas;
use crate::ui::{Area, theme};

pub fn draw(canvas: &mut Canvas, game: &Game, side: StashSide, selected: usize) {
    let inner = popup(canvas, 100, 30, "🛅 Visitor Lockbox");
    let bg = theme::POPUP;
    let p = &game.player;
    let col_w = inner.w.saturating_sub(3) / 2;
    let bottom = inner.y + inner.h.saturating_sub(2);

    let burden = p.burden();
    let load = format!("Carrying {} / {} · {}", fmt_lb(p.carried_weight()), fmt_lb(p.carry_capacity()), burden.name());
    canvas.text_max(inner.x, inner.y, &load, inner.w, theme::burden(burden), bg);

    let pack = p.inventory.items();
    let stash = game.stash.items();
    let top = inner.y + 2;
    let pack_title = format!("Pack ({}/{})", pack.len(), p.inventory.slots());
    let stash_title = format!("Lockbox ({}/{})", stash.len(), game.stash.slots());
    let left = Area::new(inner.x, top, col_w, bottom.saturating_sub(top));
    let right = Area::new(inner.x + col_w + 3, top, col_w, bottom.saturating_sub(top));
    column(canvas, left, &pack_title, pack, (side == StashSide::Pack).then_some(selected));
    column(canvas, right, &stash_title, stash, (side == StashSide::Lockbox).then_some(selected));

    footer(canvas, inner, "↑↓ select · Tab/←→ switch side · Enter move stack · Esc close");
}

fn column(canvas: &mut Canvas, area: Area, title: &str, items: &[Item], selected: Option<usize>) {
    let bg = theme::POPUP;
    let title_color = if selected.is_some() { theme::TITLE } else { theme::DIM };
    canvas.text_max(area.x, area.y, title, area.w, title_color, bg);
    if items.is_empty() {
        canvas.text(area.x, area.y + 1, " (empty)", theme::DIM, bg);
        return;
    }
    let visible = area.h.saturating_sub(1) as usize;
    let offset = scroll_offset(selected.unwrap_or(0), visible);
    for (y, (i, item)) in (area.y + 1..).zip(items.iter().enumerate().skip(offset).take(visible)) {
        let rbg = if selected == Some(i) { theme::SELECTED } else { bg };
        canvas.fill(area.x, y, area.w, 1, rbg);
        let wt = fmt_lb(item.weight());
        let name_w = area.w.saturating_sub(wt.len() as u16 + 1);
        canvas.text_max(area.x, y, &format!("{} {}", item.glyph(), item.display_name()), name_w, theme::TEXT, rbg);
        canvas.text(area.x + area.w.saturating_sub(wt.len() as u16), y, &wt, theme::DIM, rbg);
    }
}
