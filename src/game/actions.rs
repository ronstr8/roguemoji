//! Player actions. Each public method validates the action, applies it, and
//! advances time only if the action actually happened.

use super::{Game, Outcome, Tone};
use crate::combat::{self, Attack, HitTier};
use crate::crafting::{Recipe, recipes};
use crate::creatures::player::{Burden, MAX_SATIETY};
use crate::effects::{EffectKind, EffectSpec};
use crate::geometry::{Dir, Pos, line};
use crate::items::loot;
use crate::items::melee::Special;
use crate::items::{Consumable, Gun, Item, ItemKind};
use crate::stats::Skill;
use crate::stats::check::{self, Tier};
use crate::world::fov::SIGHT_RADIUS;
use crate::world::{Feature, Harvestable};

/// Something the UI needs to react to after an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    OpenShop(usize),
    OpenStash,
}

/// Where an item moves when shuffling between pack and lockbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StashSide {
    Pack,
    Lockbox,
}

/// Radius within which a gunshot wakes monsters.
const GUNSHOT_NOISE: i32 = 14;

impl Game {
    pub(super) fn train(&mut self, skill: Skill, xp: u32) {
        if self.player.skills.train(skill, xp) {
            let rank = self.player.skills.rank(skill);
            self.log.push(format!("Your {} skill improves to rank {}!", skill.name(), rank), Tone::Good);
        }
    }

    // ── Movement & interaction ────────────────────────────────────────────

    pub fn move_player(&mut self, dir: Dir) -> Event {
        let target = self.player.pos.step(dir);
        if let Some(idx) = self.monster_index_at(target) {
            self.melee_attack(idx);
            return Event::None;
        }
        match self.map.feature(target).copied() {
            Some(Feature::Harvest { kind, .. }) => {
                self.harvest(target, kind);
                return Event::None;
            }
            Some(Feature::Kiosk(k)) => return Event::OpenShop(k),
            Some(Feature::Lockbox) => {
                self.log
                    .push("The lockbox irises open. Its contents are the same as every other one's.", Tone::Special);
                return Event::OpenStash;
            }
            _ => {}
        }
        if !self.map.passable(target) {
            return Event::None;
        }
        let burden = self.player.burden();
        if burden == Burden::Overloaded {
            self.log.push(burden.describe(), Tone::Bad);
            return Event::None;
        }
        self.player.pos = target;
        self.describe_floor();
        if self.map.feature(target) == Some(&Feature::Beacon) {
            self.log.push("You lay your hand on the Beacon. It sings, and the sky answers.", Tone::Special);
            self.outcome = Some(Outcome::Won);
            return Event::None;
        }
        // Heavy loads cost fractional turns; carry the remainder to the next step.
        let halves = self.player.move_debt + burden.move_cost_halves();
        self.player.move_debt = halves % 2;
        self.pass_time(halves / 2);
        Event::None
    }

    fn describe_floor(&mut self) {
        let here: Vec<String> = self.items_at(self.player.pos).map(|fi| fi.item.display_name()).collect();
        match here.len() {
            0 => {}
            1 => self.log.info(format!("You see {} here. (g to pick up)", here[0])),
            n => self.log.info(format!("You see {n} items here. (g to pick up)")),
        }
        if self.map.feature(self.player.pos) == Some(&Feature::Stairs) {
            self.log.info("A way down. (> to descend)");
        }
    }

    pub fn wait(&mut self) {
        self.pass_time(1);
    }

    pub fn cycle_stance(&mut self) {
        self.player.stance = self.player.stance.next();
        self.log.info(format!("You shift to a {} stance.", self.player.stance.name()));
    }

    pub fn descend(&mut self) {
        if self.map.feature(self.player.pos) != Some(&Feature::Stairs) {
            self.log.info("There is no way down here.");
            return;
        }
        self.enter_level(self.depth + 1);
    }

