//! Help pop-up: controls and core rules.

use super::{footer, popup};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

const LINES: &[(&str, &str)] = &[
    ("MOVE", "arrows · hjkl yubn · numpad.  Bump to attack, harvest 🌿🚗, or use 🏧"),
    (". / 5", "wait a turn"),
    ("f", "fire gun: Tab/←→ cycle targets, f/Enter shoot, Esc cancel"),
    ("r", "reload (revolvers and tube magazines take longer)"),
    ("g ,", "pick up items and crystals"),
    ("a / A", "absorb one crystal / absorb until healed"),
    ("s", "cycle stance: Aggressive ⇄ Balanced ⇄ Guarded"),
    (">", "descend when standing on 🔽"),
    ("i c m", "inventory · character sheet · crafting"),
    ("Q", "quit"),
    ("", ""),
    ("COMBAT", "2d10 + skill vs Defense. Margin 0-3 graze (½), 4-8 hit, 9+ crit (×1.5, ignores armor)."),
    ("", "Momentum: consecutive solid hits on one target add +1 to hit (max +3)."),
    ("", "Getting hurt, missing, or switching targets resets it."),
    ("GUNS", "Damage comes from muzzle energy: bullet weight × velocity², which depends on barrel"),
    ("", "length. JHP/SP wound more; FMJ/slugs penetrate armor. Buckshot spreads with distance."),
    ("CRYSTALS", "Everything that dies leaves 💎. Absorb them to heal, or spend them at Autovends."),
    ("FOOD", "Harvest components and craft meals and drinks for timed attribute buffs."),
    ("GOAL", "Find the Visitor Beacon 📡 on depth 10."),
];

pub fn draw(canvas: &mut Canvas) {
    let inner = popup(canvas, 100, LINES.len() as u16 + 5, "Help");
    for (i, (key, text)) in LINES.iter().enumerate() {
        let y = inner.y + i as u16;
        if y + 1 >= inner.y + inner.h {
            break;
        }
        canvas.text(inner.x, y, key, theme::TITLE, theme::POPUP);
        canvas.text_max(inner.x + 10, y, text, inner.w.saturating_sub(10), theme::TEXT, theme::POPUP);
    }
    footer(canvas, inner, "Esc close");
}
