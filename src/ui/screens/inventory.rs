//! Inventory pop-up: equipped gear, the pack, and item details.

use super::{footer, item_details, item_row, need_text, popup, scroll_offset};
use crate::creatures::Player;
use crate::creatures::player::Slot;
use crate::game::Game;
use crate::items::Item;
use crate::items::loot::sale_value;
use crate::items::weight::fmt_lb;
use crate::ui::canvas::Canvas;
use crate::ui::{Area, theme};

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

    pub fn label(self) -> String {
        let name = match self.key {
            SortKey::Pickup => return "pickup order".into(),
            SortKey::Name => "name",
            SortKey::Weight => "weight",
            SortKey::Value => "value",
        };
        format!("{name} {}", if self.descending { "↓" } else { "↑" })
    }

    /// Step to the next order. Returns the row the item on row `selected`
    /// moved to, so it stays highlighted.
    pub fn advance(&mut self, items: &[Item], depth: u32, selected: usize) -> usize {
        let item = self.order(items, depth).get(selected).copied();
        *self = self.next();
        let order = self.order(items, depth);
        item.and_then(|i| order.iter().position(|&o| o == i)).unwrap_or(0)
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

/// Rows above the pack list: one per equipment slot.
pub const SLOT_ROWS: usize = Slot::ALL.len();

/// What a row of the inventory list holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Slot(Slot),
    /// An index into the pack (which stays in pickup order).
    Pack(usize),
}

/// The row at `selected`: equipment slots first, then the pack in `sort` order.
pub fn row_at(game: &Game, sort: Sort, selected: usize) -> Option<Row> {
    match selected.checked_sub(SLOT_ROWS) {
        None => Some(Row::Slot(Slot::ALL[selected])),
        Some(i) => sort.order(game.player.inventory.items(), game.depth).get(i).map(|&idx| Row::Pack(idx)),
    }
}

pub fn row_count(game: &Game) -> usize {
    SLOT_ROWS + game.player.inventory.len()
}

/// Where the cursor starts: the first pack item, or the top if the pack is empty.
pub fn first_row(game: &Game) -> usize {
    if game.player.inventory.is_empty() { 0 } else { SLOT_ROWS }
}

/// The equipped list's label for a slot, e.g. ` 🔫 Service Pistol [15/15]`.
fn slot_label(p: &Player, slot: Slot) -> String {
    match (slot, p.equipped(slot)) {
        (_, Some(item)) => format!(" {} {}", item.glyph(), item.display_name()),
        (Slot::Melee, None) => format!(" {} {}", p.melee_def().glyph, p.melee_def().name),
        (_, None) => format!(" — no {}", slot.name()),
    }
}

pub fn draw(canvas: &mut Canvas, game: &Game, sort: Sort, selected: usize) {
    let inner = popup(canvas, 104, 34, "Inventory");
    let p = &game.player;
    let list_w = 50.min(inner.w / 2);
    let (x, mut y) = (inner.x, inner.y);
    let bottom = inner.y + inner.h.saturating_sub(2);

    canvas.text(x, y, "Equipped", theme::TITLE, theme::POPUP);
    y += 1;
    for (i, slot) in Slot::ALL.into_iter().enumerate() {
        let item = p.equipped(slot);
        item_row(canvas, Area::new(x, y, list_w, 1), &slot_label(p, slot), item.as_ref(), game.depth, i == selected);
        y += 1;
    }
    let burden = p.burden();
    let load = format!("Load {} / {} ({})", fmt_lb(p.carried_weight()), fmt_lb(p.carry_capacity()), burden.name());
    canvas.text_max(x, y, &load, list_w, theme::burden(burden), theme::POPUP);
    y += 1;
    for m in p.needs() {
        let color = if m.stage().is_warning() { theme::BAD } else { theme::TEXT };
        canvas.text_max(x, y, &need_text(m), list_w, color, theme::POPUP);
        y += 1;
    }
    y += 1;

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
    let pack_selected = selected.checked_sub(SLOT_ROWS);
    let offset = scroll_offset(pack_selected.unwrap_or(0), visible);
    for (i, &idx) in order.iter().enumerate().skip(offset).take(visible) {
        let item = &items[idx];
        let letter = (b'a' + i as u8) as char;
        let label = format!("{letter}) {} {}", item.glyph(), item.display_name());
        item_row(canvas, Area::new(x, y, list_w, 1), &label, Some(item), game.depth, pack_selected == Some(i));
        y += 1;
    }

    let dx = inner.x + list_w + 3;
    let dw = inner.w.saturating_sub(list_w + 3);
    draw_detail(canvas, Area::new(dx, inner.y, dw, bottom.saturating_sub(inner.y)), game, row_at(game, sort, selected));

    footer(canvas, inner, "a-z/↑↓ select · Enter use/equip/unequip · d drop · w stow melee · s sort · Esc close");
}