    pub fn pickup(&mut self) {
        let pos = self.player.pos;
        let (here, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut self.items).into_iter().partition(|fi| fi.pos == pos);
        self.items = rest;
        if here.is_empty() {
            self.log.info("There is nothing here.");
            return;
        }
        for fi in here {
            if fi.item.kind == ItemKind::Crystals {
                self.player.crystals += fi.item.qty;
                self.log.push(format!("You gather {} 💎.", fi.item.qty), Tone::Loot);
            } else {
                self.log.push(format!("You pick up {}.", fi.item.display_name()), Tone::Loot);
                self.give_player(fi.item);
            }
        }
        self.pass_time(1);
    }

    /// Absorb crystals to heal: one, or as many as needed to top off.
    pub fn absorb(&mut self, to_full: bool) {
        let p = &self.player;
        let missing = p.max_hp() - p.hp;
        if missing <= 0 {
            self.log.info("You're already at full health.");
            return;
        }
        if p.crystals == 0 {
            self.log.push("You have no crystals to absorb.", Tone::Bad);
            return;
        }
        let per = p.crystal_heal();
        let wanted = if to_full { ((missing + per - 1) / per) as u32 } else { 1 };
        let used = wanted.min(p.crystals);
        self.player.crystals -= used;
        let healed = self.player.heal(per * used as i32);
        self.log.push(format!("You crush {used} 💎 in your fist and absorb the light: +{healed} HP."), Tone::Good);
        self.pass_time(1);
    }

    fn harvest(&mut self, pos: Pos, kind: Harvestable) {
        let def = kind.def();
        let skill = self.player.skill(def.skill).total();
        let result = check::roll(&mut self.dice, skill, def.difficulty);
        let mut gathered: Vec<(crate::items::Component, u32)> = Vec::new();
        let mut used = true;
        match result.tier {
            Tier::Fumble => {
                self.player.hp -= 1;
                self.log.push(format!("You fumble with the {} and hurt yourself.", def.name), Tone::Bad);
            }
            Tier::Fail => {
                used = false;
                self.log.info(format!(
                    "You try to {} the {} but come up empty. {}",
                    def.verb,
                    def.name,
                    result.summary()
                ));
            }
            tier => {
                for (c, lo, hi) in def.yields {
                    let mut n = self.dice.range(*lo as i32, *hi as i32) as u32;
                    if n > 0 && tier == Tier::Critical {
                        n += 1;
                    }
                    if n > 0 {
                        gathered.push((crate::items::Component::named(c), n));
                    }
                    if tier == Tier::Partial && !gathered.is_empty() {
                        break;
                    }
                }
                if gathered.is_empty() {
                    let (c, lo, _) = def.yields[0];
                    gathered.push((crate::items::Component::named(c), lo.max(1)));
                }
                self.train(def.skill, if tier == Tier::Critical { 2 } else { 1 });
            }
        }
        if !gathered.is_empty() {
            let names: Vec<String> = gathered.iter().map(|(c, n)| format!("{} {}", n, c.def().name)).collect();
            self.log.push(
                format!("You {} the {}: {}. {}", def.verb, def.name, names.join(", "), result.summary()),
                Tone::Loot,
            );
            for (c, n) in gathered {
                self.give_player(Item::component(c, n));
            }
        }
        if used && let Some(Feature::Harvest { uses_left, .. }) = self.map.feature_mut(pos) {
            *uses_left = uses_left.saturating_sub(1);
            if *uses_left == 0 {
                self.map.remove_feature(pos);
                self.log.info(format!("The {} is picked clean.", def.name));
            }
        }
        self.pass_time(1);
    }

    // ── Combat ────────────────────────────────────────────────────────────

