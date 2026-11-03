//! roguemoji — a post-apocalyptic roguelike drawn entirely in Unicode.

mod app;
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
mod stats;
mod ui;
mod world;

use std::io;

fn main() -> io::Result<()> {
    let seed = std::env::args().skip_while(|a| a != "--seed").nth(1).and_then(|s| s.parse().ok());
    // Validate content before taking over the terminal, so errors are readable.
    if let Err(e) = items::components::init()
        .and_then(|_| items::consumables::init())
        .and_then(|_| crafting::init())
        .and_then(|_| creatures::species::init())
    {
        eprintln!("roguemoji: bad content data:\n{e}");
        std::process::exit(1);
    }
    ui::terminal::install_panic_hook();
    let mut term = ui::terminal::Terminal::enter()?;
    let result = app::App::new(seed).run(&mut term);
    drop(term);
    result
}
