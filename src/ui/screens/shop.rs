//! Autovend kiosk pop-up: buy from its stock, or sell from your pack.

use super::{footer, paragraph, popup, scroll_offset};
use crate::game::Game;
use crate::items::Item;
use crate::items::loot::{sell_price, sell_qty};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game, kiosk: usize, selling: bool, selected: usize) {
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
    canvas.text(x, inner.y, &format!("You carry {} 💎", game.player.crystals), theme::LOOT, bg);

    // Rows: (item, price shown, whether it can be bought/sold right now).
    let rows: Vec<(&Item, Option<u32>, bool)> = if selling {
        game.player
            .inventory
            .items()
            .iter()
            .map(|it| {
                let price = sell_price(it, game.depth);
                (it, price, price.is_some())
            })
            .collect()
    } else {
        k.stock.iter().map(|(it, price)| (it, Some(*price), game.player.crystals >= *price)).collect()
    };
    let top = inner.y + 2;
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
            y += 1;
        }
        for line in item.describe() {
            y = paragraph(canvas, dx, y, dw, bottom, &line, theme::TEXT);
        }
    }
    let help = if selling {
        "↑↓ select · Enter sell · Tab buy · Esc leave"
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
        draw(&mut canvas, &g, 0, true, 0);
        let rows: Vec<String> = (0..canvas.height())
            .map(|y| canvas.row(y).iter().filter(|c| !c.cont).map(|c| c.ch).collect::<String>())
            .collect();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains(" Buy  [Sell]   You carry")));
    }
}
