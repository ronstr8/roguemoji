//! Inventory pop-up: equipped gear, the pack, and item details.

use super::{footer, paragraph, popup, scroll_offset};
use crate::game::Game;
use crate::items::loot::sale_value;
use crate::items::weight::fmt_lb;
use crate::items::{GUN_GLYPH, Item};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SortKey {
    /// The order things went into the pack.
    #[default]
    Pickup,
    Name,
    Weight,
    Value,
}

/// How the pack list is ordered. `s` steps through name, weight and value,
/// each ascending then descending, then back to pickup order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sort {
    key: SortKey,
    descending: bool,
}

impl Sort {
    pub fn next(self) -> Sort {
        let asc = |key| Sort { key, descending: false };
        match (self.key, self.descending) {
            (SortKey::Pickup, _) => asc(SortKey::Name),
            (key, false) => Sort { key, descending: true },
            (SortKey::Name, true) => asc(SortKey::Weight),
            (SortKey::Weight, true) => asc(SortKey::Value),
            (SortKey::Value, true) => Sort::default(),
        }
    }

    fn label(self) -> String {
        let name = match self.key {
            SortKey::Pickup => return "pickup order".into(),
            SortKey::Name => "name",
            SortKey::Weight => "weight",
            SortKey::Value => "value",
        };
        format!("{name} {}", if self.descending { "↓" } else { "↑" })
    }

    /// Pack indices in display order. Ties keep pickup order.
    pub fn order(self, items: &[Item], depth: u32) -> Vec<usize> {
        let mut order: Vec<usize> = (0..items.len()).collect();
        let name = |i: usize| items[i].base_name().to_lowercase();
        match self.key {
            SortKey::Pickup => return order,
            SortKey::Name => order.sort_by_cached_key(|&i| name(i)),
            SortKey::Weight => order.sort_by_cached_key(|&i| (items[i].weight(), name(i))),
            // Things an Autovend won't buy count as worth least.
            SortKey::Value => order.sort_by_cached_key(|&i| (sale_value(&items[i], depth), name(i))),
        }
        if self.descending {
            order.reverse();
        }
        order
    }
}