    fn melee_attack(&mut self, idx: usize) {
        let w = self.player.melee_def();
        let (id, name, defense, armor) = {
            let m = &self.monsters[idx];
            (m.id, m.name(), m.defense(), m.armor())
        };
        let p = &self.player;
        let attack = Attack {
            to_hit: p.skill(w.skill).total()
                + w.accuracy
                + p.stance.attack_mod()
                + p.momentum.bonus_against(id)
                + p.focus(),
            raw_damage: self.dice.range(w.damage.0, w.damage.1) + p.brawn_damage() + p.stance.damage_mod(),
            pen: w.pen,
            ignore_armor: w.special == Special::IgnoreArmor,
        };
        let out = combat::resolve(&mut self.dice, attack, defense, armor);
        self.player.momentum.record(id, out.tier);
        self.monsters[idx].alert = true;

        let tone = if out.tier == HitTier::Crit { Tone::Good } else { Tone::Combat };
        match out.tier {
            HitTier::Miss => self.log.push(format!("Your {} misses the {name}.", w.name.to_lowercase()), tone),
            _ => self.log.push(format!("You {} the {name} for {}.", verb_2p(out.tier), out.damage), tone),
        }
        self.monsters[idx].hp -= out.damage;

        if out.tier.is_solid() && !self.monsters[idx].is_dead() {
            self.apply_special(idx, w.special, out.tier, out.damage);
        }
        if out.tier != HitTier::Miss {
            self.train(w.skill, if out.tier == HitTier::Crit { 2 } else { 1 });
        }
        self.reap_monsters();
        self.pass_time(1);
    }

    fn apply_special(&mut self, idx: usize, special: Special, tier: HitTier, damage: i32) {
        let name = self.monsters[idx].name();
        let crit = tier == HitTier::Crit;
        match special {
            Special::None | Special::IgnoreArmor => {}
            Special::Bleed => {
                self.monsters[idx]
                    .effects
                    .add(EffectSpec::new(EffectKind::Bleed, if crit { 2 } else { 1 }, 4), "wound");
                self.log.push(format!("The {name} is bleeding."), Tone::Combat);
            }
            Special::Stun => {
                if crit || self.dice.chance(0.35) {
                    self.monsters[idx]
                        .effects
                        .add(EffectSpec::new(EffectKind::Stun, 0, if crit { 2 } else { 1 }), "blow");
                    self.log.push(format!("The {name} is stunned."), Tone::Combat);
                }
            }
            Special::Knockback => {
                let from = self.player.pos;
                let at = self.monsters[idx].pos;
                let dest = at.offset((at.x - from.x).signum(), (at.y - from.y).signum());
                if self.is_free(dest) {
                    self.monsters[idx].pos = dest;
                }
                self.monsters[idx].effects.add(EffectSpec::new(EffectKind::Stun, 0, 1), "impact");
                self.log.push(format!("The {name} is hurled back!"), Tone::Combat);
            }
            Special::Drain => {
                let healed = self.player.heal((damage / 3).max(1));
                if healed > 0 {
                    self.log.push(format!("You drink in {healed} HP."), Tone::Good);
                }
            }
        }
    }

    /// Monsters you can see and have a clear line of fire to, nearest first.
    pub fn targets(&self) -> Vec<u32> {
        self.visible_monsters()
            .into_iter()
            .filter(|m| m.pos.distance(self.player.pos) <= SIGHT_RADIUS + 4)
            .filter(|m| crate::world::fov::clear_line(&self.map, self.player.pos, m.pos))
            .map(|m| m.id)
            .collect()
    }

    /// To-hit modifier and defense for a shot at `id`, for the targeting UI.
    pub fn shot_odds(&self, id: u32) -> Option<(i32, i32)> {
        let gun = self.player.gun.as_ref()?;
        let ammo = gun.ammo?;
        let m = self.monster(id)?;
        let dist = m.pos.distance(self.player.pos);
        Some((self.shot_modifier(gun, ammo, dist, id), m.defense()))
    }

    fn shot_modifier(&self, gun: &Gun, ammo: crate::items::Ammo, dist: i32, id: u32) -> i32 {
        let p = &self.player;
        p.skill(gun.def().skill).total()
            + gun.hit_modifier(ammo, dist)
            + p.stance.attack_mod()
            + p.momentum.bonus_against(id)
            + p.focus()
    }

