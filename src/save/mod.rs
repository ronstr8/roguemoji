//! Saved games: writing a run to disk and reading it back.
//!
//! A save is a bookmark, not a checkpoint. Loading a run removes its file,
//! and the game writes it again when you quit, lose the connection or reach
//! a new phase. Dying or winning removes it for good, so a death can't be
//! undone by reloading.
//!
//! On your own machine there's one save slot. On a shared server (`--shared`)
//! every player is the same system user, so each run gets a resume code (see
//! `code`) and its file is named after it.

pub mod code;
pub mod form;
pub mod scores;

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::game::Game;

/// Bumped when the save layout changes so that older saves can't be read.
const FORMAT: u32 = 1;

/// The slot name when there's only one: a single player on their own machine.
const LOCAL_SLOT: &str = "save";

/// Where saves live, and whether they're keyed by resume code.
pub struct Store {
    dir: PathBuf,
    shared: bool,
}

#[derive(Serialize)]
struct SaveOut<'a> {
    format: u32,
    version: &'a str,
    game: &'a Game,
}

/// Read first, so a save from another version gets a clear message rather
/// than whatever field happened to fail.
#[derive(Deserialize)]
struct Header {
    format: u32,
    version: String,
}

#[derive(Deserialize)]
struct SaveIn {
    game: Game,
}

impl Store {
    pub fn new(dir: PathBuf, shared: bool) -> Self {
        Self { dir, shared }
    }

    /// The per-user data folder: `%APPDATA%\roguemoji` on Windows, otherwise
    /// `$XDG_DATA_HOME/roguemoji` or `~/.local/share/roguemoji`.
    pub fn default_dir() -> Option<PathBuf> {
        let env = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let base = if cfg!(windows) {
            env("APPDATA")
        } else {
            env("XDG_DATA_HOME").or_else(|| env("HOME").map(|h| h.join(".local").join("share")))
        };
        base.map(|b| b.join("roguemoji"))
    }

    /// Whether runs are found by resume code rather than kept in one slot.
    pub fn is_shared(&self) -> bool {
        self.shared
    }

    /// The slot for a new run: a fresh resume code when shared.
    pub fn new_slot(&self) -> String {
        if self.shared { code::generate() } else { LOCAL_SLOT.to_string() }
    }

    /// The local slot, if a run is saved in it.
    pub fn saved_local(&self) -> Option<String> {
        (!self.shared && self.path(LOCAL_SLOT).exists()).then(|| LOCAL_SLOT.to_string())
    }

    fn path(&self, slot: &str) -> PathBuf {
        self.dir.join(format!("{slot}.json"))
    }

    /// Save `game` to `slot`, replacing what was there. The new file is
    /// written beside the old and renamed over it, so a crash mid-write
    /// leaves the old save intact.
    pub fn write(&self, slot: &str, game: &Game) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let out = SaveOut { format: FORMAT, version: crate::VERSION, game };
        let text = serde_json::to_string(&out).map_err(io::Error::other)?;
        let tmp = self.dir.join(format!("{slot}.json.tmp"));
        fs::write(&tmp, text)?;
        fs::rename(&tmp, self.path(slot))
    }

    /// Read the run in `slot` without touching the file.
    pub fn read(&self, slot: &str) -> Result<Game, String> {
        let text = fs::read_to_string(self.path(slot)).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => "No saved run goes by that code.".to_string(),
            _ => format!("The save couldn't be read: {e}."),
        })?;
        parse(&text)
    }

    /// Load the run in `slot` and remove its file: it's written again on
    /// quit, so a run can't be reloaded to undo what happened since.
    pub fn take(&self, slot: &str) -> Result<Game, String> {
        let game = self.read(slot)?;
        self.remove(slot);
        Ok(game)
    }

    /// Forget the run in `slot`, as when it ends. Already gone is fine.
    pub fn remove(&self, slot: &str) {
        let _ = fs::remove_file(self.path(slot));
    }
}

fn parse(text: &str) -> Result<Game, String> {
    let header: Header = serde_json::from_str(text).map_err(|_| "That file isn't a roguemoji save.".to_string())?;
    if header.format != FORMAT {
        return Err(format!(
            "That run was saved by roguemoji {}, and this version ({}) can't read it.",
            header.version,
            crate::VERSION
        ));
    }
    let save: SaveIn = serde_json::from_str(text)
        .map_err(|e| format!("That run (saved by roguemoji {}) can't be loaded: {e}.", header.version))?;
    Ok(save.game)
}

#[cfg(test)]
mod tests;
