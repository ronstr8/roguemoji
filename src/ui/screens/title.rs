//! Title screen and background selection.

use super::paragraph;
use crate::creatures::Background;
use crate::stats::{Attribute, Skill};
use crate::ui::canvas::{Canvas, text_width};
use crate::ui::{Area, theme};

const BANNER: &[&str] = &["💥  r o g u e m o j i  💥", "🐀 🧟 🦍   the world ended politely   📡 💎 🔫"];

pub fn draw(canvas: &mut Canvas, selected: usize) {
    let w = canvas.width();
    canvas.fill(0, 0, w, canvas.height(), theme::BG);
    let center = |s: &str| w.saturating_sub(text_width(s) as u16) / 2;
    let mut y = 1;
    for (i, line) in BANNER.iter().enumerate() {
        let color = if i == 0 { theme::TITLE } else { theme::DIM };
        canvas.text(center(line), y, line, color, theme::BG);
        y += 2;
    }

    let panel_w = 84.min(w.saturating_sub(4));
    let x = w.saturating_sub(panel_w) / 2;
    let panel = Area::new(x, y, panel_w, canvas.height().saturating_sub(y + 1));
    canvas.frame(panel, "Who were you, before?", theme::BORDER, theme::POPUP);
    let ix = x + 2;
    let mut iy = y + 2;
    for (i, bg) in Background::ALL.iter().enumerate() {
        let rbg = if i == selected { theme::SELECTED } else { theme::POPUP };
        canvas.fill(ix, iy, 22, 1, rbg);
        canvas.text(ix, iy, &format!("{}) {} {}", i + 1, bg.glyph(), bg.name()), theme::TEXT, rbg);
        iy += 2;
    }

    let bg = Background::ALL[selected];
    let preview = bg.create(String::new());
    let dx = ix + 25;
    let dw = panel_w.saturating_sub(29);
    let bottom = canvas.height().saturating_sub(3);
    let mut dy = y + 2;
    canvas.text(dx, dy, bg.name(), theme::TITLE, theme::POPUP);
    dy += 1;
    dy = paragraph(canvas, dx, dy, dw, bottom, bg.pitch(), theme::TEXT) + 1;
    let attrs: Vec<String> =
        Attribute::ALL.iter().map(|a| format!("{} {}", a.abbrev(), preview.base.get(*a))).collect();
    dy = paragraph(canvas, dx, dy, dw, bottom, &attrs.join("  "), theme::WARN) + 1;
    let skills: Vec<String> = Skill::ALL
        .iter()
        .filter(|s| preview.skills.rank(**s) > 0)
        .map(|s| format!("{} {}", s.name(), preview.skills.rank(*s)))
        .collect();
    dy = paragraph(canvas, dx, dy, dw, bottom, &format!("Skills: {}", skills.join(", ")), theme::TEXT) + 1;
    let mut kit: Vec<String> = Vec::new();
    if let Some(m) = preview.melee {
        kit.push(format!("{} {}", m.def().glyph, m.def().name));
    }
    if let Some(g) = &preview.gun {
        kit.push(format!("🔫 {}", g.def().name));
    }
    if let Some(a) = preview.armor {
        kit.push(format!("{} {}", a.def().glyph, a.def().name));
    }
    kit.extend(preview.inventory.items().iter().map(|i| format!("{} {}", i.glyph(), i.display_name())));
    paragraph(canvas, dx, dy, dw, bottom, &format!("Kit: {}", kit.join(" · ")), theme::DIM);

    let hint = "↑↓ / 1-4 choose · Enter begin · Q quit";
    canvas.text(center(hint), canvas.height().saturating_sub(1), hint, theme::DIM, theme::BG);
}
