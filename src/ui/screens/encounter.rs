//! Encounter pop-up: the scene you walked into and what it did to you.

use super::{footer, paragraph, popup};
use crate::game::{EncounterReport, Game};
use crate::ui::canvas::{Canvas, wrap};
use crate::ui::theme;

const WIDTH: u16 = 66;

pub fn draw(canvas: &mut Canvas, game: &Game) {
    let Some(report) = &game.last_encounter else { return };
    let def = report.encounter.def();
    let body_w = WIDTH - 4;
    let text_lines = wrap(&report.text, body_w as usize).len() as u16;
    let outcome = outcome_lines(report);
    let height = text_lines + outcome.len() as u16 + 6;
    let inner = popup(canvas, WIDTH, height, &format!("{} {}", def.glyph, def.name));
    let bottom = inner.y + inner.h.saturating_sub(1);

    let mut y = paragraph(canvas, inner.x, inner.y, inner.w, bottom, &report.text, theme::TEXT) + 1;
    for (line, color) in outcome {
        y = paragraph(canvas, inner.x, y, inner.w, bottom, &line, color);
    }
    footer(canvas, inner, "Enter / Esc continue");
}

/// What happened, one line each: the resist check, need changes, gifts.
fn outcome_lines(report: &EncounterReport) -> Vec<(String, crossterm::style::Color)> {
    let mut lines = Vec::new();
    if let (Some(check), Some(resist)) = (report.resist, &report.encounter.def().resist) {
        let name = resist.attribute.name();
        let line = if report.resisted() {
            let text = report.resist_text.as_deref().unwrap_or_default();
            (format!("{} {name}: {text}", check.summary()), theme::GOOD)
        } else {
            (format!("{} {name}: it gets to you.", check.summary()), theme::BAD)
        };
        lines.push(line);
    }
    for (need, delta) in &report.changes {
        let color = if *delta < 0 { theme::BAD } else { theme::GOOD };
        lines.push((format!("{} {delta:+}", need.def().name), color));
    }
    for item in &report.gifts {
        lines.push((format!("You receive {} {}.", item.glyph(), item.display_name()), theme::LOOT));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::world::encounter::Encounter;

    #[test]
    fn shows_the_scene_and_what_changed() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        let e = Encounter::named("friendly_stranger");
        g.last_encounter = Some(EncounterReport {
            encounter: e,
            text: "A woman with a three-legged dog waves you over.".into(),
            resist: None,
            resist_text: None,
            changes: e.def().needs.clone(),
            gifts: e.def().gives.clone(),
        });
        let mut canvas = Canvas::new(100, 30);
        draw(&mut canvas, &g);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains("🤝 Friendly Stranger")));
        assert!(rows.iter().any(|r| r.contains("three-legged dog")), "the rolled scene is shown");
        assert!(rows.iter().any(|r| r.contains("Hope +")));
        assert!(rows.iter().any(|r| r.contains("You receive")));
        assert!(rows.iter().any(|r| r.contains("Enter / Esc continue")));
    }
}
