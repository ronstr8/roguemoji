//! Death and victory pop-up.

use super::{footer, paragraph, popup};
use crate::game::{Game, Outcome};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

pub fn draw(canvas: &mut Canvas, game: &Game) {
    let (title, text, color) = match &game.outcome {
        Some(Outcome::Won) => (
            "📡 The Beacon Answers",
            "Light pours up from the Beacon and somewhere far above, something turns to look. Whatever comes next, you were the one who called.".to_string(),
            theme::SPECIAL,
        ),
        Some(Outcome::Died(cause)) => ("💀 You Have Died", format!("{cause}."), theme::BAD),
        None => return,
    };
    let inner = popup(canvas, 64, 14, title);
    let p = &game.player;
    let bottom = inner.y + inner.h.saturating_sub(1);
    let mut y = paragraph(canvas, inner.x, inner.y + 1, inner.w, bottom, &text, color) + 1;
    for line in [
        format!("{} the {}", p.name, p.background),
        format!("Depth {} · {} kills · {} turns · {} 💎 unspent", game.depth, p.kills, game.turn, p.crystals),
    ] {
        y = paragraph(canvas, inner.x, y, inner.w, bottom, &line, theme::TEXT);
    }
    footer(canvas, inner, "Enter new game · Q quit");
}
