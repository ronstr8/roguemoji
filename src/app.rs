//! Application shell: screen state machine, input dispatch and drawing.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEvent, KeyEventKind};

use crate::crafting::recipes;
use crate::creatures::Background;
use crate::creatures::player;
use crate::creatures::player::Slot;
use crate::dice::Dice;
use crate::game::{Event, Game, StashSide, Tone};
use crate::geometry::Pos;
use crate::input::{self, Command};
use crate::save::scores::Score;
use crate::save::{self, Store};
use crate::stats::check::success_chance;
use crate::ui::canvas::Canvas;
use crate::ui::screens;
use crate::ui::terminal::Terminal;
use crate::ui::theme;
use crate::ui::{self, MIN_HEIGHT, MIN_WIDTH};

/// How long each turn of sleep shows on screen.
const SLEEP_TICK: Duration = Duration::from_millis(40);

/// How long the screen stays white after stepping through a portal.
const WHITEOUT: Duration = Duration::from_millis(500);

/// One full breath of an encounter's glow, bright to bright.
const PULSE: Duration = Duration::from_millis(2400);
/// How often the glow is redrawn while an encounter is in sight.
const PULSE_FRAME: Duration = Duration::from_millis(100);
/// The glow never fades below this, so encounters always stand out.
const GLOW_FLOOR: f32 = 0.3;

/// Longest wait for input before checking whether the player has gone.
const IDLE_WAIT: Duration = Duration::from_millis(500);

/// Set by a signal handler when the terminal hangs up or the server asks the
/// game to stop, so the run is saved on the way out instead of lost.
pub static HANGUP: LazyLock<Arc<AtomicBool>> = LazyLock::new(Default::default);

const NAMES: &[&str] = &["Ash", "Rook", "Juniper", "Vega", "Sable", "Cass", "Ozzie", "Wren", "Dell", "Marlow"];

enum Screen {
    Title {
        selected: usize,
    },
    Play,
    Inventory {
        selected: usize,
    },
    Character,
    Crafting {
        selected: usize,
    },
    Shop {
        kiosk: usize,
        selling: bool,
        selected: usize,
    },
    Stash {
        side: StashSide,
        selected: usize,
    },
    Targeting {
        targets: Vec<u32>,
        index: usize,
    },
    Shoot {
        targets: Vec<u32>,
        selected: usize,
    },
    /// The first help line shown.
    Help {
        scroll: usize,
    },
    Encounter,
    /// Sitting at a wager; the table is in `Game::table`.
    Wager,
    /// Choosing how to open the harvest node or cache at `pos`.
    OpenWith {
        pos: Pos,
        selected: usize,
    },
    ConfirmQuit,
    Ended,
    /// Typing a resume code on a shared server.
    Resume {
        typed: String,
        error: Option<String>,
    },
    /// The run is saved on a shared server; shows its code before quitting.
    Saved {
        code: String,
    },
    /// A saved run couldn't be loaded; any key goes back to the title.
    Notice(String),
    /// Starting a new run as background `selected` would replace the saved one.
    ConfirmNew {
        selected: usize,
    },
    /// Naming the new run's character, background `selected`. Left empty,
    /// `suggestion` is used.
    Name {
        selected: usize,
        typed: String,
        suggestion: String,
    },
    /// The leaderboard, loaded when opened. `mine` highlights the run just
    /// finished; `from_end` says whether Esc goes back to the end screen.
    Scores {
        list: Vec<Score>,
        mine: Option<usize>,
        from_end: bool,
    },
}

pub struct App {
    game: Option<Game>,
    screen: Screen,
    seed: Option<u64>,
    last_target: Option<u32>,
    /// A 💥 drawn on a creature a shot struck, until the deadline passes.
    flash: Option<(Pos, Instant)>,
    /// The screen is blanked white until this moment, after a portal jump.
    whiteout: Option<Instant>,
    inv_sort: screens::inventory::Sort,
    /// When the app started; the glow's pulse is timed from here.
    started: Instant,
    running: bool,
    /// Where runs are saved; `None` if there's nowhere to put them.
    store: Option<Store>,
    /// The save slot of the run in progress: its resume code when shared.
    slot: Option<String>,
    /// Where the run just finished placed on the leaderboard, and out of how many.
    placed: Option<(usize, usize)>,
    /// The name chosen last, offered again for the next run.
    last_name: Option<String>,
}