    pub fn fire_at(&mut self, target_id: u32) {
        let Some(gun) = self.player.gun.clone() else {
            self.log.info("You have no gun equipped.");
            return;
        };
        let ammo = match gun.ammo {
            Some(a) if gun.loaded > 0 => a,
            _ => {
                self.log.push(format!("*click* The {} is empty. (r to reload)", gun.def().name), Tone::Bad);
                return;
            }
        };
        let Some(target_pos) = self.monster(target_id).map(|m| m.pos) else { return };

        // The bullet stops at the first creature or wall along its path.
        let mut hit_idx = None;
        for p in line(self.player.pos, target_pos).into_iter().skip(1) {
            if self.map.opaque(p) {
                break;
            }
            if let Some(i) = self.monster_index_at(p) {
                hit_idx = Some(i);
                break;
            }
        }
        if let Some(g) = self.player.gun.as_mut() {
            g.loaded -= 1;
        }
        self.alert_nearby(GUNSHOT_NOISE);

        let Some(idx) = hit_idx else {
            self.log.push("Your shot strikes the wall.", Tone::Combat);
            self.pass_time(1);
            return;
        };
        let (id, name, pos, defense, armor) = {
            let m = &self.monsters[idx];
            (m.id, m.name(), m.pos, m.defense(), m.armor())
        };
        let dist = pos.distance(self.player.pos);
        let profile = gun.ballistics(ammo);
        let fraction = Gun::pellet_fraction(ammo, dist);
        let raw = (profile.base_damage * fraction * self.dice.float(0.8, 1.2)).round().max(1.0) as i32;
        // A spread of small pellets is stopped by armor more easily.
        let effective_armor = if profile.projectiles > 1 { armor * 2 } else { armor };
        let attack = Attack {
            to_hit: self.shot_modifier(&gun, ammo, dist, id),
            raw_damage: raw,
            pen: profile.pen,
            ignore_armor: false,
        };
        let out = combat::resolve(&mut self.dice, attack, defense, effective_armor);
        self.player.momentum.record(id, out.tier);

        let gun_name = &gun.def().name;
        let tone = if out.tier == HitTier::Crit { Tone::Good } else { Tone::Combat };
        match out.tier {
            HitTier::Miss => self.log.push(format!("You fire the {gun_name} and miss the {name}."), tone),
            _ => self
                .log
                .push(format!("You fire the {gun_name}: {} the {name} for {}.", verb_3p(out.tier), out.damage), tone),
        }
        self.monsters[idx].hp -= out.damage;
        if out.tier != HitTier::Miss {
            self.train(gun.def().skill, if out.tier == HitTier::Crit { 2 } else { 1 });
        }
        self.reap_monsters();
        self.pass_time(1);
    }

    fn alert_nearby(&mut self, radius: i32) {
        let me = self.player.pos;
        for m in &mut self.monsters {
            if m.pos.distance(me) <= radius {
                m.alert = true;
            }
        }
    }

    pub fn reload(&mut self) {
        let Some(mut gun) = self.player.gun.clone() else {
            self.log.info("You have no gun equipped.");
            return;
        };
        let def = gun.def();
        let inv = &self.player.inventory;
        let choice = match gun.ammo {
            Some(a) if inv.count_ammo(a) > 0 => Some(a),
            _ => inv.ammo_types(|a| gun.accepts(a)).first().copied(),
        };
        let Some(ammo) = choice else {
            self.log.push(format!("You have no {} ammunition.", def.cartridge.name()), Tone::Bad);
            return;
        };
        if gun.ammo != Some(ammo) && gun.loaded > 0 {
            if let Some(old) = gun.ammo {
                self.give_player(Item::ammo(old, gun.loaded));
            }
            gun.loaded = 0;
        }
        let need = def.capacity - gun.loaded;
        if need == 0 {
            self.log.info(format!("The {} is already full.", def.name));
            return;
        }
        let have = self.player.inventory.count_ammo(ammo);
        let n = need.min(have);
        self.player.inventory.consume_ammo(ammo, n);
        gun.loaded += n;
        gun.ammo = Some(ammo);
        let turns = def.action.reload_turns();
        self.log.info(format!("You load {n} rounds of {} into the {}.", ammo.name(), def.name));
        self.player.gun = Some(gun);
        self.pass_time(turns);
    }

    // ── Inventory ─────────────────────────────────────────────────────────

