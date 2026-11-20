//! The leaderboard pop-up: the best finished runs, best first.

use super::{footer, popup};
use crate::save::scores::Score;
use crate::ui::canvas::Canvas;
use crate::ui::theme;

/// How many runs the board lists.
pub const SHOWN: usize = 15;

/// `mine` is the index of the run just finished, to highlight it.
pub fn draw(canvas: &mut Canvas, scores: &[Score], mine: Option<usize>) {
    let inner = popup(canvas, 76, SHOWN as u16 + 6, "🏆 Leaderboard");
    let mut y = inner.y + 1;
    if scores.is_empty() {
        canvas.text(inner.x, y, "No runs have ended yet. Be the first.", theme::DIM, theme::POPUP);
    } else {
        let header = format!(
            "{:>3}  {:<16}  {:<12}  {:<14}  {:>6}  {:>5}",
            "#", "Name", "Background", "Result", "Turns", "Kills"
        );
        canvas.text_max(inner.x, y, &header, inner.w, theme::TITLE, theme::POPUP);
        y += 1;
        for (i, s) in scores.iter().enumerate().take(SHOWN) {
            let color = if Some(i) == mine { theme::WARN } else { theme::TEXT };
            canvas.text_max(inner.x, y, &row(i + 1, s), inner.w, color, theme::POPUP);
            y += 1;
        }
    }
    footer(canvas, inner, "Esc close");
}

fn row(place: usize, s: &Score) -> String {
    let result = if s.won { "📡 Won".to_string() } else { format!("💀 Phase {}", s.phase) };
    // Emoji are two cells wide, so pad the result one less.
    format!("{place:>3}  {:<16}  {:<12}  {result:<13}  {:>6}  {:>5}", s.name, s.background, s.turns, s.kills)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(name: &str, won: bool) -> Score {
        let (phase, turns, kills) = (4, 1234, 17);
        Score {
            name: name.into(),
            background: "Scavenger".into(),
            won,
            phase,
            turns,
            kills,
            end: String::new(),
            version: "0".into(),
        }
    }

    #[test]
    fn lists_runs_with_their_results() {
        let mut canvas = Canvas::new(80, 24);
        draw(&mut canvas, &[score("Vega", true), score("Rook", false)], Some(1));
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        assert!(rows.iter().any(|r| r.contains("1  Vega") && r.contains("📡 Won") && r.contains("1234")));
        assert!(rows.iter().any(|r| r.contains("2  Rook") && r.contains("💀 Phase 4")));
    }

    #[test]
    fn an_empty_board_says_so() {
        let mut canvas = Canvas::new(80, 24);
        draw(&mut canvas, &[], None);
        assert!(canvas.text_rows().iter().any(|r| r.contains("No runs have ended yet")));
    }
}
