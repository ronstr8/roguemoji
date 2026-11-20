//! The leaderboard: one line per finished run, ranked by how far it got.
//!
//! Scores live in `scores.jsonl` beside the saves. Each finished run appends
//! one short line, which the OS writes in one piece, so players finishing at
//! the same moment on a shared server can't clobber each other's scores.

use std::cmp::Ordering;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

use serde::{Deserialize, Serialize};

use super::Store;
use crate::game::{Game, Outcome};

const SCORES_FILE: &str = "scores.jsonl";

/// A finished run, as the leaderboard shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub name: String,
    pub background: String,
    /// Reached the Beacon.
    pub won: bool,
    /// The deepest phase reached.
    pub phase: u32,
    pub turns: u64,
    pub kills: u32,
    /// How it ended, e.g. "Killed by a Rat in Phase 2".
    pub end: String,
    /// The release that played it, in case balance changes need explaining.
    pub version: String,
}

impl Score {
    /// The score for `game`, once it's over.
    pub fn of(game: &Game) -> Option<Score> {
        let end = match game.outcome.as_ref()? {
            Outcome::Won => format!("Answered the Beacon in Phase {}", game.depth),
            Outcome::Died(cause) => cause.clone(),
        };
        let p = &game.player;
        Some(Score {
            name: p.name.clone(),
            background: p.background.to_string(),
            won: matches!(game.outcome, Some(Outcome::Won)),
            phase: game.depth,
            turns: game.turn,
            kills: p.kills,
            end,
            version: crate::VERSION.to_string(),
        })
    }

    /// Best first: a win beats any loss, then the deeper phase. Between two
    /// wins the faster one is better; between two deaths at the same phase,
    /// the one that held out longer. Kills don't count, so farming weak
    /// monsters never beats pushing on.
    pub fn rank(&self, other: &Score) -> Ordering {
        let turns = if self.won { self.turns.cmp(&other.turns) } else { other.turns.cmp(&self.turns) };
        other.won.cmp(&self.won).then(other.phase.cmp(&self.phase)).then(turns)
    }
}

impl Store {
    /// Add a finished run to the leaderboard.
    pub fn record(&self, score: &Score) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let mut line = serde_json::to_string(score).map_err(io::Error::other)?;
        line.push('\n');
        let mut file = OpenOptions::new().create(true).append(true).open(self.dir.join(SCORES_FILE))?;
        file.write_all(line.as_bytes())
    }

    /// Every recorded run, best first. Lines that don't parse are skipped,
    /// so one bad line can't hide the rest of the board.
    pub fn scores(&self) -> Vec<Score> {
        let text = fs::read_to_string(self.dir.join(SCORES_FILE)).unwrap_or_default();
        let mut scores: Vec<Score> = text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
        scores.sort_by(Score::rank);
        scores
    }

    /// Where `score` places on the board (1 is best), and how many runs it has.
    pub fn placing(&self, score: &Score) -> Option<(usize, usize)> {
        let scores = self.scores();
        scores.iter().position(|s| s == score).map(|i| (i + 1, scores.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(name: &str, won: bool, phase: u32, turns: u64) -> Score {
        Score {
            name: name.into(),
            background: "Drifter".into(),
            won,
            phase,
            turns,
            kills: 0,
            end: String::new(),
            version: "0".into(),
        }
    }

    fn names(scores: &[Score]) -> Vec<&str> {
        scores.iter().map(|s| s.name.as_str()).collect()
    }

    #[test]
    fn wins_then_depth_then_turns() {
        let mut board = vec![
            score("deep death", false, 5, 900),
            score("slow win", true, 6, 5000),
            score("shallow death", false, 2, 9000),
            score("fast win", true, 6, 3000),
            score("quick death", false, 5, 100),
        ];
        board.sort_by(Score::rank);
        assert_eq!(names(&board), ["fast win", "slow win", "deep death", "quick death", "shallow death"]);
    }

    #[test]
    fn runs_are_recorded_and_ranked() {
        let dir = std::env::temp_dir().join(format!("roguemoji-scores-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let store = Store::new(dir.clone(), true);
        assert!(store.scores().is_empty());
        store.record(&score("a", false, 1, 10)).unwrap();
        store.record(&score("b", false, 3, 10)).unwrap();
        fs::write(dir.join(SCORES_FILE), fs::read_to_string(dir.join(SCORES_FILE)).unwrap() + "not json\n").unwrap();
        store.record(&score("c", false, 2, 10)).unwrap();
        assert_eq!(names(&store.scores()), ["b", "c", "a"], "the bad line is skipped");
        assert_eq!(store.placing(&score("c", false, 2, 10)), Some((2, 3)));
        let _ = fs::remove_dir_all(&dir);
    }
}