impl App {
    pub fn new(seed: Option<u64>, store: Option<Store>) -> Self {
        Self {
            game: None,
            screen: Screen::Title { selected: 0 },
            seed,
            last_target: None,
            flash: None,
            whiteout: None,
            inv_sort: Default::default(),
            started: Instant::now(),
            running: true,
            store,
            slot: None,
            placed: None,
            last_name: None,
        }
    }

    pub fn run(mut self, term: &mut Terminal) -> io::Result<()> {
        let result = self.event_loop(term);
        // However the loop ended (a quit, a dropped connection, a stop
        // signal), a run still in progress is kept.
        self.save_run();
        result
    }

    fn event_loop(&mut self, term: &mut Terminal) -> io::Result<()> {
        while self.running && !HANGUP.load(Ordering::Relaxed) {
            let (w, h) = Terminal::size();
            let mut canvas = Canvas::new(w, h);
            self.draw(&mut canvas);
            term.present(&canvas)?;
            // Asleep, turns pass on their own; a key press interrupts.
            if self.sleeping() && !event::poll(SLEEP_TICK)? {
                self.sleep_tick();
                continue;
            }
            // While a flash, whiteout or glow is up, wait only until the next
            // one is due, then redraw.
            if let Some(until) = self.next_redraw() {
                let left = until.saturating_duration_since(Instant::now());
                if left.is_zero() || !event::poll(left)? {
                    self.expire_effects();
                    continue;
                }
            }
            // Wake now and then to notice a hangup.
            if !event::poll(IDLE_WAIT)? {
                continue;
            }
            match event::read()? {
                TermEvent::Key(key) if key.kind != KeyEventKind::Release => self.on_key(key),
                TermEvent::Resize(..) => term.invalidate(),
                _ => {}
            }
        }
        Ok(())
    }

    /// The soonest moment a timed effect needs the screen redrawn, if any.
    fn next_redraw(&self) -> Option<Instant> {
        let pulse = self.pulsing().then(|| Instant::now() + PULSE_FRAME);
        self.flash.map(|(_, t)| t).into_iter().chain(self.whiteout).chain(pulse).min()
    }

    /// Whether an encounter glow or a flashing face is on screen and
    /// uncovered, so worth animating. Pop-ups dim the map, which hides both.
    fn pulsing(&self) -> bool {
        let map_showing = matches!(self.screen, Screen::Play | Screen::Targeting { .. } | Screen::ConfirmQuit);
        let animated = |g: &Game| g.map.sees_encounter() || g.player.face().flash.is_some();
        map_showing && self.game.as_ref().is_some_and(|g| !g.is_over() && animated(g))
    }

    /// The glow's strength now: a slow breath between `GLOW_FLOOR` and full.
    fn glow(&self) -> f32 {
        let phase = self.started.elapsed().as_secs_f32() / PULSE.as_secs_f32();
        let breath = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
        GLOW_FLOOR + (1.0 - GLOW_FLOOR) * breath
    }

    /// Drop the timed screen effects whose moment has passed.
    fn expire_effects(&mut self) {
        let now = Instant::now();
        self.flash = self.flash.filter(|(_, t)| *t > now);
        self.whiteout = self.whiteout.filter(|t| *t > now);
    }

    /// Whether the player is asleep on the map, so turns should pass by themselves.
    fn sleeping(&self) -> bool {
        matches!(self.screen, Screen::Play) && self.game.as_ref().is_some_and(|g| g.player.is_asleep() && !g.is_over())
    }

    fn sleep_tick(&mut self) {
        let Some(game) = self.game.as_mut() else { return };
        game.sleep_turn();
        if game.is_over() {
            self.screen = Screen::Ended;
            self.finish_run();
        }
    }

    // ── Drawing ───────────────────────────────────────────────────────────

