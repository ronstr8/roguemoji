//! The passage of time.

use super::{Game, Outcome, Tone};
use crate::creatures::needs::{Change, Strain};
use crate::creatures::player::Sleep;
use crate::effects::{EffectKind, EffectSpec};
use crate::items::{Component, Item};
use crate::world::Tile;

/// Safety cap so chained stuns can never lock the game.
const MAX_STUN_CHAIN: u32 = 6;

impl Game {
    /// Advance the world after the player spends `turns` turns of time.
    pub(super) fn pass_time(&mut self, turns: u32) {
        for _ in 0..turns {
            self.one_turn();
        }
        let mut chained = 0;
        while self.player.effects.has(EffectKind::Stun) && !self.is_over() && chained < MAX_STUN_CHAIN {
            self.log.push("You reel, stunned!", Tone::Bad);
            self.one_turn();
            chained += 1;
        }
        self.refresh_fov();
        self.note_burden();
    }

    fn one_turn(&mut self) {
        if self.is_over() {
            return;
        }
        self.turn += 1;
        self.tick_player();
        if self.is_over() {
            return;
        }
        self.refresh_fov();
        let hp_before_monsters = self.player.hp;
        self.monsters_act();
        self.tick_monsters();
        self.reap_monsters();
        self.check_wake(hp_before_monsters);
    }

    /// Drain hunger and thirst, and rest unless asleep, where it refills.
    /// Logs stages entered and hurts; collapses you when rest runs out.
    fn tick_needs(&mut self) {
        let p = &mut self.player;
        let strain = Strain { fought: std::mem::take(&mut p.fought), burden_halves: p.burden().move_cost_halves() };
        let turn = self.turn;
        let mut changes: Vec<Change> = p.waking_needs_mut().map(|m| m.tick(turn, strain)).into();
        if p.is_asleep() {
            let sleep = p.sleep_def();
            p.rest.adjust(sleep.recover);
            if sleep.heal_every > 0 && turn.is_multiple_of(u64::from(sleep.heal_every)) {
                p.heal(1);
            }
        } else {
            changes.push(p.rest.tick(turn, strain));
        }
        for change in changes {
            if let Some(msg) = change.message {
                self.log.push(msg, Tone::Bad);
            }
            if let Some(msg) = change.hurt {
                self.player.hp -= 1;
                self.log.push(msg, Tone::Bad);
            }
        }
        if !self.player.is_asleep() && self.player.rest.value == 0 {
            self.player.sleep = Some(Sleep::Collapse);
            self.log.push("Exhaustion takes you. You collapse where you stand. 💤", Tone::Bad);
        }
    }

    /// Wake the player if something should: being hit, or for a nap, a
    /// monster in view or full rest; after collapsing, rest back to `wake_at`.
    fn check_wake(&mut self, hp_before_monsters: i32) {
        let Some(kind) = self.player.sleep else { return };
        let p = &self.player;
        let reason = if p.hp < hp_before_monsters {
            Some(("Pain jolts you awake!", true))
        } else if kind == Sleep::Nap && !self.in_sight().is_empty() {
            Some(("Something moves nearby. You wake.", false))
        } else if kind == Sleep::Nap && p.rest.value >= p.rest.def().max {
            Some(("You wake, rested.", false))
        } else if kind == Sleep::Collapse && p.rest.value >= p.sleep_def().wake_at {
            Some(("You come to, still worn out.", false))
        } else {
            None
        };
        if let Some((msg, jolted)) = reason {
            self.wake(msg, jolted);
        }
    }

    /// End sleep. Jolted awake, you're groggy for a turn.
    pub(super) fn wake(&mut self, msg: &str, jolted: bool) {
        self.player.sleep = None;
        if jolted {
            self.player.effects.add(EffectSpec::new(EffectKind::Stun, 0, 1), "grogginess");
        }
        self.log.push(msg, if jolted { Tone::Bad } else { Tone::Info });
    }

    fn tick_player(&mut self) {
        self.tick_needs();
        let p = &mut self.player;

        // Toxic terrain.
        if self.map.tile(p.pos) == Tile::Sludge {
            p.effects.add(EffectSpec::new(EffectKind::Poison, 1, 4), "sludge");
            self.log.push("The sludge burns your skin.", Tone::Bad);
        }

        let report = p.effects.tick();
        if report.damage > 0 {
            p.hp -= report.damage;
            p.momentum.reset();
        }
        if report.healing > 0 {
            p.heal(report.healing);
        }
        // Max HP can fall when a Fortitude boost expires.
        p.hp = p.hp.min(p.max_hp());
        for label in report.expired {
            self.log.push(format!("{label} wears off."), Tone::Info);
        }
        if p.is_dead() && self.outcome.is_none() {
            self.outcome = Some(Outcome::Died(format!("Succumbed to wounds and sickness in Phase {}", self.depth)));
        }
    }

    fn tick_monsters(&mut self) {
        for m in &mut self.monsters {
            if m.is_dead() {
                continue;
            }
            let report = m.effects.tick();
            m.hp = (m.hp - report.damage + report.healing).min(m.max_hp);
        }
    }

    /// Remove dead monsters, dropping their crystals and remains.
    pub(super) fn reap_monsters(&mut self) {
        let dead: Vec<usize> = (0..self.monsters.len()).filter(|&i| self.monsters[i].is_dead()).collect();
        for &i in dead.iter().rev() {
            let m = self.monsters.remove(i);
            let def = m.def();
            let crystals = self.dice.range(def.crystals.0 as i32, def.crystals.1 as i32) as u32;
            let mut drops: Vec<Component> = Vec::new();
            for (component, chance) in &def.drops {
                if self.dice.chance(*chance) {
                    drops.push(*component);
                }
            }
            self.drop_at(m.pos, Item::crystals(crystals));
            for c in &drops {
                self.drop_at(m.pos, Item::component(*c, 1));
            }
            self.player.kills += 1;
            self.player.momentum.forget(m.id);
            let visible = self.map.is_visible(m.pos);
            if visible {
                self.log.push(format!("The {} dies, crystallizing into {} 💎.", def.name, crystals), Tone::Loot);
            }
        }
    }
}
