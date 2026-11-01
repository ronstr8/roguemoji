//! An off-screen character grid that understands double-width glyphs.
//!
//! Emoji occupy two terminal columns. A wide glyph is stored in its first
//! cell and the second cell is marked as a continuation, so drawing and
//! diffing stay column-accurate.

use crossterm::style::Color;
use unicode_width::UnicodeWidthChar;

use super::{Area, theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    /// Right half of a wide glyph; not printed itself.
    pub cont: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: theme::TEXT, bg: theme::BG, cont: false }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

/// Display width of a string in terminal columns.
pub fn text_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, cells: vec![Cell::default(); width as usize * height as usize] }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn row(&self, y: u16) -> &[Cell] {
        let start = y as usize * self.width as usize;
        &self.cells[start..start + self.width as usize]
    }

    fn idx(&self, x: u16, y: u16) -> Option<usize> {
        (x < self.width && y < self.height).then(|| y as usize * self.width as usize + x as usize)
    }

    fn set(&mut self, x: u16, y: u16, cell: Cell) {
        let Some(i) = self.idx(x, y) else { return };
        // Overwriting the right half of a wide glyph breaks the left half.
        if self.cells[i].cont && x > 0 {
            self.cells[i - 1].ch = ' ';
        }
        // Overwriting a wide glyph's left half orphans its right half.
        if !cell.cont && x + 1 < self.width && self.cells[i + 1].cont {
            self.cells[i + 1] = Cell { ch: ' ', cont: false, ..self.cells[i + 1] };
        }
        self.cells[i] = cell;
    }

    /// Draw a string; returns the number of columns used. Clips at the right edge.
    pub fn text(&mut self, x: u16, y: u16, s: &str, fg: Color, bg: Color) -> u16 {
        self.text_max(x, y, s, u16::MAX, fg, bg)
    }

    /// Draw a string clipped to `max` columns.
    pub fn text_max(&mut self, x: u16, y: u16, s: &str, max: u16, fg: Color, bg: Color) -> u16 {
        let mut col = 0u16;
        for ch in s.chars() {
            let w = ch.width().unwrap_or(0) as u16;
            if w == 0 {
                continue;
            }
            let cx = x.saturating_add(col);
            if col + w > max || cx + w > self.width {
                break;
            }
            self.set(cx, y, Cell { ch, fg, bg, cont: false });
            if w == 2 {
                self.set(cx + 1, y, Cell { ch: ' ', fg, bg, cont: true });
            }
            col += w;
        }
        col
    }

    /// Fade everything already drawn, as a backdrop for a modal pop-up.
    pub fn dim(&mut self) {
        for c in &mut self.cells {
            c.fg = theme::MEMORY;
            c.bg = theme::BG;
        }
    }

    pub fn fill(&mut self, x: u16, y: u16, w: u16, h: u16, bg: Color) {
        for yy in y..y.saturating_add(h).min(self.height) {
            for xx in x..x.saturating_add(w).min(self.width) {
                self.set(xx, yy, Cell { ch: ' ', fg: theme::TEXT, bg, cont: false });
            }
        }
    }

    pub fn hline(&mut self, x: u16, y: u16, w: u16, fg: Color, bg: Color) {
        for xx in x..x.saturating_add(w).min(self.width) {
            self.set(xx, y, Cell { ch: '─', fg, bg, cont: false });
        }
    }

    /// A filled, bordered box with an optional title in the top edge.
    pub fn frame(&mut self, area: Area, title: &str, fg: Color, bg: Color) {
        let Area { x, y, w, h } = area;
        if w < 2 || h < 2 {
            return;
        }
        self.fill(x, y, w, h, bg);
        let (r, b) = (x + w - 1, y + h - 1);
        for xx in x + 1..r {
            self.set(xx, y, Cell { ch: '─', fg, bg, cont: false });
            self.set(xx, b, Cell { ch: '─', fg, bg, cont: false });
        }
        for yy in y + 1..b {
            self.set(x, yy, Cell { ch: '│', fg, bg, cont: false });
            self.set(r, yy, Cell { ch: '│', fg, bg, cont: false });
        }
        self.set(x, y, Cell { ch: '╭', fg, bg, cont: false });
        self.set(r, y, Cell { ch: '╮', fg, bg, cont: false });
        self.set(x, b, Cell { ch: '╰', fg, bg, cont: false });
        self.set(r, b, Cell { ch: '╯', fg, bg, cont: false });
        if !title.is_empty() {
            let t = format!(" {title} ");
            self.text_max(x + 2, y, &t, w.saturating_sub(4), theme::TITLE, bg);
        }
    }
}

/// Greedy word wrap to `width` columns.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let needed = text_width(&line) + usize::from(!line.is_empty()) + text_width(word);
        if needed > width && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_glyphs_take_two_columns() {
        let mut c = Canvas::new(10, 1);
        let used = c.text(0, 0, "a💎b", theme::TEXT, theme::BG);
        assert_eq!(used, 4);
        assert!(c.row(0)[2].cont);
        assert_eq!(c.row(0)[3].ch, 'b');
    }

    #[test]
    fn overwriting_half_a_wide_glyph_clears_it() {
        let mut c = Canvas::new(4, 1);
        c.text(0, 0, "💎", theme::TEXT, theme::BG);
        c.text(1, 0, "x", theme::TEXT, theme::BG);
        assert_eq!(c.row(0)[0].ch, ' ');
        assert_eq!(c.row(0)[1].ch, 'x');
    }

    #[test]
    fn wrap_respects_width() {
        let lines = wrap("the quick brown fox jumps", 10);
        assert!(lines.iter().all(|l| text_width(l) <= 10));
        assert_eq!(lines.join(" "), "the quick brown fox jumps");
    }
}