    fn draw(&self, canvas: &mut Canvas) {
        if self.whiteout.is_some() {
            canvas.fill(0, 0, canvas.width(), canvas.height(), theme::WHITEOUT);
            return;
        }
        if canvas.width() < MIN_WIDTH || canvas.height() < MIN_HEIGHT {
            ui::draw_too_small(canvas);
            return;
        }
        let note = self.title_note();
        let Some(game) = &self.game else {
            let selected = match self.screen {
                Screen::Title { selected } | Screen::ConfirmNew { selected } | Screen::Name { selected, .. } => {
                    selected
                }
                _ => 0,
            };
            screens::title::draw(canvas, selected, note.as_deref(), self.store.is_some());
            match &self.screen {
                Screen::Resume { typed, error } => screens::saves::draw_entry(canvas, typed, error.as_deref()),
                Screen::Notice(text) => screens::saves::draw_notice(canvas, text),
                Screen::ConfirmNew { .. } => screens::saves::draw_confirm_new(canvas),
                Screen::Name { typed, suggestion, .. } => screens::saves::draw_name(canvas, typed, suggestion),
                Screen::Scores { list, mine, .. } => screens::scores::draw(canvas, list, *mine),
                _ => {}
            }
            return;
        };
        if let Screen::Title { selected } = self.screen {
            screens::title::draw(canvas, selected, note.as_deref(), self.store.is_some());
            return;
        }

        let (target, status) = match &self.screen {
            Screen::Targeting { targets, index } => self.targeting_status(game, targets[*index]),
            Screen::Shoot { targets, selected } => (game.monster(targets[*selected]).map(|m| m.pos), None),
            Screen::ConfirmQuit => (None, Some(self.quit_prompt().to_string())),
            _ => (None, None),
        };
        let clock_ms = self.started.elapsed().as_millis();
        let fx = ui::Effects { flash: self.flash.map(|(pos, _)| pos), glow: self.glow(), clock_ms };
        ui::draw_play(canvas, game, target, fx, status.as_deref());

        match &self.screen {
            Screen::Inventory { selected } => screens::inventory::draw(canvas, game, self.inv_sort, *selected),
            Screen::Character => screens::character::draw(canvas, game, self.shared_code()),
            Screen::Crafting { selected } => screens::crafting::draw(canvas, game, *selected),
            Screen::Shop { kiosk, selling, selected } => {
                screens::shop::draw(canvas, game, *kiosk, *selling, self.inv_sort, *selected)
            }
            Screen::Stash { side, selected } => screens::stash::draw(canvas, game, *side, self.inv_sort, *selected),
            Screen::Shoot { targets, selected } => screens::shoot::draw(canvas, game, targets, *selected),
            Screen::Help { scroll } => screens::help::draw(canvas, *scroll),
            Screen::Encounter => screens::encounter::draw(canvas, game),
            Screen::Wager => screens::wager::draw(canvas, game),
            Screen::OpenWith { pos, selected } => screens::open_with::draw(canvas, game, *pos, *selected),
            Screen::Ended => screens::ended::draw(canvas, game, self.placed),
            Screen::Saved { code } => screens::saves::draw_saved(canvas, code),
            Screen::Scores { list, mine, .. } => screens::scores::draw(canvas, list, *mine),
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
            Screen::Help { scroll } => self.on_help(key, scroll),
            Screen::Character | Screen::Encounter => {
                if input::is_cancel(key) || input::is_confirm(key) {
                    Screen::Play
                } else {
                    screen_restore(&screen)
                }
            }
            Screen::Wager => self.on_wager(key),
            Screen::OpenWith { pos, selected } => self.on_open_with(key, pos, selected),
            Screen::Crafting { selected } => self.on_crafting(key, selected),
            Screen::Shop { kiosk, selling, selected } => self.on_shop(key, kiosk, selling, selected),
            Screen::Stash { side, selected } => self.on_stash(key, side, selected),
            Screen::Targeting { targets, index } => self.on_targeting(key, targets, index),
            Screen::Shoot { targets, selected } => self.on_shoot(key, targets, selected),
            Screen::ConfirmQuit => self.on_confirm_quit(key),
            Screen::Resume { typed, error } => self.on_resume(key, typed, error),
            Screen::Saved { code } => {
                self.running = false;
                Screen::Saved { code }
            }
            Screen::Notice(_) => Screen::Title { selected: 0 },
            Screen::ConfirmNew { selected } => match key.code {
                KeyCode::Char('y' | 'Y') => self.ask_name(selected),
                _ => Screen::Title { selected },
            },
            Screen::Name { selected, typed, suggestion } => self.on_name(key, selected, typed, suggestion),
            Screen::Scores { list, mine, from_end } => {
                match (input::is_cancel(key) || input::is_confirm(key), from_end) {
                    (false, _) => Screen::Scores { list, mine, from_end },
                    (true, true) => Screen::Ended,
                    (true, false) => Screen::Title { selected: 0 },
                }
            }
            Screen::Ended => match key.code {
                KeyCode::Enter => {
                    self.game = None;
                    self.placed = None;
                    Screen::Title { selected: 0 }
                }
                KeyCode::Char('l' | 'L') if self.store.is_some() => self.scores_screen(true),
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.running = false;
                    Screen::Ended
                }
                _ => Screen::Ended,
            },
        };
        // Any action can end the game, or knock you out (back to the map to sleep it off).
        let browsing = matches!(self.screen, Screen::Title { .. } | Screen::Scores { .. });
        if self.game.as_ref().is_some_and(|g| g.is_over()) && !browsing {
            self.screen = Screen::Ended;
            self.finish_run();
        } else if self.game.as_ref().is_some_and(|g| g.player.is_asleep()) {
            self.screen = Screen::Play;
        }
    }

    /// Enter bets while you can, and otherwise leaves; Esc always leaves.
    fn on_wager(&mut self, key: KeyEvent) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let bet = input::is_confirm(key) && game.place_bet();
        if bet && (game.is_over() || game.player.is_asleep()) {
            // The turn spent betting ended you or knocked you out; you're up from the table.
            game.leave_table();
            Screen::Play
        } else if bet {
            Screen::Wager
        } else if input::is_cancel(key) || input::is_confirm(key) {
            game.leave_table();
            Screen::Play
        } else {
            Screen::Wager
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
                // A new run takes over the one save slot, so check first.
                if self.store.as_ref().and_then(Store::saved_local).is_some() {
                    return Screen::ConfirmNew { selected };
                }
                self.ask_name(selected)
            }
            KeyCode::Char('c' | 'C' | 'r' | 'R') => self.on_title_resume(selected),
            KeyCode::Char('l' | 'L') if self.store.is_some() => self.scores_screen(false),
            _ => Screen::Title { selected: input::list_nav(key, selected, n) },
        }
    }

    // ── Saved runs ────────────────────────────────────────────────────────

    /// Ask for a name for a new run as background `selected`, offering the
    /// last one chosen.
    fn ask_name(&self, selected: usize) -> Screen {
        let suggestion = Dice::from_entropy().pick(NAMES).to_string();
        Screen::Name { selected, typed: self.last_name.clone().unwrap_or_default(), suggestion }
    }

    fn on_name(&mut self, key: KeyEvent, selected: usize, mut typed: String, suggestion: String) -> Screen {
        match key.code {
            KeyCode::Esc => Screen::Title { selected },
            KeyCode::Enter => {
                let name = player::tidy_name(&typed).unwrap_or(suggestion);
                self.last_name = Some(name.clone());
                let player = Background::ALL[selected].create(name);
                self.begin_run(Game::new(player, self.seed))
            }
            _ => {
                input::edit_line(&mut typed, key, player::NAME_MAX, player::name_char);
                Screen::Name { selected, typed, suggestion }
            }
        }
    }

    /// The leaderboard, highlighting the run just finished if `from_end`.
    fn scores_screen(&self, from_end: bool) -> Screen {
        let list = self.store.as_ref().map(Store::scores).unwrap_or_default();
        let mine = self.placed.filter(|_| from_end).map(|(place, _)| place - 1);
        Screen::Scores { list, mine, from_end }
    }

    /// Start `game` as a new run, saved from the first turn so that its
    /// resume code works straight away.
    fn begin_run(&mut self, game: Game) -> Screen {
        self.slot = self.store.as_ref().map(Store::new_slot);
        self.game = Some(game);
        self.last_target = None;
        if let (Some(code), Some(game)) = (self.shared_code().map(str::to_string), self.game.as_mut()) {
            game.log.push(
                format!("Your resume code is {code}. Write it down: you'll need it to come back."),
                Tone::Special,
            );
        }
        self.save_run();
        Screen::Play
    }

    /// Load the run saved in `slot` and play on.
    fn resume_run(&mut self, slot: String) -> Result<(), String> {
        let store = self.store.as_ref().ok_or("There's nowhere to load saved runs from.")?;
        self.game = Some(store.take(&slot)?);
        self.slot = Some(slot);
        self.last_target = None;
        Ok(())
    }

    /// Write the run in progress to its slot. Does nothing between runs,
    /// once a run has ended, or with nowhere to save.
    fn save_run(&mut self) {
        let (Some(store), Some(slot), Some(game)) = (&self.store, &self.slot, self.game.as_mut()) else { return };
        if game.is_over() {
            return;
        }
        // A wager's pop-up isn't saved, so get up from the table first.
        if game.table.is_some() {
            game.leave_table();
        }
        if let Err(e) = store.write(slot, game) {
            game.log.push(format!("The run couldn't be saved: {e}."), Tone::Bad);
        }
    }

    /// The run has ended in death or victory: it goes on the leaderboard and
    /// its save goes. Keyed to the slot, so a run is only recorded once.
    fn finish_run(&mut self) {
        let (Some(store), Some(game)) = (&self.store, &self.game) else { return };
        if self.slot.is_none() {
            return;
        }
        if let Some(score) = Score::of(game) {
            self.placed = store.record(&score).ok().and_then(|_| store.placing(&score));
        }
        self.forget_run();
    }

    /// The run is over, by death, victory or abandoning it: its save goes.
    fn forget_run(&mut self) {
        if let (Some(store), Some(slot)) = (&self.store, self.slot.take()) {
            store.remove(&slot);
        }
    }

    /// The run's resume code, on a shared server.
    fn shared_code(&self) -> Option<&str> {
        self.slot.as_deref().filter(|_| self.store.as_ref().is_some_and(Store::is_shared))
    }

    /// The title screen's line about saved runs, if there's one to offer.
    fn title_note(&self) -> Option<String> {
        let store = self.store.as_ref()?;
        if store.is_shared() {
            return Some("R  resume a saved run with its code".into());
        }
        let slot = store.saved_local()?;
        Some(match store.read(&slot) {
            Ok(g) => format!("C  continue {} the {} · phase {}", g.player.name, g.player.background, g.depth),
            Err(_) => "C  continue your saved run".into(),
        })
    }

    fn quit_prompt(&self) -> &'static str {
        if self.slot.is_some() {
            "Save and quit? (y) save · (a) abandon this run · any other key plays on"
        } else {
            "Really quit? This run will be lost. (y/n)"
        }
    }

    fn on_confirm_quit(&mut self, key: KeyEvent) -> Screen {
        match key.code {
            KeyCode::Char('y' | 'Y') => {
                self.save_run();
                if let Some(code) = self.shared_code() {
                    return Screen::Saved { code: code.to_string() };
                }
                self.running = false;
                Screen::Play
            }
            KeyCode::Char('a' | 'A') if self.slot.is_some() => {
                self.forget_run();
                self.running = false;
                Screen::Play
            }
            _ => Screen::Play,
        }
    }

    /// C continues the local save; R asks for a resume code on a shared server.
    fn on_title_resume(&mut self, selected: usize) -> Screen {
        let Some(store) = &self.store else { return Screen::Title { selected } };
        if store.is_shared() {
            return Screen::Resume { typed: String::new(), error: None };
        }
        let Some(slot) = store.saved_local() else { return Screen::Title { selected } };
        match self.resume_run(slot) {
            Ok(()) => Screen::Play,
            Err(e) => Screen::Notice(e),
        }
    }

    fn on_resume(&mut self, key: KeyEvent, mut typed: String, error: Option<String>) -> Screen {
        const NOT_A_CODE: &str = "That isn't a resume code: three words and four digits, like ember-otter-mesa-4127.";
        match key.code {
            KeyCode::Esc => Screen::Title { selected: 0 },
            KeyCode::Enter => {
                let loaded =
                    save::code::normalize(&typed).ok_or(NOT_A_CODE.to_string()).and_then(|c| self.resume_run(c));
                match loaded {
                    Ok(()) => Screen::Play,
                    Err(e) => Screen::Resume { typed, error: Some(e) },
                }
            }
            _ => {
                let error = if input::edit_line(&mut typed, key, 40, |c| c.is_ascii_graphic() || c == ' ') {
                    None
                } else {
                    error
                };
                Screen::Resume { typed, error }
            }
        }
    }

    fn on_play(&mut self, key: KeyEvent) -> Screen {
        if let Some(game) = self.game.as_mut().filter(|g| g.player.is_asleep()) {
            game.rouse();
            return Screen::Play;
        }
        let Some(cmd) = input::play_command(key) else { return Screen::Play };
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        match cmd {
            Command::Move(dir) => match game.move_player(dir) {
                Event::OpenShop(kiosk) => return Screen::Shop { kiosk, selling: false, selected: 0 },
                Event::OpenStash => return Screen::Stash { side: StashSide::Pack, selected: 0 },
                Event::OpenEncounter => return Screen::Encounter,
                Event::OpenWager => return Screen::Wager,
                Event::ChooseWay(pos) => return Screen::OpenWith { pos, selected: first_ready(game, pos) },
                Event::Portal => {
                    self.whiteout = Some(Instant::now() + WHITEOUT);
                    // A new phase is a natural bookmark, in case the game is cut off.
                    self.save_run();
                }
                Event::None => {}
            },
            Command::Wait => game.wait(),
            Command::Fire => return self.begin_targeting(),
            Command::Shoot => return self.begin_shoot(),
            Command::Reload => game.reload(),
            Command::Pickup => game.pickup(),
            Command::Absorb { to_full } => game.absorb(to_full),
            Command::Drink => game.drink_here(),
            Command::Nap => game.nap(),
            Command::Stance => game.cycle_stance(),
            Command::Inventory => return Screen::Inventory { selected: screens::inventory::first_row(game) },
            Command::Character => return Screen::Character,
            Command::Crafting => return Screen::Crafting { selected: 0 },
            Command::Help => return Screen::Help { scroll: 0 },
            Command::Quit => return Screen::ConfirmQuit,
        }
        Screen::Play
    }

    fn begin_targeting(&mut self) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let targets = game.shot_targets();
        if targets.is_empty() {
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
                self.shoot(targets[index]);
                Screen::Play
            }
            _ => Screen::Targeting { targets, index },
        }
    }

    /// `s`: shoot the only creature in range, or ask which one.
    fn begin_shoot(&mut self) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let targets = game.shot_targets();
        match targets.as_slice() {
            [] => Screen::Play,
            [only] => {
                self.shoot(*only);
                Screen::Play
            }
            _ => {
                // Start on the last creature shot at, so momentum is one keypress away.
                let selected = self.last_target.and_then(|id| targets.iter().position(|t| *t == id)).unwrap_or(0);
                Screen::Shoot { targets, selected }
            }
        }
    }

    fn on_shoot(&mut self, key: KeyEvent, targets: Vec<u32>, selected: usize) -> Screen {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Screen::Play,
            KeyCode::Enter | KeyCode::Char('s') => {
                self.shoot(targets[selected]);
                Screen::Play
            }
            _ => Screen::Shoot { selected: input::list_nav(key, selected, targets.len()), targets },
        }
    }

    fn shoot(&mut self, id: u32) {
        const FLASH: Duration = Duration::from_millis(300);
        self.last_target = Some(id);
        if let Some(pos) = self.game.as_mut().and_then(|g| g.fire_at(id)) {
            self.flash = Some((pos, Instant::now() + FLASH));
        }
    }

    fn on_inventory(&mut self, key: KeyEvent, selected: usize) -> Screen {
        use screens::inventory::{Row, SLOT_ROWS, row_at, row_count};
        let sort = self.inv_sort;
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let row = row_at(game, sort, selected);
        match key.code {
            KeyCode::Esc | KeyCode::Char('i') | KeyCode::Char('q') => return Screen::Play,
            KeyCode::Enter | KeyCode::Char('e') => match row {
                Some(Row::Pack(idx)) => game.use_item(idx),
                Some(Row::Slot(slot)) => game.unequip(slot),
                None => {}
            },
            KeyCode::Char('d') => {
                if let Some(Row::Pack(idx)) = row {
                    game.drop_item(idx);
                }
            }
            KeyCode::Char('s') => {
                let items = game.player.inventory.items();
                let selected = match selected.checked_sub(SLOT_ROWS) {
                    Some(i) => SLOT_ROWS + self.inv_sort.advance(items, game.depth, i),
                    None => {
                        self.inv_sort = self.inv_sort.next();
                        selected
                    }
                };
                return Screen::Inventory { selected };
            }
            KeyCode::Char('w') => game.unequip(Slot::Melee),
            KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown => {
                return Screen::Inventory { selected: input::list_nav(key, selected, row_count(game)) };
            }
            KeyCode::Char(c) if c.is_ascii_lowercase() => {
                let idx = (c as u8 - b'a') as usize;
                if idx < game.player.inventory.len() {
                    return Screen::Inventory { selected: SLOT_ROWS + idx };
                }
            }
            _ => {}
        }
        if game.is_over() {
            return Screen::Ended;
        }
        Screen::Inventory { selected: selected.min(row_count(game) - 1) }
    }

    fn on_help(&mut self, key: KeyEvent, scroll: usize) -> Screen {
        if input::is_cancel(key) || input::is_confirm(key) || key.code == KeyCode::Char('?') {
            return Screen::Play;
        }
        let (_, height) = Terminal::size();
        Screen::Help { scroll: screens::help::scroll(key, scroll, screens::help::max_scroll(height)) }
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

    fn on_open_with(&mut self, key: KeyEvent, pos: Pos, selected: usize) -> Screen {
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        let ways = game.node_at(pos).map_or(0, |d| d.open.len());
        let chosen = match key.code {
            KeyCode::Char(c @ '1'..='9') => Some(c as usize - '1' as usize),
            _ if input::is_confirm(key) => Some(selected),
            _ => None,
        };
        // A way you lack the tool for keeps the pop-up open.
        let ready = |w: usize| game.node_at(pos).is_some_and(|d| d.ways_for(&game.player).contains(&w));
        if input::is_cancel(key) {
            Screen::Play
        } else if let Some(way) = chosen.filter(|&w| ready(w)) {
            game.harvest_at(pos, way);
            Screen::Play
        } else {
            Screen::OpenWith { pos, selected: input::list_nav(key, selected, ways) }
        }
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
        let sort = self.inv_sort;
        let Some(game) = self.game.as_mut() else { return Screen::Play };
        if input::is_cancel(key) {
            return Screen::Play;
        }
        fn items_of(game: &Game, side: StashSide) -> &[crate::items::Item] {
            match side {
                StashSide::Pack => game.player.inventory.items(),
                StashSide::Lockbox => game.stash.items(),
            }
        }
        let len_of = |game: &Game, side: StashSide| items_of(game, side).len();
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
                // Rows follow the sort; the pack and stash keep pickup order.
                if let Some(&idx) = sort.order(items_of(game, side), game.depth).get(selected) {
                    game.stash_transfer(side, idx);
                }
                if game.is_over() {
                    return Screen::Ended;
                }
                Screen::Stash { side, selected: selected.min(len_of(game, side).saturating_sub(1)) }
            }
            KeyCode::Char('s') => {
                let selected = self.inv_sort.advance(items_of(game, side), game.depth, selected);
                Screen::Stash { side, selected }
            }
            _ => Screen::Stash { side, selected: input::list_nav(key, selected, len_of(game, side)) },
        }
    }
}

