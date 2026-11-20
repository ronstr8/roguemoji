//! roguemoji — a post-apocalyptic roguelike drawn entirely in Unicode.

mod app;
mod color;
mod combat;
mod content;
mod crafting;
mod creatures;
mod dice;
mod effects;
mod game;
mod geometry;
mod input;
mod items;
mod save;
mod stats;
mod ui;
mod world;

use std::io;
use std::path::PathBuf;

/// The release version, from Cargo.toml. See CHANGELOG.md.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> io::Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("roguemoji {VERSION}");
        return Ok(());
    }
    let after = |flag: &str| std::env::args().skip_while(|a| a != flag).nth(1);
    let seed = after("--seed").and_then(|s| s.parse().ok());
    // A shared server (every player the same system user) keys saves by
    // resume code; anywhere else there's one save slot.
    let shared = std::env::args().any(|a| a == "--shared");
    let store = after("--save-dir").map(PathBuf::from).or_else(save::Store::default_dir);
    let store = store.map(|dir| save::Store::new(dir, shared));
    // Validate content before taking over the terminal, so errors are readable.
    // Needs come first: other content names them.
    if let Err(e) = creatures::needs::init()
        .and_then(|_| items::components::init())
        .and_then(|_| items::consumables::init())
        .and_then(|_| items::melee::init())
        .and_then(|_| items::armor::init())
        .and_then(|_| items::packs::init())
        .and_then(|_| items::ballistics::init_cartridges())
        .and_then(|_| items::ballistics::init_ammo())
        .and_then(|_| items::guns::init())
        .and_then(|_| crafting::init())
        .and_then(|_| creatures::species::init())
        .and_then(|_| creatures::face::init())
        .and_then(|_| world::harvest::init())
        .and_then(|_| world::encounter::init())
    {
        eprintln!("roguemoji: bad content data:\n{e}");
        std::process::exit(1);
    }
    catch_hangups();
    ui::terminal::install_panic_hook();
    let mut term = ui::terminal::Terminal::enter()?;
    let result = app::App::new(seed, store).run(&mut term);
    drop(term);
    result
}

/// A closed terminal or a stop request sets `app::HANGUP` instead of killing
/// the game outright, so the run in progress is saved on the way out.
#[cfg(unix)]
fn catch_hangups() {
    for signal in [signal_hook::consts::SIGHUP, signal_hook::consts::SIGTERM] {
        if let Err(e) = signal_hook::flag::register(signal, std::sync::Arc::clone(&app::HANGUP)) {
            eprintln!("roguemoji: can't catch signal {signal} ({e}); a dropped connection will lose the run");
        }
    }
}

#[cfg(not(unix))]
fn catch_hangups() {}
