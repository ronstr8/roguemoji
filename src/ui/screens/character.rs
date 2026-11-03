//! Character sheet pop-up: attributes, skills and derived values.

use super::{footer, paragraph, popup};
use crate::creatures::player::CAPACITY_PER_BRAWN;
use crate::game::Game;
use crate::items::weight::fmt_lb;
use crate::stats::attributes::bonus;
use crate::stats::skills::MAX_RANK;
use crate::stats::{Attribute, Skill};
use crate::ui::canvas::Canvas;
use crate::ui::hud::bar;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game) {
    let p = &game.player;
    let inner = popup(canvas, 84, 34, &format!("{} the {}", p.name, p.background));
    let (x, mut y) = (inner.x, inner.y);
    let bg = theme::POPUP;

    canvas.text(x, y, "Physical", theme::TITLE, bg);
    canvas.text(x + 30, y, "Mental", theme::TITLE, bg);
    y += 1;
    let attr_cell = |canvas: &mut Canvas, cx: u16, cy: u16, a: Attribute| {
        let base = p.base.get(a);
        let eff = p.attr(a);
        let color = match eff.cmp(&base) {
            std::cmp::Ordering::Greater => theme::GOOD,
            std::cmp::Ordering::Less => theme::BAD,
            std::cmp::Ordering::Equal => theme::TEXT,
        };
        canvas.text(cx, cy, &format!("{:<10} {:>2} ({:+})", a.name(), eff, bonus(eff)), color, bg);
    };
    for (phys, ment) in Attribute::PAIRS {
        attr_cell(canvas, x, y, phys);
        attr_cell(canvas, x + 30, y, ment);
        y += 1;
    }
    y += 1;

    canvas.text(x, y, "Skill        Rank    Mod   (rank + attr)   Practice", theme::TITLE, bg);
    y += 1;
    for s in Skill::ALL {
        let b = p.skill(s);
        let rank = p.skills.rank(s);
        let pips = "●".repeat(rank as usize) + &"○".repeat((MAX_RANK - rank) as usize);
        let practice = match p.skills.xp_needed(s) {
            Some(n) => format!("{} {}/{}", bar(p.skills.xp(s) as i32, n as i32, 8), p.skills.xp(s), n),
            None => "mastered".into(),
        };
        let (a1, a2) = s.attributes();
        let note = if b.halved { "½" } else { " " };
        let line = format!(
            "{:<12} {}  {:>+3}   ({:+} {:+}{note} {}/{})  {}",
            s.name(),
            pips,
            b.total(),
            b.rank_bonus,
            b.attr_bonus,
            a1.abbrev(),
            a2.abbrev(),
            practice
        );
        let color = if b.unskilled { theme::DIM } else { theme::TEXT };
        canvas.text_max(x, y, &line, inner.w, color, bg);
        y += 1;
    }
    y += 1;

    let derived = [
        format!("Max HP {}  ·  Defense {}  ·  Armor {}", p.max_hp(), p.defense(), p.armor_value()),
        format!(
            "Each 💎 heals {}  ·  Crystals {}  ·  Satiety {} ({})",
            p.crystal_heal(),
            p.crystals,
            p.satiety.max(0),
            p.hunger_label()
        ),
        format!(
            "Carry {} (Brawn {:+} lb{})  ·  Load {}  ·  {}",
            fmt_lb(p.carry_capacity()),
            bonus(p.attr(Attribute::Brawn)) * CAPACITY_PER_BRAWN / 100,
            p.pack.map_or(String::new(), |k| format!(", pack +{}", fmt_lb(k.def().capacity))),
            fmt_lb(p.carried_weight()),
            p.burden().name()
        ),
        format!("Kills {}  ·  Depth {}  ·  Turn {}", p.kills, game.depth, game.turn),
    ];
    for line in derived {
        canvas.text_max(x, y, &line, inner.w, theme::TEXT, bg);
        y += 1;
    }
    y += 1;
    let rules = "Checks roll 2d10 + skill vs a target: margin 0-3 partial, 4-8 success, 9+ critical. \
        Skill = rank×2 + the average of its two attribute bonuses. ½ marks an unskilled bonus: only half of a \
        positive attribute bonus applies (penalties apply in full). Skills improve with successful use.";
    paragraph(canvas, x, y, inner.w, inner.y + inner.h.saturating_sub(1), rules, theme::DIM);
    footer(canvas, inner, "Esc close");
}
