//! The scrolling map viewport. Every map cell is two terminal columns wide.

use std::collections::{HashMap, HashSet};

use super::canvas::Canvas;
use super::{Area, theme};
use crate::game::Game;
use crate::geometry::{Pos, line};

pub const PLAYER_GLYPH: &str = "🧑";

fn camera(origin: i32, view: i32, size: i32) -> i32 {
    (origin - view / 2).clamp(0, (size - view).max(0))
}

pub fn draw(canvas: &mut Canvas, area: Area, game: &Game, target: Option<Pos>) {
    canvas.fill(area.x, area.y, area.w, area.h, theme::BG);
    let map = &game.map;
    let view_w = (area.w / 2) as i32;
    let view_h = area.h as i32;
    let me = game.player.pos;
    let cam_x = camera(me.x, view_w, map.width());
    let cam_y = camera(me.y, view_h, map.height());

    let monsters: HashMap<Pos, &str> =
        game.monsters.iter().filter(|m| !m.is_dead()).map(|m| (m.pos, m.def().glyph.as_str())).collect();
    let items: HashMap<Pos, &str> = game.items.iter().map(|fi| (fi.pos, fi.item.glyph())).collect();
    let path: HashSet<Pos> = target.map(|t| line(me, t).into_iter().skip(1).collect()).unwrap_or_default();

    for vy in 0..view_h {
        for vx in 0..view_w {
            let p = Pos::new(cam_x + vx, cam_y + vy);
            if !map.in_bounds(p) || !map.is_explored(p) {
                continue;
            }
            let sx = area.x + (vx * 2) as u16;
            let sy = area.y + vy as u16;
            let bg = if Some(p) == target {
                theme::TARGET_BG
            } else if path.contains(&p) {
                theme::PATH_BG
            } else {
                theme::BG
            };
            let tile = map.tile(p);
            if map.is_visible(p) {
                let glyph = if p == me {
                    Some(PLAYER_GLYPH)
                } else {
                    monsters.get(&p).or_else(|| items.get(&p)).copied().or_else(|| map.feature(p).map(|f| f.glyph()))
                };
                match glyph {
                    Some(g) => canvas.text(sx, sy, g, theme::TEXT, bg),
                    None => canvas.text(sx, sy, tile.glyph(), tile.color(), bg),
                };
            } else {
                // Remembered terrain is drawn dim; features are remembered as-is.
                match map.feature(p) {
                    Some(f) => canvas.text(sx, sy, f.glyph(), theme::MEMORY, bg),
                    None => canvas.text(sx, sy, tile.glyph(), theme::MEMORY, bg),
                };
            }
        }
    }
}