    /// Eat, drink, or equip the item in pack slot `idx`.
    pub fn use_item(&mut self, idx: usize) {
        let Some(item) = self.player.inventory.get(idx).cloned() else { return };
        match item.kind {
            ItemKind::Consumable(c) => {
                self.player.inventory.take(idx, 1);
                self.consume(c);
                self.pass_time(1);
            }
            ItemKind::Gun(_) | ItemKind::Melee(_) | ItemKind::Armor(_) | ItemKind::Pack(_) => self.equip(idx),
            ItemKind::Ammo(_) => self.log.info("Equip a matching gun and press r to load it."),
            ItemKind::Component(_) => self.log.info("Components are used for crafting (m)."),
            ItemKind::Crystals => {}
        }
    }

    fn consume(&mut self, c: Consumable) {
        let def = c.def();
        let p = &mut self.player;
        p.satiety = (p.satiety + def.satiety).min(MAX_SATIETY);
        let healed = if def.heal > 0 { p.heal(def.heal) } else { 0 };
        if def.cures {
            p.effects.clear_harmful();
        }
        for spec in &def.effects {
            p.effects.add(*spec, &def.name);
        }
        let mut msg = format!("You consume the {}.", def.name);
        if healed > 0 {
            msg.push_str(&format!(" +{healed} HP."));
        }
        if !def.effects.is_empty() {
            let fx: Vec<String> = def.effects.iter().map(|e| e.describe()).collect();
            msg.push_str(&format!(" ({})", fx.join(", ")));
        }
        self.log.push(msg, Tone::Good);
    }

    fn equip(&mut self, idx: usize) {
        let Some(item) = self.player.inventory.take(idx, 1) else { return };
        let p = &mut self.player;
        let (old, name) = match item.kind {
            ItemKind::Gun(g) => {
                let name = g.def().name.as_str();
                (p.gun.replace(g).map(Item::gun), name)
            }
            ItemKind::Melee(m) => (p.melee.replace(m).map(|o| Item::one(ItemKind::Melee(o))), m.def().name.as_str()),
            ItemKind::Armor(a) => (p.armor.replace(a).map(|o| Item::one(ItemKind::Armor(o))), a.def().name.as_str()),
            ItemKind::Pack(k) => (p.pack.replace(k).map(|o| Item::one(ItemKind::Pack(o))), k.def().name.as_str()),
            other => {
                let _ = p.inventory.add(Item { kind: other, qty: item.qty });
                return;
            }
        };
        if let Some(old) = old {
            self.give_player(old);
        }
        self.log.info(format!("You ready the {name}."));
        self.pass_time(1);
    }

    pub fn unequip_melee(&mut self) {
        if let Some(m) = self.player.melee.take() {
            self.give_player(Item::one(ItemKind::Melee(m)));
            self.log.info(format!("You stow the {} and raise your fists.", m.def().name));
        }
    }

    pub fn drop_item(&mut self, idx: usize) {
        let Some(item) = self.player.inventory.take(idx, u32::MAX) else { return };
        self.log.info(format!("You drop {}.", item.display_name()));
        let pos = self.player.pos;
        self.drop_at(pos, item);
        self.pass_time(1);
    }

    /// Move a whole stack between the pack and the lockbox stash.
    pub fn stash_transfer(&mut self, from: StashSide, idx: usize) {
        let (src, dst) = match from {
            StashSide::Pack => (&mut self.player.inventory, &mut self.stash),
            StashSide::Lockbox => (&mut self.stash, &mut self.player.inventory),
        };
        let Some(item) = src.get(idx) else { return };
        if !dst.has_room_for(item) {
            let msg = match from {
                StashSide::Pack => "The lockbox is full.",
                StashSide::Lockbox => "Your pack has no free slots.",
            };
            self.log.push(msg, Tone::Bad);
            return;
        }
        let Some(item) = src.take(idx, u32::MAX) else { return };
        let name = item.display_name();
        let _ = dst.add(item);
        match from {
            StashSide::Pack => self.log.info(format!("You stow {name} in the lockbox.")),
            StashSide::Lockbox => self.log.push(format!("You take {name} from the lockbox."), Tone::Loot),
        }
        self.pass_time(1);
    }

    // ── Crafting & trade ──────────────────────────────────────────────────

