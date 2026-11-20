//! Full-screen and pop-up views.

pub mod character;
pub mod crafting;
pub mod encounter;
pub mod ended;
pub mod help;
pub mod inventory;
pub mod open_with;
pub mod shoot;
pub mod shop;
pub mod stash;
pub mod title;
pub mod wager;

use super::canvas::Canvas;
use super::{Area, theme};
use crate::creatures::player::Player;
use crate::items::guns::LoadStats;
use crate::items::loot::{sell_price, sold_singly};
use crate::items::weight::fmt_lb;
use crate::items::{Item, ItemKind, Stat};
use std::cmp::Ordering;

/// Draw a centered pop-up frame and return its inner content area.
pub fn popup(canvas: &mut Canvas, want_w: u16, want_h: u16, title: &str) -> Area {
    let w = want_w.min(canvas.width().saturating_sub(2)).max(20);
    let h = want_h.min(canvas.height().saturating_sub(2)).max(6);
    let x = canvas.width().saturating_sub(w) / 2;
    let y = canvas.height().saturating_sub(h) / 2;
    canvas.dim();
    canvas.frame(Area::new(x, y, w, h), title, theme::BORDER, theme::POPUP);
    Area::new(x + 2, y + 1, w.saturating_sub(4), h.saturating_sub(2))
}

/// Draw an undimmed pop-up frame at a fixed spot, leaving the map readable
/// behind it; returns its inner content area.
pub fn panel(canvas: &mut Canvas, outer: Area, title: &str) -> Area {
    canvas.frame(outer, title, theme::BORDER, theme::POPUP);
    Area::new(outer.x + 2, outer.y + 1, outer.w.saturating_sub(4), outer.h.saturating_sub(2))
}

/// Footer hint on the last inner row of a pop-up.
pub fn footer(canvas: &mut Canvas, inner: Area, text: &str) {
    let y = inner.y + inner.h.saturating_sub(1);
    canvas.text_max(inner.x, y, text, inner.w, theme::DIM, theme::POPUP);
}

/// One line showing what you carry, your burden, and how much to shed to
/// reach the next lighter level, colored by burden.
pub fn draw_load(canvas: &mut Canvas, x: u16, y: u16, w: u16, p: &crate::creatures::player::Player) {
    let (carried, cap, burden) = (p.carried_weight(), p.carry_capacity(), p.burden());
    let mut line = format!("Carrying {} / {} ({})", fmt_lb(carried), fmt_lb(cap), burden.name());
    if let Some((lighter, limit)) = burden.lighter(cap) {
        line += &format!(" · shed {} to be {}", fmt_lb(carried - limit), lighter.name());
    }
    canvas.text_max(x, y, &line, w, theme::burden(burden), theme::POPUP);
}

/// A need's meter as text, e.g. `Hydration 800/1000 (Hydrated)`.
pub fn need_text(m: &crate::creatures::needs::Meter) -> String {
    format!("{} {}/{} ({})", m.def().name, m.value, m.def().max, m.label())
}

/// List columns, right-aligned: value then weight.
const WT_W: u16 = 9;
const VAL_W: u16 = 8;

/// One item-list row at `at`: a label, then the item's sale value and weight if there is one.
pub fn item_row(canvas: &mut Canvas, at: Area, label: &str, item: Option<&Item>, depth: u32, on: bool) {
    let (x, y, w) = (at.x, at.y, at.w);
    let bg = if on { theme::SELECTED } else { theme::POPUP };
    canvas.fill(x, y, w, 1, bg);
    let color = if item.is_some() { theme::TEXT } else { theme::DIM };
    canvas.text_max(x, y, label, w.saturating_sub(WT_W + VAL_W + 1), color, bg);
    let Some(item) = item else { return };
    let right = |s: &str, end: u16| end.saturating_sub(crate::ui::canvas::text_width(s) as u16);
    // A stack sold one at a time is priced per unit, as the Sell tab charges it.
    let value = match sell_price(item, depth) {
        Some(v) if sold_singly(item) => format!("{v}💎 ea"),
        Some(v) => format!("{v}💎"),
        None => "—".to_string(),
    };
    canvas.text(right(&value, x + w - WT_W), y, &value, theme::LOOT, bg);
    let wt = fmt_lb(item.weight());
    canvas.text(right(&wt, x + w), y, &wt, theme::DIM, bg);
}

/// An item's description, its stats beside `against` (the stats of the gear
/// it would replace), then for a gun a table of the loads it fires. Returns
/// the next free row.
pub fn item_details(canvas: &mut Canvas, area: Area, item: &Item, player: &Player, against: Option<&[Stat]>) -> u16 {
    let (x, w, bottom) = (area.x, area.w, area.y + area.h);
    let mut y = area.y;
    for line in item.describe() {
        y = paragraph(canvas, x, y, w, bottom, &line, theme::TEXT);
    }
    let loads = match &item.kind {
        ItemKind::Gun(g) => g.loads(),
        _ => Vec::new(),
    };
    let brawn = player.brawn_damage();
    let stats = item.stats(brawn);
    let equipped = against.unwrap_or_default();
    if !stats.is_empty() {
        let rest = Area::new(x, y + 1, w, bottom.saturating_sub(y + 1));
        // Short panes get the stats as one wrapped line so the load table still fits.
        y = if rest.h as usize >= stats.len() + load_table_height(&loads, w) {
            stat_block(canvas, rest, &stats, equipped)
        } else {
            let joined: Vec<String> = stats
                .iter()
                .map(|s| if s.label.is_empty() { s.full_text() } else { format!("{} {}", s.label, s.full_text()) })
                .collect();
            paragraph(canvas, x, rest.y, w, bottom, &joined.join(" · "), theme::TEXT)
        };
    }
    if !loads.is_empty() {
        y = load_table(canvas, Area::new(x, y + 1, w, bottom.saturating_sub(y + 1)), &loads);
    }
    y
}

