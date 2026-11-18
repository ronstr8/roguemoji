//! Help pop-up: controls and core rules.

use super::{footer, popup};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

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
    ("COMBAT", "2d10 + skill vs Defense. Margin 0-3 graze (½), 4-8 hit, 9+ crit (×1.5, ignores armor)."),
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
    ("LOAD", "Carry 45 lb + 6 per Brawn point over 5, plus a worn pack 🎒. Over that you're Burdened"),
    ("", "(slower, -1 DEF); over 1.5× Strained (half speed, -3 DEF); over 2× you can't move."),
    ("🛅", "Visitor lockboxes share one stash: leave gear in one, take it out of any other."),
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
