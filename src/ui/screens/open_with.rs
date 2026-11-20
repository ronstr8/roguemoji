//! Open-with pop-up: the ways into a cache or node, with your odds for each.

use super::{footer, popup};
use crate::game::Game;
use crate::geometry::Pos;
use crate::ui::canvas::Canvas;
use crate::ui::theme;

const WIDTH: u16 = 64;

pub fn draw(canvas: &mut Canvas, game: &Game, pos: Pos, selected: usize) {
    let Some(def) = game.node_at(pos) else { return };
    let height = def.open.len() as u16 + 4;
    let inner = popup(canvas, WIDTH, height, &format!("{} {}", def.glyph, def.name));
    let p = &game.player;
    for (i, way) in def.open.iter().enumerate() {
        let y = inner.y + i as u16;
        let bg = if i == selected { theme::SELECTED } else { theme::POPUP };
        canvas.fill(inner.x, y, inner.w, 1, bg);
        let tool = way.tool.map(|t| format!(" · {} {}", t.glyph(), t.name())).unwrap_or_default();
        let left = format!("{}) {} ({}{tool})", i + 1, way.verb, way.test_name());
        let (right, color) = if way.ready(p) {
            (format!("{:>3.0}%", way.odds(p) * 100.0), theme::TEXT)
        } else {
            ("no tool".to_string(), theme::DIM)
        };
        let left_w = inner.w.saturating_sub(right.len() as u16 + 1);
        canvas.text_max(inner.x, y, &left, left_w, color, bg);
        canvas.text(inner.x + inner.w.saturating_sub(right.len() as u16), y, &right, color, bg);
    }
    footer(canvas, inner, "↑↓ / 1-9 pick · Enter try · Esc leave it");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::world::{Feature, Harvestable};

    #[test]
    fn lists_each_way_with_odds_or_whats_missing() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        let pos = g.player.pos.step(crate::geometry::Dir::E);
        g.map.set_feature(pos, Feature::harvest(Harvestable::named("military_crate")));
        g.player.melee = None;
        let mut canvas = Canvas::new(100, 30);
        draw(&mut canvas, &g, pos, 0);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains("1) pick the lock on (Lockpicking · 🪛 Lockpicks)") && r.contains('%')));
        assert!(rows.iter().any(|r| r.contains("2) pry open (Brawn · 🦯 Crowbar)") && r.contains("no tool")));
        assert!(rows.iter().any(|r| r.contains("Esc leave it")));
    }
}
