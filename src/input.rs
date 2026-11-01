//! Translates key presses into play-mode commands.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::geometry::Dir;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Move(Dir),
    Wait,
    Fire,
    Reload,
    Pickup,
    Absorb { to_full: bool },
    Descend,
    Stance,
    Inventory,
    Character,
    Crafting,
    Help,
    Quit,
}

pub fn play_command(key: KeyEvent) -> Option<Command> {
    use Command::*;
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Some(Quit),
            _ => None,
        };
    }
    let cmd = match key.code {
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('8') => Move(Dir::N),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('2') => Move(Dir::S),
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('4') => Move(Dir::W),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('6') => Move(Dir::E),
        KeyCode::Char('y') | KeyCode::Char('7') | KeyCode::Home => Move(Dir::NW),
        KeyCode::Char('u') | KeyCode::Char('9') | KeyCode::PageUp => Move(Dir::NE),
        KeyCode::Char('b') | KeyCode::Char('1') | KeyCode::End => Move(Dir::SW),
        KeyCode::Char('n') | KeyCode::Char('3') | KeyCode::PageDown => Move(Dir::SE),
        KeyCode::Char('.') | KeyCode::Char('5') => Wait,
        KeyCode::Char('f') => Fire,
        KeyCode::Char('r') => Reload,
        KeyCode::Char('g') | KeyCode::Char(',') => Pickup,
        KeyCode::Char('a') => Absorb { to_full: false },
        KeyCode::Char('A') => Absorb { to_full: true },
        KeyCode::Char('>') => Descend,
        KeyCode::Char('s') => Stance,
        KeyCode::Char('i') => Inventory,
        KeyCode::Char('c') => Character,
        KeyCode::Char('m') => Crafting,
        KeyCode::Char('?') => Help,
        KeyCode::Char('Q') => Quit,
        _ => return None,
    };
    Some(cmd)
}

/// Shared list navigation for pop-ups: returns the new selection.
pub fn list_nav(key: KeyEvent, selected: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => selected.checked_sub(1).unwrap_or(len - 1),
        KeyCode::Down | KeyCode::Char('j') => (selected + 1) % len,
        KeyCode::PageUp => selected.saturating_sub(10),
        KeyCode::PageDown => (selected + 10).min(len - 1),
        _ => selected.min(len - 1),
    }
}

pub fn is_cancel(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Esc | KeyCode::Char('q'))
}

pub fn is_confirm(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
}
