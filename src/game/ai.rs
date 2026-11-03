//! Monster behavior: notice, chase, bite.

use super::{Game, Outcome, Tone};
use crate::combat::{self, Attack};
use crate::creatures::OnHit;
use crate::effects::{EffectKind, EffectSpec};
use crate::geometry::Pos;
use crate::stats::check;
use crate::stats::{Attribute, Skill};
use crate::world::fov::SIGHT_RADIUS;

const TURN_COST: i32 = 100;
const DREAD_TARGET: i32 = 12;

impl Game {
    pub(super) fn monsters_act(&mut self) {
        // One shared distance field toward the player for every hunter.
        let field = self.map.distances_from(self.player.pos);
        for i in 0..self.monsters.len() {
            if self.is_over() {
                return;
            }
            if self.monsters[i].is_dead() {
                continue;
            }
            self.monsters[i].energy += self.monsters[i].def().speed;
            while self.monsters[i].energy >= TURN_COST && !self.is_over() {
                self.monsters[i].energy -= TURN_COST;
                self.monster_turn(i, &field);
            }
        }
    }

    fn monster_turn(&mut self, i: usize, field: &[Option<u32>]) {
        let m = &self.monsters[i];
        if m.is_dead() || m.effects.has(EffectKind::Stun) {
            return;
        }
        let pos = m.pos;
        let target = self.player.pos;
        let dist = pos.distance(target);
        let sees = self.map.is_visible(pos) && dist <= SIGHT_RADIUS;
        if sees && !m.alert {
            self.monsters[i].alert = true;
        }

        if dist == 1 {
            self.monster_attack(i);
            return;
        }
        if self.monsters[i].alert {
            // Follow the distance field downhill toward the player.
            let best = pos
                .neighbors()
                .filter(|n| self.is_free(*n))
                .filter_map(|n| self.map.distance_at(field, n).map(|d| (d, n)))
                .min_by_key(|(d, _)| *d);
            let here = self.map.distance_at(field, pos).unwrap_or(u32::MAX);
            if let Some((d, n)) = best
                && d < here
            {
                self.monsters[i].pos = n;
            }
        } else if self.dice.chance(0.3) {
            let dirs: Vec<Pos> = pos.neighbors().filter(|n| self.is_free(*n)).collect();
            if !dirs.is_empty() {
                let n = *self.dice.pick(&dirs);
                self.monsters[i].pos = n;
            }
        }
    }

    fn monster_attack(&mut self, i: usize) {
        let def = self.monsters[i].def();
        let name = &def.name;
        let attack = Attack {
            to_hit: def.attack,
            raw_damage: self.dice.range(def.damage.0, def.damage.1),
            pen: 0,
            ignore_armor: false,
        };
        let out = combat::resolve(&mut self.dice, attack, self.player.defense(), self.player.armor_value());

        if !out.tier.is_solid() && out.damage == 0 {
            if out.tier == combat::HitTier::Miss {
                self.log.push(format!("The {name} misses you."), Tone::Combat);
                self.train(Skill::Dodge, 1);
            } else {
                self.log.push(format!("The {name} grazes you, but your armor holds."), Tone::Combat);
            }
            return;
        }

        self.player.hp -= out.damage;
        if out.damage > 0 {
            self.player.momentum.reset();
        }
        let tone = if out.tier == combat::HitTier::Crit { Tone::Bad } else { Tone::Combat };
        self.log.push(format!("The {name} {} you for {}.", out.tier.verb(), out.damage), tone);

        if out.tier.is_solid() {
            match def.on_hit {
                OnHit::None => {}
                OnHit::Inflict(spec) => {
                    self.player.effects.add(spec, name);
                    self.log.push(format!("You are {}!", spec.kind.label().to_lowercase()), Tone::Bad);
                }
                OnHit::Dread => {
                    let resolve =
                        check::roll(&mut self.dice, self.player.attr_bonus(Attribute::Willpower), DREAD_TARGET);
                    if !resolve.tier.succeeded() {
                        self.player.effects.add(EffectSpec::new(EffectKind::Stun, 0, 1), "dread");
                        self.log.push(format!("Terror of the {name} freezes you in place!"), Tone::Bad);
                    } else {
                        self.log.push("You steel your nerve against the dread.", Tone::Good);
                    }
                }
            }
        }

        if self.player.is_dead() && self.outcome.is_none() {
            let article = if name.starts_with(['A', 'E', 'I', 'O', 'U']) { "an" } else { "a" };
            self.outcome = Some(Outcome::Died(format!("Killed by {article} {name} on depth {}", self.depth)));
        }
    }
}
