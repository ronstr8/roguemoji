//! Visitor lockbox pop-up: shuffle items between the pack and the shared stash.

use super::inventory::Sort;
use super::{draw_load, footer, item_row, popup, scroll_offset};
use crate::game::{Game, StashSide};
use crate::items::Item;
use crate::ui::canvas::Canvas;
use crate::ui::{Area, theme};

/// Both columns follow `sort`; `selected` is a row on screen in the `side` column.
pub fn draw(canvas: &mut Canvas, game: &Game, side: StashSide, sort: Sort, selected: usize) {
    let inner = popup(canvas, 100, 30, "🛅 Visitor Lockbox");
    let p = &game.player;
    let col_w = inner.w.saturating_sub(3) / 2;
    let bottom = inner.y + inner.h.saturating_sub(2);

    draw_load(canvas, inner.x, inner.y, inner.w, p);

    let pack = p.inventory.items();
    let stash = game.stash.items();
    let top = inner.y + 2;
    let pack_title = format!("Pack ({}/{})", pack.len(), p.inventory.slots());
    let stash_title = format!("Lockbox ({}/{})", stash.len(), game.stash.slots());
    let left = Area::new(inner.x, top, col_w, bottom.saturating_sub(top));
    let right = Area::new(inner.x + col_w + 3, top, col_w, bottom.saturating_sub(top));
    let col = |items, on: bool| Column::new(items, sort, game.depth, on.then_some(selected));
    column(canvas, left, &pack_title, col(pack, side == StashSide::Pack), game.depth);
    column(canvas, right, &stash_title, col(stash, side == StashSide::Lockbox), game.depth);

    let hint = format!("↑↓ select · Tab/←→ switch side · Enter move stack · s sort: {} · Esc close", sort.label());
    footer(canvas, inner, &hint);
}

/// One side's items, the order to show them in, and the highlighted row.
struct Column<'a> {
    items: &'a [Item],
    order: Vec<usize>,
    selected: Option<usize>,
}

impl<'a> Column<'a> {
    fn new(items: &'a [Item], sort: Sort, depth: u32, selected: Option<usize>) -> Self {
        Column { items, order: sort.order(items, depth), selected }
    }
}

fn column(canvas: &mut Canvas, area: Area, title: &str, col: Column, depth: u32) {
    let title_color = if col.selected.is_some() { theme::TITLE } else { theme::DIM };
    canvas.text_max(area.x, area.y, title, area.w, title_color, theme::POPUP);
    if col.items.is_empty() {
        canvas.text(area.x, area.y + 1, " (empty)", theme::DIM, theme::POPUP);
        return;
    }
    let visible = area.h.saturating_sub(1) as usize;
    let offset = scroll_offset(col.selected.unwrap_or(0), visible);
    for (y, (row, &idx)) in (area.y + 1..).zip(col.order.iter().enumerate().skip(offset).take(visible)) {
        let item = &col.items[idx];
        let label = format!("{} {}", item.glyph(), item.display_name());
        item_row(canvas, Area::new(area.x, y, area.w, 1), &label, Some(item), depth, col.selected == Some(row));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::items::loot::sale_value;
    use crate::items::{Armor, Consumable, ItemKind};

    #[test]
    fn both_columns_follow_the_sort() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        g.stash.add(Item::food(Consumable::named("hardtack"), 2)).unwrap();
        g.stash.add(Item::one(ItemKind::Armor(Armor::named("kevlar_vest")))).unwrap();
        let sort = Sort::default().next().next().next().next().next().next(); // value ↓
        let mut canvas = Canvas::new(110, 32);
        draw(&mut canvas, &g, StashSide::Lockbox, sort, 0);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains("s sort: value ↓")));
        let vest_value = sale_value(&Item::one(ItemKind::Armor(Armor::named("kevlar_vest"))), g.depth).unwrap();
        // The row under the titles holds each column's first item.
        let titles = rows.iter().position(|r| r.contains("Lockbox (")).expect("lockbox title");
        let first = &rows[titles + 1];
        assert!(first.contains("Kevlar Vest") && first.contains(&format!("{vest_value}💎")), "{first}");
    }
}