pub fn draw(canvas: &mut Canvas, game: &Game, sort: Sort, selected: usize) {
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
    let armor = p.armor.map_or(("  ", "—"), |a| (a.def().glyph.as_str(), a.def().name.as_str()));
    canvas.text_max(x, y, &format!(" {} {}", armor.0, armor.1), list_w, theme::TEXT, theme::POPUP);
    y += 1;
    let pack = p.pack.map_or(("  ", "— (no pack)".to_string()), |k| {
        (k.def().glyph.as_str(), format!("{} (+{})", k.def().name, fmt_lb(k.def().capacity)))
    });
    canvas.text_max(x, y, &format!(" {} {}", pack.0, pack.1), list_w, theme::TEXT, theme::POPUP);
    y += 1;
    let burden = p.burden();
    let load = format!("Load {} / {} · {}", fmt_lb(p.carried_weight()), fmt_lb(p.carry_capacity()), burden.name());
    canvas.text_max(x, y, &load, list_w, theme::burden(burden), theme::POPUP);
    y += 2;

    let items = p.inventory.items();
    let order = sort.order(items, game.depth);
    let pack_title = format!("Pack ({}/{}) · {}", items.len(), p.inventory.slots(), fmt_lb(p.inventory.weight()));
    let used = canvas.text(x, y, &pack_title, theme::TITLE, theme::POPUP) + 2;
    let label = format!("sort: {}", sort.label());
    canvas.text_max(x + used, y, &label, list_w.saturating_sub(used), theme::DIM, theme::POPUP);
    y += 1;
    if p.inventory.is_empty() {
        canvas.text(x, y, " (empty)", theme::DIM, theme::POPUP);
    }
    let visible = bottom.saturating_sub(y) as usize;
    let offset = scroll_offset(selected, visible);
    // Right-hand columns: value then weight, each right-aligned.
    const WT_W: u16 = 9;
    const VAL_W: u16 = 6;
    for (i, &idx) in order.iter().enumerate().skip(offset).take(visible) {
        let item = &items[idx];
        let letter = (b'a' + i as u8) as char;
        let bg = if i == selected { theme::SELECTED } else { theme::POPUP };
        canvas.fill(x, y, list_w, 1, bg);
        let name_w = list_w.saturating_sub(WT_W + VAL_W + 1);
        let text = format!("{letter}) {} {}", item.glyph(), item.display_name());
        canvas.text_max(x, y, &text, name_w, theme::TEXT, bg);
        let value = sale_value(item, game.depth).map_or("—".to_string(), |v| format!("{v}💎"));
        let right = |s: &str, end: u16| end.saturating_sub(crate::ui::canvas::text_width(s) as u16);
        let val_end = x + list_w - WT_W;
        canvas.text(right(&value, val_end), y, &value, theme::LOOT, bg);
        let wt = fmt_lb(item.weight());
        canvas.text(right(&wt, x + list_w), y, &wt, theme::DIM, bg);
        y += 1;
    }

    // Detail panel.
    let dx = inner.x + list_w + 3;
    let dw = inner.w.saturating_sub(list_w + 3);
    if let Some(item) = order.get(selected).map(|&i| &items[i]) {
        let mut dy = inner.y;
        canvas.text_max(dx, dy, &format!("{} {}", item.glyph(), item.display_name()), dw, theme::TITLE, theme::POPUP);
        dy += 1;
        canvas.text_max(dx, dy, item.category(), dw, theme::DIM, theme::POPUP);
        dy += 1;
        let worth = match sale_value(item, game.depth) {
            Some(v) => format!("An Autovend would pay {v} 💎 for it."),
            None => "No Autovend will buy it.".to_string(),
        };
        canvas.text_max(dx, dy, &worth, dw, theme::LOOT, theme::POPUP);
        dy += 2;
        for line in item.describe() {
            dy = paragraph(canvas, dx, dy, dw, bottom, &line, theme::TEXT);
        }
    }

    footer(canvas, inner, "a-z/↑↓ select · Enter use/equip · d drop · w stow melee · s sort · Esc close");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::items::{Armor, Consumable, ItemKind, MeleeWeapon};

    fn game() -> Game {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        for item in [
            Item::one(ItemKind::Armor(Armor::named("kevlar_vest"))),
            Item::one(ItemKind::Melee(MeleeWeapon::named("crowbar"))),
            Item::food(Consumable::named("hardtack"), 4),
        ] {
            g.player.inventory.add(item).unwrap();
        }
        g
    }

    #[test]
    fn sort_cycles_through_every_mode() {
        let mut s = Sort::default();
        let mut labels = Vec::new();
        for _ in 0..7 {
            labels.push(s.label());
            s = s.next();
        }
        assert_eq!(labels, ["pickup order", "name ↑", "name ↓", "weight ↑", "weight ↓", "value ↑", "value ↓"]);
        assert_eq!(s, Sort::default());
    }

    #[test]
    fn orders_by_each_key() {
        let g = game();
        let items = g.player.inventory.items();
        let names = |sort: Sort| -> Vec<String> {
            sort.order(items, g.depth).into_iter().map(|i| items[i].base_name()).collect()
        };
        let by_name = names(Sort { key: SortKey::Name, descending: false });
        let mut sorted = by_name.clone();
        sorted.sort_by_key(|n| n.to_lowercase());
        assert_eq!(by_name, sorted);
        let desc = names(Sort { key: SortKey::Name, descending: true });
        assert_eq!(desc, by_name.iter().rev().cloned().collect::<Vec<_>>());

        let values: Vec<Option<u32>> = Sort { key: SortKey::Value, descending: true }
            .order(items, g.depth)
            .into_iter()
            .map(|i| sale_value(&items[i], g.depth))
            .collect();
        assert!(values.windows(2).all(|w| w[0] >= w[1]), "{values:?}");
        assert_eq!(values.last(), Some(&None), "unsellable items sort last when descending");

        let weights: Vec<u32> = Sort { key: SortKey::Weight, descending: false }
            .order(items, g.depth)
            .into_iter()
            .map(|i| items[i].weight())
            .collect();
        assert!(weights.windows(2).all(|w| w[0] <= w[1]), "{weights:?}");
    }

    #[test]
    fn renders_value_and_weight_columns() {
        let g = game();
        let mut canvas = Canvas::new(120, 36);
        draw(&mut canvas, &g, Sort { key: SortKey::Value, descending: true }, 0);
        let rows: Vec<String> = (0..canvas.height())
            .map(|y| canvas.row(y).iter().filter(|c| !c.cont).map(|c| c.ch).collect::<String>())
            .collect();
        let screen = rows.join("\n");
        println!("{screen}");
        assert!(screen.contains("sort: value ↓"));
        let vest = rows.iter().find(|r| r.contains("Kevlar Vest") && r.contains("a)")).expect("vest is first");
        let worth = sale_value(&Item::one(ItemKind::Armor(Armor::named("kevlar_vest"))), g.depth).unwrap();
        assert!(vest.contains(&format!("{worth}💎")) && vest.contains("8.0 lb"), "{vest}");
        let title = rows.iter().find(|r| r.contains("Pack (")).unwrap();
        assert!(title.contains(" lb  sort: value ↓"), "{title}");
        assert!(rows.iter().any(|r| r.contains("Crowbar") && r.contains("—")));
    }
}
