//! Sidebar status panel and message log.

use crossterm::style::Color;

use super::canvas::Canvas;
use super::{Area, theme};
use crate::creatures::player::Sleep;
use crate::game::Game;
use crate::items::weight::fmt_lb;
use crate::items::{CRYSTAL_GLYPH, GUN_GLYPH};

/// A text bar like `████████░░░░`.
pub fn bar(value: i32, max: i32, width: usize) -> String {
    let filled = if max <= 0 { 0 } else { ((value.max(0) as usize) * width).div_ceil(max as usize).min(width) };
    "█".repeat(filled) + &"░".repeat(width - filled)
}

pub fn draw_sidebar(canvas: &mut Canvas, area: Area, game: &Game) {
    let bg = theme::PANEL;
    canvas.frame(area, "roguemoji", theme::BORDER, bg);
    let x = area.x + 2;
    let max = area.w.saturating_sub(4);
    let mut y = area.y + 1;
    let bottom = area.y + area.h - 1;
    let p = &game.player;

    let line = |canvas: &mut Canvas, y: &mut u16, text: &str, fg: Color| {
        if *y < bottom {
            canvas.text_max(x, *y, text, max, fg, bg);
            *y += 1;
        }
    };

    line(canvas, &mut y, &format!("{} the {}", p.name, p.background), theme::TITLE);
    line(canvas, &mut y, &format!("Phase {} · {}", game.depth, game.biome.name()), theme::DIM);
    y += 1;

    let max_hp = p.max_hp();
    let hp_color = match p.hp * 100 / max_hp.max(1) {
        60.. => theme::GOOD,
        30..=59 => theme::WARN,
        _ => theme::BAD,
    };
    line(canvas, &mut y, &format!("HP {} {}/{}", bar(p.hp, max_hp, 14), p.hp, max_hp), hp_color);
    // Two needs per line: all four don't fit the sidebar.
    // Each label takes its own stage's color, so one bad need doesn't paint its neighbor.
    for pair in p.needs().chunks(2) {
        if y >= bottom {
            break;
        }
        let mut col = 0;
        for (i, m) in pair.iter().enumerate() {
            if i > 0 {
                col += canvas.text_max(x + col, y, "  ·  ", max.saturating_sub(col), theme::TEXT, bg);
            }
            let color = if m.stage().is_warning() { theme::BAD } else { theme::TEXT };
            col += canvas.text_max(x + col, y, m.label(), max.saturating_sub(col), color, bg);
        }
        y += 1;
    }
    line(canvas, &mut y, &format!("{} {} crystals", CRYSTAL_GLYPH, p.crystals), theme::TEXT);
    line(canvas, &mut y, &format!("DEF {}  ARMOR {}  Turn {}", p.defense(), p.armor_value(), game.turn), theme::TEXT);
    let burden = p.burden();
    let load = format!("Load {}/{} ({})", fmt_lb(p.carried_weight()), fmt_lb(p.carry_capacity()), burden.name());
    line(canvas, &mut y, &load, theme::burden(burden));
    let status = match p.sleep {
        Some(Sleep::Nap) => "💤 Napping (any key to get up)".to_string(),
        Some(Sleep::Collapse) => "💤 Out cold".to_string(),
        None => format!("Stance {}  {}", p.stance.name(), "▲".repeat(p.momentum.stacks() as usize)),
    };
    line(canvas, &mut y, &status, theme::WARN);
    y += 1;

    let melee = p.melee_def();
    let melee_skill = p.skill(melee.skill).total();
    line(canvas, &mut y, &format!("{} {} ({:+})", melee.glyph, melee.name, melee_skill), theme::TEXT);
    match &p.gun {
        Some(g) => {
            let skill = p.skill(g.def().skill).total();
            line(canvas, &mut y, &format!("{} {} ({:+})", GUN_GLYPH, g.def().name, skill), theme::TEXT);
            let status = g.status();
            let reserve = g.ammo.map(|a| p.inventory.count_ammo(a)).unwrap_or(0);
            let color = if g.loaded == 0 { theme::BAD } else { theme::DIM };
            line(canvas, &mut y, &format!("   [{status}]  +{reserve} spare"), color);
        }
        None => line(canvas, &mut y, &format!("{GUN_GLYPH} no gun"), theme::DIM),
    }
    match p.armor {
        Some(a) => line(canvas, &mut y, &format!("{} {}", a.def().glyph, a.def().name), theme::TEXT),
        None => line(canvas, &mut y, "   no armor", theme::DIM),
    }
    if let Some(k) = p.pack {
        line(canvas, &mut y, &format!("{} {}", k.def().glyph, k.def().name), theme::TEXT);
    }

    let effects: Vec<_> = p.effects.iter().collect();
    if !effects.is_empty() {
        y += 1;
        for e in effects {
            let color = if e.kind.is_harmful() { theme::BAD } else { theme::GOOD };
            let amount = match e.kind {
                crate::effects::EffectKind::Boost(_) => format!(" {:+}", e.potency),
                _ => String::new(),
            };
            line(canvas, &mut y, &format!("• {}{} ({})", e.kind.label(), amount, e.turns), color);
        }
    }

    let seen = game.visible_monsters();
    if !seen.is_empty() && y + 2 < bottom {
        y += 1;
        line(canvas, &mut y, "In sight:", theme::DIM);
        for m in seen {
            let text = format!("{} {} — {}", m.def().glyph, m.name(), m.health_word());
            line(canvas, &mut y, &text, theme::TEXT);
        }
    }

    // Key reminder pinned to the bottom of the panel.
    let hint_y = bottom.saturating_sub(1);
    if hint_y > y {
        canvas.text_max(x, hint_y, "i inv · c char · m craft · ? help", max, theme::DIM, bg);
    }
}

