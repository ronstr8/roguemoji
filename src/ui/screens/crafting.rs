//! Crafting pop-up: recipe list, ingredients on hand, odds of success.

use super::{footer, paragraph, popup, scroll_offset};
use crate::crafting::RECIPES;
use crate::game::Game;
use crate::stats::Skill;
use crate::stats::check::success_chance;
use crate::ui::canvas::Canvas;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game, selected: usize) {
    let inner = popup(canvas, 100, 30, "Crafting");
    let list_w = 34.min(inner.w / 2);
    let bottom = inner.y + inner.h.saturating_sub(2);
    let visible = bottom.saturating_sub(inner.y) as usize;
    let offset = scroll_offset(selected, visible);
    let bg = theme::POPUP;

    for (row, (i, r)) in RECIPES.iter().enumerate().skip(offset).take(visible).enumerate() {
        let y = inner.y + row as u16;
        let rbg = if i == selected { theme::SELECTED } else { bg };
        let ok = game.can_craft(r);
        canvas.fill(inner.x, y, list_w, 1, rbg);
        let mark = if ok { "✔" } else { " " };
        let color = if ok { theme::GOOD } else { theme::DIM };
        canvas.text_max(inner.x, y, &format!("{mark} {} {}", r.glyph(), r.name()), list_w, color, rbg);
    }

    let Some(r) = RECIPES.get(selected) else { return };
    let dx = inner.x + list_w + 3;
    let dw = inner.w.saturating_sub(list_w + 3);
    let mut y = inner.y;
    canvas.text_max(dx, y, &format!("{} {}", r.glyph(), r.name()), dw, theme::TITLE, bg);
    y += 2;
    canvas.text(dx, y, "Requires:", theme::DIM, bg);
    y += 1;
    let inv = &game.player.inventory;
    for (c, n) in r.components() {
        let have = inv.count_component(c);
        let color = if have >= n { theme::GOOD } else { theme::BAD };
        canvas.text_max(dx, y, &format!("  {} {}  {}/{}", c.def().glyph, c.def().name, have, n), dw, color, bg);
        y += 1;
    }
    if let Some(tool) = r.consumes_tool {
        let color = if game.has_tool(r) { theme::GOOD } else { theme::BAD };
        canvas.text_max(dx, y, &format!("  {} {} (consumed)", tool.def().glyph, tool.def().name), dw, color, bg);
        y += 1;
    }
    y += 1;
    let skill = game.player.skill(Skill::Crafting).total();
    let chance = success_chance(skill, r.difficulty) * 100.0;
    canvas.text_max(
        dx,
        y,
        &format!("Difficulty {} · Crafting {:+} · {:.0}% success", r.difficulty, skill, chance),
        dw,
        theme::WARN,
        bg,
    );
    y += 2;
    for line in r.output.item().describe() {
        y = paragraph(canvas, dx, y, dw, bottom, &line, theme::TEXT);
    }
    y += 1;
    paragraph(
        canvas,
        dx,
        y,
        dw,
        bottom,
        "Fail: nothing lost. Fumble: materials ruined. Critical: double food, or spare material.",
        theme::DIM,
    );
    footer(canvas, inner, "↑↓ select · Enter craft (2 turns) · Esc close");
}
