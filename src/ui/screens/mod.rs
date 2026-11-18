//! Full-screen and pop-up views.

pub mod character;
pub mod crafting;
pub mod ended;
pub mod help;
pub mod inventory;
pub mod shoot;
pub mod shop;
pub mod stash;
pub mod title;

use super::canvas::Canvas;
use super::{Area, theme};
use crate::items::Item;
use crate::items::loot::sale_value;
use crate::items::weight::fmt_lb;

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
const VAL_W: u16 = 6;

/// One item-list row at `at`: a label, then the item's sale value and weight if there is one.
pub fn item_row(canvas: &mut Canvas, at: Area, label: &str, item: Option<&Item>, depth: u32, on: bool) {
    let (x, y, w) = (at.x, at.y, at.w);
    let bg = if on { theme::SELECTED } else { theme::POPUP };
    canvas.fill(x, y, w, 1, bg);
    let color = if item.is_some() { theme::TEXT } else { theme::DIM };
    canvas.text_max(x, y, label, w.saturating_sub(WT_W + VAL_W + 1), color, bg);
    let Some(item) = item else { return };
    let right = |s: &str, end: u16| end.saturating_sub(crate::ui::canvas::text_width(s) as u16);
    let value = sale_value(item, depth).map_or("—".to_string(), |v| format!("{v}💎"));
    canvas.text(right(&value, x + w - WT_W), y, &value, theme::LOOT, bg);
    let wt = fmt_lb(item.weight());
    canvas.text(right(&wt, x + w), y, &wt, theme::DIM, bg);
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