/// The right-hand panel: everything about the selected item.
fn draw_detail(canvas: &mut Canvas, area: Area, game: &Game, row: Option<Row>) {
    let (item, equipped) = match row {
        Some(Row::Slot(slot)) => (game.player.equipped(slot), true),
        Some(Row::Pack(idx)) => (game.player.inventory.get(idx).cloned(), false),
        None => (None, false),
    };
    let (dx, dw, bottom) = (area.x, area.w, area.y + area.h);
    let mut dy = area.y;
    let Some(item) = item else {
        if let Some(Row::Slot(slot)) = row {
            let text = if slot == Slot::Melee { "You fight bare-handed." } else { "Nothing equipped." };
            canvas.text_max(dx, dy, &format!("No {} equipped", slot.name()), dw, theme::TITLE, theme::POPUP);
            canvas.text_max(dx, dy + 2, text, dw, theme::DIM, theme::POPUP);
        }
        return;
    };
    canvas.text_max(dx, dy, &format!("{} {}", item.glyph(), item.display_name()), dw, theme::TITLE, theme::POPUP);
    dy += 1;
    let category = if equipped { format!("{} · equipped", item.category()) } else { item.category().to_string() };
    canvas.text_max(dx, dy, &category, dw, theme::DIM, theme::POPUP);
    dy += 1;
    let worth = match sale_value(&item, game.depth) {
        Some(v) => format!("An Autovend would pay {v} 💎 for it."),
        None => "No Autovend will buy it.".to_string(),
    };
    canvas.text_max(dx, dy, &worth, dw, theme::LOOT, theme::POPUP);
    dy += 2;
    item_details(canvas, Area::new(dx, dy, dw, bottom.saturating_sub(dy)), &item);
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
        let rows = canvas.text_rows();
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

    #[test]
    fn melee_and_armor_details_show_a_stats_block() {
        let g = game();
        let items = g.player.inventory.items();
        for (name, label) in [("Kevlar Vest", "Dodge Penalty  "), ("Crowbar", "Penetration  ")] {
            let idx = items.iter().position(|i| i.base_name() == name).unwrap();
            let mut canvas = Canvas::new(120, 40);
            draw(&mut canvas, &g, Sort::default(), SLOT_ROWS + idx);
            let rows = canvas.text_rows();
            assert!(rows.iter().any(|r| r.contains(label)), "{name} shows {label}\n{}", rows.join("\n"));
            println!("{}", rows.join("\n"));
        }
    }

    #[test]
    fn rows_put_equipment_before_the_pack() {
        let g = game();
        let sort = Sort::default();
        assert_eq!(row_at(&g, sort, 0), Some(Row::Slot(Slot::Melee)));
        assert_eq!(row_at(&g, sort, SLOT_ROWS - 1), Some(Row::Slot(Slot::Pack)));
        assert_eq!(row_at(&g, sort, SLOT_ROWS), Some(Row::Pack(0)));
        assert_eq!(row_at(&g, sort, row_count(&g)), None);
        assert_eq!(first_row(&g), SLOT_ROWS);
    }

    #[test]
    fn renders_equipped_gear_needs_and_its_details() {
        let g = game();
        let gun = g.player.gun.clone().expect("this background starts with a gun");
        let mut canvas = Canvas::new(120, 40);
        draw(&mut canvas, &g, Sort::default(), 1);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        let name = &gun.def().name;
        let listed = rows.iter().filter(|r| r.contains(&format!("🔫 {name}")) && r.contains(" lb")).count();
        assert_eq!(listed, 1, "the equipped gun is listed with its weight");
        assert!(rows.iter().any(|r| r.contains("· equipped")), "the gun's details show on the right");
        let value_col = |label: &str, value: &str| {
            let row = rows.iter().find(|r| r.contains(label) && r.contains(value)).unwrap_or_else(|| panic!("{label}"));
            // Measured from the label: emoji earlier in the row are two columns wide.
            let from = row.find(label).unwrap();
            row[from..from + row[from..].find(value).unwrap()].chars().count()
        };
        let d = gun.def();
        let col = value_col("Type", d.cartridge.name());
        assert_eq!(value_col("Reload Turns", &d.action.reload_turns().to_string()), col, "values share a column");
        assert_eq!(value_col("Capacity", &d.capacity.to_string()), col);
        for label in ["Barrel", "Skill", "Accuracy", "Weight"] {
            assert!(rows.iter().any(|r| r.contains(&format!("{label}  "))), "stat line {label}");
        }
        let header = rows.iter().find(|r| r.contains("Load") && r.contains("FPS")).expect("load table header");
        for col in ["Gr", "Rng", "DMG", "Pen"] {
            assert!(header.contains(col), "{header}");
        }
        let loaded = gun.loads().into_iter().find(|l| l.loaded).expect("starts loaded").name;
        assert!(rows.iter().any(|r| r.contains(&format!("▸ {loaded}"))), "the chambered load is marked");
        assert!(rows.iter().any(|r| r.contains("Satiety 750/1000 (Fed)")));
        assert!(rows.iter().any(|r| r.contains("Hydration 800/1000 (Hydrated)")));
    }
}
