//! Pop-ups for saved runs: typing a resume code, the code shown on quitting,
//! and a plain notice when a save can't be loaded.

use super::{footer, paragraph, popup};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

/// Typing a resume code on a shared server.
pub fn draw_entry(canvas: &mut Canvas, typed: &str, error: Option<&str>) {
    let inner = popup(canvas, 60, 12, "Resume a run");
    let bottom = inner.y + inner.h.saturating_sub(1);
    let text = "Enter the resume code you were given, like ember-otter-mesa-4127.";
    let mut y = paragraph(canvas, inner.x, inner.y + 1, inner.w, bottom, text, theme::TEXT) + 1;
    canvas.text_max(inner.x, y, &format!("> {typed}▏"), inner.w, theme::TITLE, theme::POPUP);
    y += 2;
    if let Some(e) = error {
        paragraph(canvas, inner.x, y, inner.w, bottom, e, theme::BAD);
    }
    footer(canvas, inner, "Enter resume · Esc back");
}

/// The run was saved on a shared server: here's how to get it back.
pub fn draw_saved(canvas: &mut Canvas, code: &str) {
    let inner = popup(canvas, 60, 12, "💾 Run saved");
    let bottom = inner.y + inner.h.saturating_sub(1);
    let text = "To pick it up again, choose R on the title screen and enter this code:";
    let mut y = paragraph(canvas, inner.x, inner.y + 1, inner.w, bottom, text, theme::TEXT) + 1;
    canvas.text_max(inner.x + 2, y, code, inner.w, theme::TITLE, theme::POPUP);
    y += 2;
    let note = "Write it down: it's the only way back to this run.";
    paragraph(canvas, inner.x, y, inner.w, bottom, note, theme::WARN);
    footer(canvas, inner, "Any key quits");
}

/// Something went wrong with a save; any key goes back.
pub fn draw_notice(canvas: &mut Canvas, text: &str) {
    let inner = popup(canvas, 60, 10, "Saved run");
    let bottom = inner.y + inner.h.saturating_sub(1);
    paragraph(canvas, inner.x, inner.y + 1, inner.w, bottom, text, theme::BAD);
    footer(canvas, inner, "Any key goes back");
}

/// Starting over would replace the run in the one local save slot.
pub fn draw_confirm_new(canvas: &mut Canvas) {
    let inner = popup(canvas, 56, 8, "Start a new run?");
    let bottom = inner.y + inner.h.saturating_sub(1);
    let text = "Your saved run will be lost. Press C on the title screen to continue it instead.";
    paragraph(canvas, inner.x, inner.y + 1, inner.w, bottom, text, theme::WARN);
    footer(canvas, inner, "y start over · any other key goes back");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_shows_what_was_typed_and_why_it_failed() {
        let mut canvas = Canvas::new(80, 24);
        draw_entry(&mut canvas, "ember-ott", Some("No saved run goes by that code."));
        let rows = canvas.text_rows();
        assert!(rows.iter().any(|r| r.contains("> ember-ott")));
        assert!(rows.iter().any(|r| r.contains("No saved run goes by that code.")));
    }

    #[test]
    fn saved_shows_the_code() {
        let mut canvas = Canvas::new(80, 24);
        draw_saved(&mut canvas, "ember-otter-mesa-4127");
        assert!(canvas.text_rows().iter().any(|r| r.contains("ember-otter-mesa-4127")));
    }
}