fn screen_restore(screen: &Screen) -> Screen {
    match screen {
        Screen::Encounter => Screen::Encounter,
        _ => Screen::Character,
    }
}

/// The first way into the node at `pos` you have the tools for, to start the
/// open-with pop-up on.
fn first_ready(game: &Game, pos: Pos) -> usize {
    game.node_at(pos).and_then(|d| d.ways_for(&game.player).first().copied()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Outcome;
    use crossterm::event::KeyModifiers;
    use std::path::PathBuf;

    fn press(app: &mut App, code: KeyCode) {
        app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
    }

    fn type_text(app: &mut App, text: &str) {
        text.chars().for_each(|c| press(app, KeyCode::Char(c)));
    }

    /// An empty save folder for one test, removed when it ends.
    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("roguemoji-app-test-{}-{name}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            Dir(dir)
        }

        fn app(&self, shared: bool) -> App {
            App::new(Some(1), Some(Store::new(self.0.clone(), shared)))
        }

        fn saves(&self) -> usize {
            std::fs::read_dir(&self.0).map_or(0, |d| d.count())
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Choose the first background and take the suggested name.
    fn start(app: &mut App) {
        press(app, KeyCode::Enter);
        assert!(matches!(app.screen, Screen::Name { .. }));
        press(app, KeyCode::Enter);
        assert!(matches!(app.screen, Screen::Play));
    }

    fn die(app: &mut App) {
        app.game.as_mut().unwrap().outcome = Some(Outcome::Died("Tested to death in Phase 1".into()));
        press(app, KeyCode::Char('.'));
        assert!(matches!(app.screen, Screen::Ended));
    }

    #[test]
    fn a_new_run_is_saved_at_once_and_kept_on_quitting() {
        let dir = Dir::new("quit");
        let mut app = dir.app(false);
        start(&mut app);
        assert_eq!(dir.saves(), 1, "saved from the first turn");
        press(&mut app, KeyCode::Char('Q'));
        press(&mut app, KeyCode::Char('y'));
        assert!(!app.running);
        assert_eq!(dir.saves(), 1);
    }

    #[test]
    fn continuing_loads_the_run_and_takes_its_file() {
        let dir = Dir::new("continue");
        let mut first = dir.app(false);
        start(&mut first);
        first.game.as_mut().unwrap().wait();
        let turn = first.game.as_ref().unwrap().turn;
        first.save_run();

        let mut second = dir.app(false);
        assert!(second.title_note().is_some_and(|n| n.starts_with("C  continue")));
        press(&mut second, KeyCode::Char('c'));
        assert!(matches!(second.screen, Screen::Play));
        assert_eq!(second.game.as_ref().unwrap().turn, turn);
        assert_eq!(dir.saves(), 0, "loading removes the save until it's written again");
    }

    #[test]
    fn a_new_run_asks_before_replacing_the_saved_one() {
        let dir = Dir::new("replace");
        let mut first = dir.app(false);
        start(&mut first);
        let name = first.game.as_ref().unwrap().player.name.clone();
        first.save_run();

        let mut second = dir.app(false);
        press(&mut second, KeyCode::Enter);
        assert!(matches!(second.screen, Screen::ConfirmNew { .. }));
        press(&mut second, KeyCode::Char('n'));
        assert!(matches!(second.screen, Screen::Title { .. }));
        assert!(second.title_note().is_some_and(|n| n.contains(&name)), "the save is untouched");
        press(&mut second, KeyCode::Enter);
        press(&mut second, KeyCode::Char('y'));
        assert!(matches!(second.screen, Screen::Name { .. }));
    }

    #[test]
    fn the_chosen_name_is_tidied_and_offered_again() {
        let dir = Dir::new("name");
        let mut app = dir.app(false);
        press(&mut app, KeyCode::Enter);
        type_text(&mut app, "  Mad   Max\u{1F600}  ");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.game.as_ref().unwrap().player.name, "Mad Max", "spaces squeezed, emoji refused");
        die(&mut app);
        press(&mut app, KeyCode::Enter);
        press(&mut app, KeyCode::Enter);
        assert!(matches!(&app.screen, Screen::Name { typed, .. } if typed == "Mad Max"));
    }

    #[test]
    fn a_finished_run_is_ranked_once() {
        let dir = Dir::new("ranked");
        let mut app = dir.app(true);
        start(&mut app);
        die(&mut app);
        press(&mut app, KeyCode::Char('x'));
        press(&mut app, KeyCode::Char('x'));
        assert_eq!(app.placed, Some((1, 1)), "extra keys on the end screen don't record it again");
        press(&mut app, KeyCode::Char('L'));
        assert!(matches!(&app.screen, Screen::Scores { list, mine: Some(0), from_end: true } if list.len() == 1));
        press(&mut app, KeyCode::Esc);
        assert!(matches!(app.screen, Screen::Ended));
    }

    #[test]
    fn an_abandoned_run_is_not_ranked() {
        let dir = Dir::new("unranked");
        let mut app = dir.app(false);
        start(&mut app);
        press(&mut app, KeyCode::Char('Q'));
        press(&mut app, KeyCode::Char('a'));
        assert!(app.store.as_ref().unwrap().scores().is_empty());
    }

    #[test]
    fn dying_removes_the_save_for_good() {
        let dir = Dir::new("death");
        let mut app = dir.app(false);
        start(&mut app);
        die(&mut app);
        app.save_run();
        assert_eq!(dir.saves(), 1, "only the leaderboard is left");
        assert_eq!(app.store.as_ref().unwrap().scores().len(), 1);
    }

    #[test]
    fn abandoning_removes_the_save() {
        let dir = Dir::new("abandon");
        let mut app = dir.app(false);
        start(&mut app);
        press(&mut app, KeyCode::Char('Q'));
        press(&mut app, KeyCode::Char('a'));
        assert!(!app.running);
        app.save_run();
        assert_eq!(dir.saves(), 0);
    }

    #[test]
    fn a_shared_run_comes_back_by_its_code() {
        let dir = Dir::new("shared");
        let mut first = dir.app(true);
        start(&mut first);
        let code = first.shared_code().unwrap().to_string();
        let log = first.game.as_ref().unwrap().log.recent(1)[0].text.clone();
        assert!(log.contains(&code), "the code is announced: {log}");
        press(&mut first, KeyCode::Char('Q'));
        press(&mut first, KeyCode::Char('y'));
        assert!(matches!(&first.screen, Screen::Saved { code: c } if *c == code));

        let mut second = dir.app(true);
        press(&mut second, KeyCode::Char('r'));
        type_text(&mut second, "wrong-code");
        press(&mut second, KeyCode::Enter);
        assert!(matches!(&second.screen, Screen::Resume { error: Some(e), .. } if e.contains("isn't a resume code")));
        for _ in 0.."wrong-code".len() {
            press(&mut second, KeyCode::Backspace);
        }
        type_text(&mut second, &code.to_uppercase().replace('-', " "));
        press(&mut second, KeyCode::Enter);
        assert!(matches!(second.screen, Screen::Play));
        assert_eq!(second.shared_code(), Some(code.as_str()));
    }
}
