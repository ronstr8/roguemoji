//! Autovend kiosk pop-up: buy from its stock, or sell from your pack.

use super::inventory::Sort;
use super::{draw_load, footer, item_details, paragraph, popup, scroll_offset};
use crate::game::Game;
use crate::items::Item;
use crate::items::loot::{sell_price, sell_qty};
use crate::items::weight::{fmt_lb, unit_weight};
use crate::ui::canvas::Canvas;
use crate::ui::{Area, theme};

pub fn draw(canvas: &mut Canvas, game: &Game, kiosk: usize, selling: bool, sort: Sort, selected: usize) {
    let inner = popup(canvas, 96, 24, "🏧 Autovend");
    let bg = theme::POPUP;
    let Some(k) = game.kiosks.get(kiosk) else { return };
    let list_w = 46.min(inner.w / 2);
    let bottom = inner.y + inner.h.saturating_sub(2);

    // Tabs, then crystals on hand.
    let (buy_c, sell_c) = if selling { (theme::DIM, theme::TITLE) } else { (theme::TITLE, theme::DIM) };
    let mut x = inner.x;
    x += canvas.text(x, inner.y, if selling { " Buy " } else { "[Buy]" }, buy_c, bg) + 1;
    x += canvas.text(x, inner.y, if selling { "[Sell]" } else { " Sell " }, sell_c, bg) + 3;
    x += canvas.text(x, inner.y, &format!("You carry {} 💎", game.player.crystals), theme::LOOT, bg);
    if selling {
        canvas.text(x + 3, inner.y, &format!("sort: {}", sort.label()), theme::DIM, bg);
    }

    draw_load(canvas, inner.x, inner.y + 1, inner.w, &game.player);

    // Rows: (item, price shown, whether it can be bought/sold right now).
    let rows: Vec<(&Item, Option<u32>, bool)> = if selling {
        let items = game.player.inventory.items();
        sort.order(items, game.depth)
            .into_iter()
            .map(|i| {
                let price = sell_price(&items[i], game.depth);
                (&items[i], price, price.is_some())
            })
            .collect()
    } else {
        k.stock.iter().map(|(it, price)| (it, Some(*price), game.player.crystals >= *price)).collect()
    };
    let top = inner.y + 3;
    if rows.is_empty() {
        let msg = if selling { "Your pack is empty." } else { "SOLD OUT. HAVE A NICE DAY." };
        canvas.text(inner.x, top, msg, theme::DIM, bg);
    }
    let visible = bottom.saturating_sub(top) as usize;
    let offset = scroll_offset(selected, visible);
    for (y, (i, (item, price, ok))) in (top..).zip(rows.iter().enumerate().skip(offset).take(visible)) {
        let rbg = if i == selected { theme::SELECTED } else { bg };
        canvas.fill(inner.x, y, list_w, 1, rbg);
        let color = if *ok { theme::TEXT } else { theme::DIM };
        let price = price.map_or("  —".to_string(), |p| format!("{p:>3}"));
        let label = format!("{price}💎  {} {}", item.glyph(), item.display_name());
        canvas.text_max(inner.x, y, &label, list_w, color, rbg);
    }

    if let Some((item, price, _)) = rows.get(selected) {
        let dx = inner.x + list_w + 3;
        let dw = inner.w.saturating_sub(list_w + 3);
        let mut y = top;
        canvas.text_max(dx, y, &item.display_name(), dw, theme::TITLE, bg);
        y += 2;
        if selling {
            let note = match price {
                None => "The Autovend won't buy this.".to_string(),
                Some(p) if sell_qty(item) == item.qty => format!("Sells for {p} 💎."),
                Some(p) => format!("Sells for {p} 💎 each."),
            };
            y = paragraph(canvas, dx, y, dw, bottom, &note, theme::LOOT);
        }
        // What selling it sheds, or buying it adds.
        let sold = if selling { sell_qty(item) } else { item.qty };
        let w = fmt_lb(unit_weight(&item.kind) * sold);
        let wnote = if selling { format!("Selling sheds {w}.") } else { format!("Weighs {w}.") };
        y = paragraph(canvas, dx, y, dw, bottom, &wnote, theme::DIM);
        y += 1;
        item_details(canvas, Area::new(dx, y, dw, bottom.saturating_sub(y)), item);
    }
    let help = if selling {
        "↑↓ select · Enter sell · s sort · Tab buy · Esc leave"
    } else {
        "↑↓ select · Enter buy · Tab sell · Esc leave   (crystals spent here can't heal you)"
    };
    footer(canvas, inner, help);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::game::Kiosk;

    #[test]
    fn renders_tabs_and_sale_prices() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        g.kiosks = vec![Kiosk { stock: Vec::new() }];
        let mut canvas = Canvas::new(120, 30);
        // Value, descending: two steps past name.
        let mut sort = Sort::default();
        for _ in 0..6 {
            sort = sort.next();
        }
        draw(&mut canvas, &g, 0, true, sort, 0);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains(" Buy  [Sell]   You carry") && r.contains("sort: value ↓")));
        assert!(rows.iter().any(|r| r.contains("Carrying ") && r.contains(" lb (Light)")));
        assert!(rows.iter().any(|r| r.contains("Selling sheds ")));
        // Sale prices run high to low down the list.
        let prices: Vec<u32> = rows
            .iter()
            .filter_map(|r| r.split('│').nth(1)?.trim_start().split_once('💎')?.0.trim().parse().ok())
            .collect();
        assert!(prices.len() >= 2 && prices.windows(2).all(|w| w[0] >= w[1]), "{prices:?}");
    }

    #[test]
    fn a_shotgun_for_sale_shows_every_load_in_its_table() {
        use crate::items::{Ammo, Gun, GunModel};
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        let buck = Ammo::lookup("12ga_buck").unwrap();
        let gun = Gun::loaded_with(GunModel::named("pump_shotgun_12ga"), buck, 0);
        g.kiosks = vec![Kiosk { stock: vec![(Item::gun(gun.clone()), 30)] }];
        let mut canvas = Canvas::new(100, 26);
        draw(&mut canvas, &g, 0, false, Sort::default(), 0);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        for load in gun.loads() {
            assert!(rows.iter().any(|r| r.contains(load.ammo.name())), "{} is listed", load.ammo.name());
        }
        assert!(rows.iter().any(|r| r.contains("Rng in tiles: effective-max")), "the legend fits");
        assert!(!rows.iter().any(|r| r.contains("▸ 12ga")), "an empty gun has nothing chambered");
    }
}