pub fn draw_log(canvas: &mut Canvas, area: Area, game: &Game, status_line: Option<&str>) {
    canvas.fill(area.x, area.y, area.w, area.h, theme::BG);
    canvas.hline(area.x, area.y, area.w, theme::BORDER, theme::BG);
    let rows = area.h.saturating_sub(1) as usize;
    let (rows, status_row) = match status_line {
        Some(_) => (rows.saturating_sub(1), true),
        None => (rows, false),
    };
    let messages = game.log.recent(rows);
    for (i, msg) in messages.iter().enumerate() {
        let fade = i + 1 < messages.len().saturating_sub(2);
        let color = if fade { theme::DIM } else { theme::tone(msg.tone) };
        canvas.text_max(area.x + 1, area.y + 1 + i as u16, &msg.display(), area.w - 2, color, theme::BG);
    }
    if let (true, Some(s)) = (status_row, status_line) {
        let y = area.y + area.h - 1;
        canvas.fill(area.x, y, area.w, 1, theme::SELECTED);
        canvas.text_max(area.x + 1, y, s, area.w - 2, theme::TITLE, theme::SELECTED);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;

    fn sidebar(g: &Game) -> Vec<String> {
        let mut canvas = Canvas::new(40, 40);
        draw_sidebar(&mut canvas, Area::new(0, 0, 38, 40), g);
        canvas.text_rows()
    }

    #[test]
    fn shows_every_need() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        let rows = sidebar(&g);
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains("Fed  ·  Hydrated")));
        assert!(rows.iter().any(|r| r.contains("Rested  ·  Hopeful")));
        assert!(rows.iter().any(|r| r.contains("💎") && r.contains("crystals")));
        g.player.thirst.value = 0;
        assert!(sidebar(&g).iter().any(|r| r.contains("Parched")));
    }

    #[test]
    fn each_need_is_colored_by_its_own_stage() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        g.player.thirst.value = 0;
        let mut canvas = Canvas::new(40, 40);
        draw_sidebar(&mut canvas, Area::new(0, 0, 38, 40), &g);
        let rows = canvas.text_rows();
        let y = rows.iter().position(|r| r.contains("Fed  ·  Parched")).expect("needs line");
        let color_of = |word: &str| {
            let col = rows[y][..rows[y].find(word).unwrap()].chars().count();
            canvas.row(y as u16)[col].fg
        };
        assert_eq!(color_of("Fed"), theme::TEXT, "a fine need stays plain beside a bad one");
        assert_eq!(color_of("Parched"), theme::BAD);
    }
}
