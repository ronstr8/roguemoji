//! Death and victory pop-up.

use super::{footer, paragraph, popup};
use crate::game::{Game, Outcome};
use crate::ui::canvas::Canvas;
use crate::ui::theme;

/// `placed` is where the run landed on the leaderboard, and out of how many.
pub fn draw(canvas: &mut Canvas, game: &Game, placed: Option<(usize, usize)>) {
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
        format!("Phase {} · {} kills · {} turns · {} 💎 unspent", game.depth, p.kills, game.turn, p.crystals),
    ] {
        y = paragraph(canvas, inner.x, y, inner.w, bottom, &line, theme::TEXT);
    }
    if let Some((place, of)) = placed {
        let line = format!("🏆 #{place} of {of} on the leaderboard");
        paragraph(canvas, inner.x, y + 1, inner.w, bottom, &line, theme::WARN);
    }
    let hint = if placed.is_some() { "Enter new game · L leaderboard · Q quit" } else { "Enter new game · Q quit" };
    footer(canvas, inner, hint);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::game::Outcome;

    #[test]
    fn shows_where_the_run_placed() {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        g.outcome = Some(Outcome::Died("Killed by a Rat in Phase 1".into()));
        let mut canvas = Canvas::new(100, 30);
        draw(&mut canvas, &g, Some((4, 37)));
        let rows = canvas.text_rows();
        assert!(rows.iter().any(|r| r.contains("#4 of 37 on the leaderboard")));
        assert!(rows.iter().any(|r| r.contains("L leaderboard")));
    }
}
