//! Help pop-up: controls and core rules. Scrolls when the terminal is too
//! short to show it all.

use crossterm::event::{KeyCode, KeyEvent};

use super::{footer, popup};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

/// Rows the frame, a gap and the footer take beyond the lines themselves.
const CHROME: u16 = 4;

const LINES: &[(&str, &str)] = &[
    ("MOVE", "arrows · hjkl yubn · numpad.  Bump to attack, harvest 🌿🚗, or use 🏧 🛅"),
    ("Space .", "wait a turn"),
    ("s", "shoot: fires at the only target in range, or pick from a list (nearest first)"),
    ("f", "fire gun: Tab/←→ cycle targets, f/Enter shoot, Esc cancel"),
    ("r", "reload (revolvers and tube magazines take longer)"),
    ("g , 5", "pick up items and crystals"),
    ("a / A", "absorb one crystal / absorb until healed"),
    ("d", "drink from water underfoot or next to you (a Survival check, or you get sick)"),
    ("z", "nap until rested: a monster in view, a hit or any key wakes you"),
    ("S", "cycle stance: Aggressive ⇄ Balanced ⇄ Guarded"),
    (">", "descend when standing on 🔽"),
    ("i c m", "inventory · character sheet · crafting"),
    ("Q", "quit"),
    ("", ""),
    ("COMBAT", "2d10 + skill vs Defense. Margin 0-3 graze (½), 4-8 hit, 9+ crit (×1.5, skips armor)."),
    ("", "Momentum: consecutive solid hits on one target add +1 to hit (max +3)."),
    ("", "Getting hurt, missing, or switching targets resets it."),
    ("GUNS", "Damage comes from muzzle energy: bullet weight × velocity², which depends on barrel"),
    ("", "length. JHP/SP wound more; FMJ/slugs penetrate armor. Buckshot spreads with distance."),
    ("", "Max range is effective range + 1 tile per 400 fps; you can't shoot past it."),
    ("CRYSTALS", "Everything that dies leaves 💎. Absorb them to heal, or buy and sell at Autovends."),
    ("FOOD", "Harvest components and craft meals and drinks for timed attribute buffs."),
    ("WATER", "Thirst drains faster than hunger and dulls your mind. Bottled water is safe; dirty"),
    ("", "water and pools can make you sick. Boil dirty water (m) to make it safe."),
    ("REST", "Every turn tires you, fighting twice as fast, heavy loads faster. Tired and Exhausted"),
    ("", "slow you; at empty you collapse and sleep where you stand. Sleep heals a little."),
    ("HOPE", "Fades with time. Walk into encounters 🤝 😱: kindness restores it, horror erodes it."),
    ("", "Places like 🧘 stay; bump them again. Despair lowers Willpower: crystals heal less."),
    ("BETS", "Gamblers 🃏 take one bet and move on; dens 🎲 stay until the house is broke. You stake"),
    ("", "crystals, so you're betting your healing. Some games are luck; others test a stat."),
    ("LOAD", "Carry 45 lb + 6 per Brawn point over 5, plus a worn pack 🎒. Over that you're Burdened"),
    ("", "(slower, -1 DEF); over 1.5× Strained (half speed, -3 DEF); over 2× you can't move."),
    ("🛅", "Visitor lockboxes share one stash: leave gear in one, take it out of any other."),
    ("GOAL", "Find the Visitor Beacon 📡 on depth 10."),
];

/// How many help lines fit in a terminal `height` rows tall.
fn visible_rows(height: u16) -> usize {
    let want = LINES.len() as u16 + CHROME;
    let frame = want.min(height.saturating_sub(2)).max(6);
    frame.saturating_sub(CHROME) as usize
}

/// The furthest the help can scroll in a terminal `height` rows tall.
pub fn max_scroll(height: u16) -> usize {
    LINES.len().saturating_sub(visible_rows(height))
}

/// The scroll position after `key`, kept within `0..=max`.
pub fn scroll(key: KeyEvent, at: usize, max: usize) -> usize {
    let page = 10;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => at.saturating_sub(1),
        KeyCode::Down | KeyCode::Char('j') => at + 1,
        KeyCode::PageUp => at.saturating_sub(page),
        KeyCode::PageDown => at + page,
        KeyCode::Home => 0,
        KeyCode::End => max,
        _ => at,
    }
    .min(max)
}

pub fn draw(canvas: &mut Canvas, scroll: usize) {
    let inner = popup(canvas, 100, LINES.len() as u16 + CHROME, "Help");
    let rows = visible_rows(canvas.height());
    let first = scroll.min(max_scroll(canvas.height()));
    for (y, (key, text)) in (inner.y..).zip(LINES.iter().skip(first).take(rows)) {
        canvas.text(inner.x, y, key, theme::TITLE, theme::POPUP);
        canvas.text_max(inner.x + 10, y, text, inner.w.saturating_sub(10), theme::TEXT, theme::POPUP);
    }
    let hint = if rows < LINES.len() {
        let last = (first + rows).min(LINES.len());
        format!("↑↓ PgUp PgDn scroll ({}-{} of {}) · Esc close", first + 1, last, LINES.len())
    } else {
        "Esc close".to_string()
    };
    footer(canvas, inner, &hint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn rows(height: u16, scroll: usize) -> Vec<String> {
        let mut canvas = Canvas::new(100, height);
        draw(&mut canvas, scroll);
        canvas.text_rows()
    }

    #[test]
    fn scrolls_on_a_small_terminal_and_not_on_a_big_one() {
        assert_eq!(max_scroll(80), 0, "everything fits");
        let max = max_scroll(24);
        assert!(max > 0);
        let top = rows(24, 0);
        println!("{}", top.join("\n"));
        assert!(top.iter().any(|r| r.contains("MOVE")));
        assert!(top.iter().any(|r| r.contains("scroll (1-")));
        let bottom = rows(24, max);
        assert!(bottom.iter().any(|r| r.contains("GOAL")), "the last line is reachable");
        assert!(!bottom.iter().any(|r| r.contains("MOVE")));
        assert!(rows(80, 0).iter().any(|r| r.contains("GOAL") && !r.contains("scroll")));
    }

    #[test]
    fn keys_scroll_within_bounds() {
        let key = |code| KeyEvent::new(code, KeyModifiers::NONE);
        assert_eq!(scroll(key(KeyCode::Up), 0, 9), 0);
        assert_eq!(scroll(key(KeyCode::Down), 8, 9), 9);
        assert_eq!(scroll(key(KeyCode::Down), 9, 9), 9);
        assert_eq!(scroll(key(KeyCode::PageDown), 2, 9), 9);
        assert_eq!(scroll(key(KeyCode::End), 0, 9), 9);
        assert_eq!(scroll(key(KeyCode::Home), 7, 9), 0);
    }
}
