//! The passage of time.

use super::{Game, Outcome, Tone};
use crate::creatures::player::{HUNGRY_AT, MAX_SATIETY};
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
        self.monsters_act();
        self.tick_monsters();
        self.reap_monsters();
    }

    fn tick_player(&mut self) {
        let p = &mut self.player;

        // Hunger.
        let was_hungry = p.satiety <= HUNGRY_AT;
        p.satiety = (p.satiety - 1).clamp(-1, MAX_SATIETY);
        if !was_hungry && p.satiety == HUNGRY_AT {
            self.log.push("Your stomach growls. You're hungry.", Tone::Bad);
        }
        if p.satiety <= 0 && self.turn.is_multiple_of(15) {
            p.hp -= 1;
            self.log.push("You are starving!", Tone::Bad);
        }

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
            self.outcome = Some(Outcome::Died(format!("Succumbed to wounds and sickness on depth {}", self.depth)));
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
            for (component, chance) in def.drops {
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