    pub fn has_tool(&self, recipe: &Recipe) -> bool {
        match recipe.consumes_tool {
            None => true,
            Some(tool) => {
                self.player.melee == Some(tool)
                    || self.player.inventory.items().iter().any(|i| i.kind == ItemKind::Melee(tool))
            }
        }
    }

    pub fn can_craft(&self, recipe: &Recipe) -> bool {
        self.has_tool(recipe) && recipe.components().all(|(c, n)| self.player.inventory.count_component(c) >= n)
    }

    pub fn craft(&mut self, recipe_idx: usize) {
        let Some(recipe) = recipes().get(recipe_idx) else { return };
        let name = recipe.name();
        if !self.can_craft(recipe) {
            self.log.push(format!("You lack what you need for {name}."), Tone::Bad);
            return;
        }
        let skill = self.player.skill(Skill::Crafting).total();
        let result = check::roll(&mut self.dice, skill, recipe.difficulty);
        match result.tier {
            Tier::Fail => {
                self.log.info(format!("You can't get the {name} right. Materials intact. {}", result.summary()));
            }
            Tier::Fumble => {
                self.consume_ingredients(recipe);
                self.log.push(format!("You botch the {name} and ruin the materials."), Tone::Bad);
            }
            tier => {
                self.consume_ingredients(recipe);
                let mut item = recipe.output.item();
                let mut msg = format!("You craft {}.", item.base_name());
                if tier == Tier::Critical {
                    if recipe.output.is_food() {
                        item.qty = 2;
                        msg = format!("Masterful! You make a double batch of {}.", item.base_name());
                    } else if let Some((c, _)) = recipe.components().next() {
                        self.give_player(Item::component(c, 1));
                        msg = format!("Masterful! You craft {} with material to spare.", item.base_name());
                    }
                }
                self.log.push(format!("{msg} {}", result.summary()), Tone::Good);
                self.give_player(item);
                self.train(Skill::Crafting, 1 + (recipe.difficulty - 8).max(0) as u32 / 3);
            }
        }
        self.pass_time(2);
    }

    fn consume_ingredients(&mut self, recipe: &Recipe) {
        for (c, n) in recipe.components() {
            self.player.inventory.consume_component(c, n);
        }
        if let Some(tool) = recipe.consumes_tool {
            let kind = ItemKind::Melee(tool);
            if let Some(i) = self.player.inventory.items().iter().position(|it| it.kind == kind) {
                self.player.inventory.take(i, 1);
            } else if self.player.melee == Some(tool) {
                self.player.melee = None;
            }
        }
    }

    pub fn buy(&mut self, kiosk: usize, idx: usize) {
        let Some((item, price)) = self.kiosks.get(kiosk).and_then(|k| k.stock.get(idx)).cloned() else { return };
        if self.player.crystals < price {
            self.log.push(format!("The Autovend flashes: INSUFFICIENT CRYSTAL ({price} needed)."), Tone::Bad);
            return;
        }
        self.player.crystals -= price;
        self.kiosks[kiosk].stock.remove(idx);
        self.log.push(format!("*ka-chunk* The Autovend dispenses {}.", item.display_name()), Tone::Loot);
        self.give_player(item);
        self.note_burden();
    }

    /// Sell from the pack: a whole stack of ammo, otherwise one item.
    pub fn sell(&mut self, idx: usize) {
        let Some(item) = self.player.inventory.get(idx) else { return };
        let Some(price) = loot::sell_price(item, self.depth) else {
            self.log.push("The Autovend flashes: ITEM NOT RECOGNIZED.", Tone::Bad);
            return;
        };
        let n = loot::sell_qty(item);
        let Some(sold) = self.player.inventory.take(idx, n) else { return };
        self.player.crystals += price;
        self.log
            .push(format!("*whirr* The Autovend takes {} and pays out {price} 💎.", sold.display_name()), Tone::Loot);
        self.note_burden();
    }
}

fn verb_2p(tier: HitTier) -> &'static str {
    match tier {
        HitTier::Miss => "miss",
        HitTier::Graze => "graze",
        HitTier::Hit => "hit",
        HitTier::Crit => "CRIT",
    }
}

fn verb_3p(tier: HitTier) -> &'static str {
    tier.verb()
}
