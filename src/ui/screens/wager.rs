//! Wager pop-up: the scene, the stake and odds, and how the last bet went.

use crossterm::style::Color;

use super::{footer, paragraph, popup};
use crate::game::{BetReport, Game, Table};
use crate::ui::canvas::{Canvas, wrap};
use crate::ui::theme;
use crate::world::wager::Verdict;

const WIDTH: u16 = 66;

pub fn draw(canvas: &mut Canvas, game: &Game) {
    let Some(table) = &game.table else { return };
    let def = table.encounter.def();
    let blocker = game.bet_blocker();
    let mut lines = vec![(terms(game, table), theme::DIM)];
    if let Some(bet) = &table.last {
        lines.extend(result_lines(bet));
    }
    if let Some(why) = &blocker {
        lines.push((why.clone(), theme::BAD));
    }
    let body_w = (WIDTH - 4) as usize;
    let text_rows = |t: &str| wrap(t, body_w).len() as u16;
    let height = text_rows(&table.text) + lines.iter().map(|(l, _)| text_rows(l)).sum::<u16>() + 6;
    let inner = popup(canvas, WIDTH, height, &format!("{} {}", def.glyph, def.name));
    let bottom = inner.y + inner.h.saturating_sub(1);
    let mut y = paragraph(canvas, inner.x, inner.y, inner.w, bottom, &table.text, theme::TEXT) + 1;
    for (line, color) in lines {
        // An empty line is a gap; an empty paragraph would take no rows.
        y = if line.is_empty() { y + 1 } else { paragraph(canvas, inner.x, y, inner.w, bottom, &line, color) };
    }
    footer(canvas, inner, &keys(table, blocker.is_none()));
}

/// The stake, the odds, what you carry and (at a den) what the house holds.
fn terms(game: &Game, table: &Table) -> String {
    let w = table.wager();
    let odds = match w.attribute {
        Some(a) => format!("{} vs {}", a.name(), w.target),
        None => format!("Luck vs {}", w.target),
    };
    let mut parts = vec![format!("Stake {} 💎", w.stake), odds, format!("You carry {} 💎", game.player.crystals)];
    if w.purse.is_some() {
        parts.push(format!("House {} 💎", game.house_purse(table.pos)));
    }
    parts.join(" · ")
}

/// The roll, the result's text, and what it did to you.
fn result_lines(bet: &BetReport) -> Vec<(String, Color)> {
    let (money, color) = match bet.verdict {
        Verdict::Push => ("Your stake comes back.".to_string(), theme::TEXT),
        _ if bet.net < 0 => (format!("You lose {} 💎.", -bet.net), theme::BAD),
        _ => (format!("You win {} 💎.", bet.net), theme::GOOD),
    };
    let mut lines = vec![
        (String::new(), theme::TEXT),
        (format!("{} {}", bet.check.summary(), bet.verdict.name()), color),
        (bet.text.clone(), theme::TEXT),
        (money, color),
    ];
    for (need, delta) in &bet.changes {
        let c = if *delta < 0 { theme::BAD } else { theme::GOOD };
        lines.push((format!("{} {delta:+}", need.def().name), c));
    }
    lines
}

fn keys(table: &Table, can_bet: bool) -> String {
    let stake = table.wager().stake;
    match (can_bet, table.bets) {
        (true, 0) => format!("Enter bet {stake} 💎 · Esc walk away"),
        (true, _) => format!("Enter bet {stake} 💎 again · Esc leave"),
        (false, _) => "Enter / Esc leave".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creatures::Background;
    use crate::world::encounter::Encounter;
    use crate::world::feature::Feature;

    fn at_table(id: &str, crystals: u32) -> Game {
        let mut g = Game::new(Background::ALL[0].create("Tester".into()), Some(1));
        g.player.crystals = crystals;
        let pos = g.player.pos;
        let e = Encounter::named(id);
        g.map.set_feature(pos, Feature::encounter(e));
        g.table = Some(Table {
            pos,
            encounter: e,
            text: "The scene.".into(),
            last: None,
            bets: 0,
            picks: Default::default(),
        });
        g
    }

    fn render(g: &Game) -> Vec<String> {
        let mut canvas = Canvas::new(100, 30);
        draw(&mut canvas, g);
        let rows = canvas.text_rows();
        println!("{}", rows.join("\n"));
        rows
    }

    #[test]
    fn offers_the_bet_then_shows_the_result() {
        let mut g = at_table("dice_den", 20);
        let rows = render(&g);
        assert!(rows.iter().any(|r| r.contains("The scene.")));
        assert!(rows.iter().any(|r| r.contains("Stake") && r.contains("House")));
        assert!(rows.iter().any(|r| r.contains("Esc walk away")));
        assert!(g.place_bet());
        let rows = render(&g);
        assert!(rows.iter().any(|r| r.contains("You win") || r.contains("You lose") || r.contains("stake comes")));
        assert!(rows.iter().any(|r| r.contains("again") || r.contains("leave")));
    }

    #[test]
    fn says_why_you_cant_bet() {
        let rows = render(&at_table("dice_den", 0));
        assert!(rows.iter().any(|r| r.contains("You need")));
        assert!(rows.iter().any(|r| r.contains("Enter / Esc leave")));
    }
}
