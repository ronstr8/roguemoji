//! Full-screen and pop-up views.

pub mod character;
pub mod crafting;
pub mod ended;
pub mod help;
pub mod inventory;
pub mod shop;
pub mod title;

use super::canvas::Canvas;
use super::{Area, theme};

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

/// Footer hint on the last inner row of a pop-up.
pub fn footer(canvas: &mut Canvas, inner: Area, text: &str) {
    let y = inner.y + inner.h.saturating_sub(1);
    canvas.text_max(inner.x, y, text, inner.w, theme::DIM, theme::POPUP);
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