/// Rows the load table needs at width `w`: a blank line, header, loads, legend.
fn load_table_height(loads: &[LoadStats], w: u16) -> usize {
    if loads.is_empty() { 0 } else { loads.len() + 2 + super::canvas::wrap(LOAD_LEGEND, w as usize).len() }
}

/// One `Label  value` per line, labels dim and padded to a shared column.
/// Beside each value, the equipped item's value for the same stat (without
/// its note): green where it's the better number, red where it's worse.
/// The numeric rows set where that column starts; a long text value (a
/// gun's Type) shows the equipped value only if it fits after it.
fn stat_block(canvas: &mut Canvas, area: Area, stats: &[Stat], equipped: &[Stat]) -> u16 {
    let width = |s: &str| super::canvas::text_width(s) as u16;
    let label_w = stats.iter().map(|s| width(s.label)).max().unwrap_or(0) + 2;
    let values: Vec<String> = stats.iter().map(Stat::full_text).collect();
    let numeric = stats.iter().zip(&values).filter(|(s, _)| s.score.is_some());
    let column = label_w + numeric.map(|(_, v)| width(v)).max().unwrap_or(0) + 3;
    let mut y = area.y;
    for (stat, value) in stats.iter().zip(&values) {
        if y >= area.y + area.h {
            break;
        }
        let at = |dx: u16| (area.x + dx, area.w.saturating_sub(dx));
        canvas.text_max(area.x, y, stat.label, area.w, theme::DIM, theme::POPUP);
        let (x, w) = at(label_w);
        canvas.text_max(x, y, value, w, theme::TEXT, theme::POPUP);
        let theirs = equipped.iter().find(|e| !e.label.is_empty() && e.label == stat.label);
        if let Some(theirs) = theirs.filter(|_| label_w + width(value) < column) {
            let color = match theirs.versus(stat) {
                Some(Ordering::Greater) => theme::GOOD,
                Some(Ordering::Less) => theme::BAD,
                _ => theme::DIM,
            };
            let (x, w) = at(column);
            canvas.text_max(x, y, &theirs.text, w, color, theme::POPUP);
        }
        y += 1;
    }
    y
}

/// The load table's right-aligned columns after the name: header and width.
const LOAD_COLUMNS: [(&str, u16); 5] = [("Gr", 4), ("FPS", 5), ("Rng", 6), ("DMG", 5), ("Pen", 4)];

/// One row per load, the chambered one marked: weight in grains, muzzle
/// velocity, effective-max range in tiles, expected damage and penetration.
fn load_table(canvas: &mut Canvas, area: Area, loads: &[LoadStats]) -> u16 {
    let header = ["Load", "Gr", "FPS", "Rng", "DMG", "Pen"].map(String::from);
    let mut y = area.y;
    let bottom = area.y + area.h;
    for (cells, fg) in std::iter::once((header, theme::DIM)).chain(loads.iter().map(|l| {
        let mark = if l.loaded { "▸ " } else { "  " };
        let cells = [
            format!("{mark}{}", l.name),
            format!("{:.0}", l.grains),
            format!("{:.0}", l.fps),
            format!("{}-{}", l.effective_range, l.max_range),
            format!("{:.1}", l.damage),
            format!("{:+}", l.pen),
        ];
        (cells, if l.loaded { theme::GOOD } else { theme::TEXT })
    })) {
        if y >= bottom {
            break;
        }
        table_row(canvas, Area::new(area.x, y, area.w, 1), &cells, fg);
        y += 1;
    }
    paragraph(canvas, area.x, y, area.w, bottom, LOAD_LEGEND, theme::DIM)
}

const LOAD_LEGEND: &str = "Rng in tiles: effective-max · ▸ loaded";

/// The name fills what the fixed columns leave; the rest right-align.
fn table_row(canvas: &mut Canvas, at: Area, cells: &[String; 6], fg: crossterm::style::Color) {
    let fixed: u16 = LOAD_COLUMNS.iter().map(|(_, w)| w).sum();
    canvas.text_max(at.x, at.y, &cells[0], at.w.saturating_sub(fixed), fg, theme::POPUP);
    let mut end = at.x + at.w.saturating_sub(fixed);
    for (cell, (_, w)) in cells[1..].iter().zip(LOAD_COLUMNS) {
        end += w;
        let x = end.saturating_sub(super::canvas::text_width(cell) as u16);
        canvas.text(x, at.y, cell, fg, theme::POPUP);
    }
}

/// First visible row for a list so that `selected` stays on screen.
pub fn scroll_offset(selected: usize, visible: usize) -> usize {
    if visible == 0 { 0 } else { selected.saturating_sub(visible - 1) }
}

/// Write wrapped paragraphs into an area; returns the next free row.
pub fn paragraph(
    canvas: &mut Canvas,
    x: u16,
    mut y: u16,
    w: u16,
    bottom: u16,
    text: &str,
    fg: crossterm::style::Color,
) -> u16 {
    for line in super::canvas::wrap(text, w as usize) {
        if y >= bottom {
            break;
        }
        canvas.text_max(x, y, &line, w, fg, theme::POPUP);
        y += 1;
    }
    y
}
