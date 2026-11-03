//! Application shell: screen state machine, input dispatch and drawing.

use std::io;

use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEvent, KeyEventKind};

use crate::crafting::recipes;
use crate::creatures::Background;
use crate::dice::Dice;
use crate::game::{Event, Game, StashSide};
use crate::input::{self, Command};
use crate::stats::check::success_chance;
use crate::ui::canvas::Canvas;
use crate::ui::screens;
use crate::ui::terminal::Terminal;
use crate::ui::{self, MIN_HEIGHT, MIN_WIDTH};

const NAMES: &[&str] = &["Ash", "Rook", "Juniper", "Vega", "Sable", "Cass", "Ozzie", "Wren", "Dell", "Marlow"];

enum Screen {
    Title { selected: usize },
    Play,
    Inventory { selected: usize },
    Character,
    Crafting { selected: usize },
    Shop { kiosk: usize, selling: bool, selected: usize },
    Stash { side: StashSide, selected: usize },
    Targeting { targets: Vec<u32>, index: usize },
    Help,
    ConfirmQuit,
    Ended,
}

pub struct App {
    game: Option<Game>,
    screen: Screen,
    seed: Option<u64>,
    last_target: Option<u32>,
    inv_sort: screens::inventory::Sort,
    running: bool,
}

impl App {
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            game: None,
            screen: Screen::Title { selected: 0 },
            seed,
            last_target: None,
            inv_sort: Default::default(),
            running: true,
        }
    }

    pub fn run(mut self, term: &mut Terminal) -> io::Result<()> {
        while self.running {
            let (w, h) = Terminal::size();
            let mut canvas = Canvas::new(w, h);
            self.draw(&mut canvas);
            term.present(&canvas)?;
            match event::read()? {
                TermEvent::Key(key) if key.kind != KeyEventKind::Release => self.on_key(key),
                TermEvent::Resize(..) => term.invalidate(),
                _ => {}
            }
        }
        Ok(())
    }

    // ── Drawing ───────────────────────────────────────────────────────────

    fn draw(&self, canvas: &mut Canvas) {
        if canvas.width() < MIN_WIDTH || canvas.height() < MIN_HEIGHT {
            ui::draw_too_small(canvas);
            return;
        }
        let Some(game) = &self.game else {
            if let Screen::Title { selected } = self.screen {
                screens::title::draw(canvas, selected);
            }
            return;
        };
        if let Screen::Title { selected } = self.screen {
            screens::title::draw(canvas, selected);
            return;
        }

        let (target, status) = match &self.screen {
            Screen::Targeting { targets, index } => self.targeting_status(game, targets[*index]),
            Screen::ConfirmQuit => (None, Some("Really quit? This run will be lost. (y/n)".to_string())),
            _ => (None, None),
        };
        ui::draw_play(canvas, game, target, status.as_deref());

        match &self.screen {
            Screen::Inventory { selected } => screens::inventory::draw(canvas, game, self.inv_sort, *selected),
            Screen::Character => screens::character::draw(canvas, game),
            Screen::Crafting { selected } => screens::crafting::draw(canvas, game, *selected),
            Screen::Shop { kiosk, selling, selected } => {
                screens::shop::draw(canvas, game, *kiosk, *selling, self.inv_sort, *selected)
            }
            Screen::Stash { side, selected } => screens::stash::draw(canvas, game, *side, *selected),
            Screen::Help => screens::help::draw(canvas),
            Screen::Ended => screens::ended::draw(canvas, game),
            _ => {}
        }
    }

    fn targeting_status(&self, game: &Game, id: u32) -> (Option<crate::geometry::Pos>, Option<String>) {
        let Some(m) = game.monster(id) else { return (None, None) };
        let dist = m.pos.distance(game.player.pos);
        let odds = game
            .shot_odds(id)
            .map(|(modifier, def)| {
                format!(" · {:+} vs DEF {} · {:.0}% to hit", modifier, def, success_chance(modifier, def) * 100.0)
            })
            .unwrap_or_default();
        let text = format!(
            "Target: {} {} ({}) · {} tiles{}   [Tab] next  [f] fire  [Esc] cancel",
            m.def().glyph,
            m.name(),
            m.health_word(),
            dist,
            odds
        );
        (Some(m.pos), Some(text))
    }

    // ── Input ─────────────────────────────────────────────────────────────

    fn on_key(&mut self, key: KeyEvent) {
        let screen = std::mem::replace(&mut self.screen, Screen::Play);
        self.screen = match screen {
            Screen::Title { selected } => self.on_title(key, selected),
            Screen::Play => self.on_play(key),
            Screen::Inventory { selected } => self.on_inventory(key, selected),
            Screen::Character | Screen::Help => {
                if input::is_cancel(key) || input::is_confirm(key) {
                    Screen::Play
                } else {
                    screen_restore(&screen)
                }
            }
            Screen::Crafting { selected } => self.on_crafting(key, selected),
            Screen::Shop { kiosk, selling, selected } => self.on_shop(key, kiosk, selling, selected),
            Screen::Stash { side, selected } => self.on_stash(key, side, selected),
            Screen::Targeting { targets, index } => self.on_targeting(key, targets, index),
            Screen::ConfirmQuit => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.running = false;
                    Screen::Play
                }
                _ => Screen::Play,
            },
            Screen::Ended => match key.code {
                KeyCode::Enter => {
                    self.game = None;
                    Screen::Title { selected: 0 }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.running = false;
                    Screen::Ended
                }
                _ => Screen::Ended,
            },
        };
        // Any action can end the game.
        if self.game.as_ref().is_some_and(|g| g.is_over()) && !matches!(self.screen, Screen::Title { .. }) {
            self.screen = Screen::Ended;
        }
    }

    fn on_title(&mut self, key: KeyEvent, selected: usize) -> Screen {
        let n = Background::ALL.len();
        match key.code {
            KeyCode::Char('Q') | KeyCode::Char('q') | KeyCode::Esc => {
                self.running = false;
                Screen::Title { selected }
            }
            KeyCode::Char(c @ '1'..='4') => Screen::Title { selected: (c as usize - '1' as usize).min(n - 1) },
            KeyCode::Enter | KeyCode::Char(' ') => {
                let mut dice = Dice::from_entropy();
                let name = dice.pick(NAMES).to_string();
                let player = Background::ALL[selected].create(name);
                self.game = Some(Game::new(player, self.seed));
                self.last_target = None;
                Screen::Play
            }
            _ => Screen::Title { selected: input::list_nav(key, selected, n) },
        }
    }

    fn on_play(&mut self, key: KeyEvent) -> Screen {
        let Some(cmd) = input::play_command(key) else { return Screen::Play };
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        match cmd {
            Command::Move(dir) => match game.move_player(dir) {
                Event::OpenShop(kiosk) => return Screen::Shop { kiosk, selling: false, selected: 0 },
                Event::OpenStash => return Screen::Stash { side: StashSide::Pack, selected: 0 },
                Event::None => {}
            },
            Command::Wait => game.wait(),
            Command::Fire => return self.begin_targeting(),
            Command::Reload => game.reload(),
            Command::Pickup => game.pickup(),
            Command::Absorb { to_full } => game.absorb(to_full),
            Command::Descend => game.descend(),
            Command::Stance => game.cycle_stance(),
            Command::Inventory => return Screen::Inventory { selected: 0 },
            Command::Character => return Screen::Character,
            Command::Crafting => return Screen::Crafting { selected: 0 },
            Command::Help => return Screen::Help,
            Command::Quit => return Screen::ConfirmQuit,
        }
        Screen::Play
    }

    fn begin_targeting(&mut self) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        if game.player.gun.is_none() {
            game.log.info("You have no gun equipped.");
            return Screen::Play;
        }
        let targets = game.targets();
        if targets.is_empty() {
            game.log.info("No targets in sight.");
            return Screen::Play;
        }
        let index = self.last_target.and_then(|id| targets.iter().position(|t| *t == id)).unwrap_or(0);
        Screen::Targeting { targets, index }
    }

    fn on_targeting(&mut self, key: KeyEvent, targets: Vec<u32>, index: usize) -> Screen {
        let n = targets.len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Screen::Play,
            KeyCode::Tab | KeyCode::Right | KeyCode::Down | KeyCode::Char('+') => {
                Screen::Targeting { targets, index: (index + 1) % n }
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Up | KeyCode::Char('-') => {
                Screen::Targeting { targets, index: (index + n - 1) % n }
            }
            KeyCode::Char('f') | KeyCode::Enter | KeyCode::Char(' ') => {
                let id = targets[index];
                self.last_target = Some(id);
                if let Some(game) = self.game.as_mut() {
                    game.fire_at(id);
                }
                Screen::Play
            }
            _ => Screen::Targeting { targets, index },
        }
    }

    fn on_inventory(&mut self, key: KeyEvent, selected: usize) -> Screen {
        let sort = self.inv_sort;
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let len = game.player.inventory.len();
        let clamp = |s: usize, len: usize| s.min(len.saturating_sub(1));
        // `selected` is a row on screen; the pack itself stays in pickup order.
        let item = sort.order(game.player.inventory.items(), game.depth).get(selected).copied();
        match key.code {
            KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q') => return Screen::Play,
            KeyCode::Enter | KeyCode::Char('e') => {
                if let Some(idx) = item {
                    game.use_item(idx);
                }
            }
            KeyCode::Char('d') => {
                if let Some(idx) = item {
                    game.drop_item(idx);
                }
            }
            KeyCode::Char('s') => {
                let selected = self.inv_sort.advance(game.player.inventory.items(), game.depth, selected);
                return Screen::Inventory { selected };
            }
            KeyCode::Char('w') => game.unequip_melee(),
            KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                return Screen::Inventory { selected: input::list_nav(key, selected, len) };
            }
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                let idx = (c as u8 - b'a') as usize;
                if idx < len {
                    return Screen::Inventory { selected: idx };
                }
            }
            _ => {}
        }
        if game.is_over() {
            return Screen::Ended;
        }
        Screen::Inventory { selected: clamp(selected, game.player.inventory.len()) }
    }

    fn on_crafting(&mut self, key: KeyEvent, selected: usize) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        if input::is_cancel(key) || key.code == KeyCode::Char('m') {
            return Screen::Play;
        }
        if key.code == KeyCode::Enter {
            game.craft(selected);
            return Screen::Crafting { selected };
        }
        Screen::Crafting { selected: input::list_nav(key, selected, recipes().len()) }
    }

    fn on_shop(&mut self, key: KeyEvent, kiosk: usize, selling: bool, selected: usize) -> Screen {
        let sort = self.inv_sort;
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        if input::is_cancel(key) {
            return Screen::Play;
        }
        let len_of = |game: &Game, selling: bool| {
            if selling { game.player.inventory.len() } else { game.kiosks.get(kiosk).map_or(0, |k| k.stock.len()) }
        };
        match key.code {
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
                let selected = selected.min(len_of(game, !selling).saturating_sub(1));
                Screen::Shop { kiosk, selling: !selling, selected }
            }
            KeyCode::Enter => {
                if selling {
                    // Rows follow the inventory sort; the pack keeps pickup order.
                    if let Some(idx) = sort.order(game.player.inventory.items(), game.depth).get(selected) {
                        game.sell(*idx);
                    }
                } else {
                    game.buy(kiosk, selected);
                }
                let selected = selected.min(len_of(game, selling).saturating_sub(1));
                Screen::Shop { kiosk, selling, selected }
            }
            KeyCode::Char('s') if selling => {
                let selected = self.inv_sort.advance(game.player.inventory.items(), game.depth, selected);
                Screen::Shop { kiosk, selling, selected }
            }
            _ => Screen::Shop { kiosk, selling, selected: input::list_nav(key, selected, len_of(game, selling)) },
        }
    }

    fn on_stash(&mut self, key: KeyEvent, side: StashSide, selected: usize) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        if input::is_cancel(key) {
            return Screen::Play;
        }
        let len_of = |game: &Game, side: StashSide| match side {
            StashSide::Pack => game.player.inventory.len(),
            StashSide::Lockbox => game.stash.len(),
        };
        match key.code {
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Left | KeyCode::Right => {
                let other = match side {
                    StashSide::Pack => StashSide::Lockbox,
                    StashSide::Lockbox => StashSide::Pack,
                };
                let selected = selected.min(len_of(game, other).saturating_sub(1));
                Screen::Stash { side: other, selected }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if selected < len_of(game, side) {
                    game.stash_transfer(side, selected);
                }
                if game.is_over() {
                    return Screen::Ended;
                }
                Screen::Stash { side, selected: selected.min(len_of(game, side).saturating_sub(1)) }
            }
            _ => Screen::Stash { side, selected: input::list_nav(key, selected, len_of(game, side)) },
        }
    }
}

fn screen_restore(screen: &Screen) -> Screen {
    match screen {
        Screen::Character => Screen::Character,
        _ => Screen::Help,
    }
}
