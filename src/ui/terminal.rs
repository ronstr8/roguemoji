//! Terminal lifecycle and flicker-free presentation of a `Canvas`.

use std::io::{self, Stdout, Write};

use crossterm::style::{Color, Print, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{self, BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{cursor, execute, queue};

use super::canvas::Canvas;

pub struct Terminal {
    out: Stdout,
    previous: Option<Canvas>,
}

impl Terminal {
    pub fn enter() -> io::Result<Self> {
        let mut out = io::stdout();
        terminal::enable_raw_mode()?;
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide, terminal::Clear(terminal::ClearType::All))?;
        Ok(Self { out, previous: None })
    }

    pub fn size() -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    /// Force a full redraw on the next present (e.g. after a resize).
    pub fn invalidate(&mut self) {
        self.previous = None;
        let _ = execute!(self.out, terminal::Clear(terminal::ClearType::All));
    }

    /// Write only the rows that changed since the last frame.
    pub fn present(&mut self, canvas: &Canvas) -> io::Result<()> {
        let full = self.previous.as_ref().is_none_or(|p| p.width() != canvas.width() || p.height() != canvas.height());
        queue!(self.out, BeginSynchronizedUpdate)?;
        for y in 0..canvas.height() {
            let row = canvas.row(y);
            if !full && self.previous.as_ref().is_some_and(|p| p.row(y) == row) {
                continue;
            }
            queue!(self.out, cursor::MoveTo(0, y))?;
            let mut fg: Option<Color> = None;
            let mut bg: Option<Color> = None;
            let mut run = String::new();
            for cell in row {
                if cell.cont {
                    continue;
                }
                if fg != Some(cell.fg) || bg != Some(cell.bg) {
                    if !run.is_empty() {
                        queue!(self.out, Print(std::mem::take(&mut run)))?;
                    }
                    if fg != Some(cell.fg) {
                        queue!(self.out, SetForegroundColor(cell.fg))?;
                        fg = Some(cell.fg);
                    }
                    if bg != Some(cell.bg) {
                        queue!(self.out, SetBackgroundColor(cell.bg))?;
                        bg = Some(cell.bg);
                    }
                }
                run.push(cell.ch);
            }
            if !run.is_empty() {
                queue!(self.out, Print(run))?;
            }
        }
        queue!(self.out, EndSynchronizedUpdate)?;
        self.out.flush()?;
        self.previous = Some(canvas.clone());
        Ok(())
    }
}

fn restore() {
    let mut out = io::stdout();
    let _ = execute!(out, crossterm::style::ResetColor, cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}

/// Make sure a panic leaves the user's terminal usable.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        default(info);
    }));
}
