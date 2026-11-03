//! Timed status effects shared by the player and monsters.

use crate::stats::Attribute;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectKind {
    /// Lose `potency` HP every other turn.
    Poison,
    /// Lose `potency` HP every turn.
    Bleed,
    /// Lose turns.
    Stun,
    /// Regain `potency` HP every other turn.
    Regen,
    /// Modify an attribute by `potency` (may be negative).
    Boost(Attribute),
    /// Add `potency` to armor.
    Fortified,
    /// Add `potency` to every to-hit roll.
    Focus,
}

impl EffectKind {
    pub fn label(self) -> String {
        match self {
            EffectKind::Poison => "Poisoned".into(),
            EffectKind::Bleed => "Bleeding".into(),
            EffectKind::Stun => "Stunned".into(),
            EffectKind::Regen => "Regen".into(),
            EffectKind::Boost(a) => a.abbrev().into(),
            EffectKind::Fortified => "Fortified".into(),
            EffectKind::Focus => "Focus".into(),
        }
    }

    pub fn is_harmful(self) -> bool {
        matches!(self, EffectKind::Poison | EffectKind::Bleed | EffectKind::Stun)
    }
}

/// A static description of an effect, used by item and monster tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectSpec {
    pub kind: EffectKind,
    pub potency: i32,
    pub turns: u32,
}

impl EffectSpec {
    pub const fn new(kind: EffectKind, potency: i32, turns: u32) -> Self {
        Self { kind, potency, turns }
    }

    pub fn describe(&self) -> String {
        match self.kind {
            EffectKind::Boost(a) => format!("{:+} {} for {} turns", self.potency, a.name(), self.turns),
            EffectKind::Regen => format!("regenerate {} HP / 2 turns for {} turns", self.potency, self.turns),
            EffectKind::Fortified => format!("+{} armor for {} turns", self.potency, self.turns),
            EffectKind::Focus => format!("+{} to hit for {} turns", self.potency, self.turns),
            EffectKind::Poison => format!("poison {} for {} turns", self.potency, self.turns),
            EffectKind::Bleed => format!("bleed {} for {} turns", self.potency, self.turns),
            EffectKind::Stun => format!("stun for {} turns", self.turns),
        }
    }
}

/// An effect as written in content files:
/// `{ kind = "boost", attribute = "brawn", potency = 2, turns = 120 }`.
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RawEffect {
    Poison { potency: i32, turns: u32 },
    Bleed { potency: i32, turns: u32 },
    Stun { turns: u32 },
    Regen { potency: i32, turns: u32 },
    Boost { attribute: Attribute, potency: i32, turns: u32 },
    Fortified { potency: i32, turns: u32 },
    Focus { potency: i32, turns: u32 },
}

impl RawEffect {
    pub fn spec(&self) -> EffectSpec {
        match *self {
            RawEffect::Poison { potency, turns } => EffectSpec::new(EffectKind::Poison, potency, turns),
            RawEffect::Bleed { potency, turns } => EffectSpec::new(EffectKind::Bleed, potency, turns),
            RawEffect::Stun { turns } => EffectSpec::new(EffectKind::Stun, 1, turns),
            RawEffect::Regen { potency, turns } => EffectSpec::new(EffectKind::Regen, potency, turns),
            RawEffect::Boost { attribute, potency, turns } => {
                EffectSpec::new(EffectKind::Boost(attribute), potency, turns)
            }
            RawEffect::Fortified { potency, turns } => EffectSpec::new(EffectKind::Fortified, potency, turns),
            RawEffect::Focus { potency, turns } => EffectSpec::new(EffectKind::Focus, potency, turns),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    pub potency: i32,
    pub turns: u32,
    pub source: &'static str,
}

/// Result of advancing effects by one turn.
#[derive(Default, Debug)]
pub struct TickReport {
    pub damage: i32,
    pub healing: i32,
    pub expired: Vec<String>,
}

const MAX_HARM_TURNS: u32 = 30;

#[derive(Clone, Debug, Default)]
pub struct Effects {
    list: Vec<Effect>,
    clock: u64,
}

impl Effects {
    /// Apply an effect. Ailments and generic buffs of the same kind merge
    /// (strongest potency, stacked duration); attribute boosts merge only with
    /// the same source so a drink's penalty can't cancel another's bonus.
    pub fn add(&mut self, spec: EffectSpec, source: &'static str) {
        let existing = self
            .list
            .iter_mut()
            .find(|e| e.kind == spec.kind && (!matches!(spec.kind, EffectKind::Boost(_)) || e.source == source));
        match existing {
            Some(e) => {
                e.potency = e.potency.max(spec.potency);
                e.turns = if spec.kind.is_harmful() {
                    (e.turns + spec.turns).min(MAX_HARM_TURNS)
                } else {
                    e.turns.max(spec.turns)
                };
            }
            None => self.list.push(Effect { kind: spec.kind, potency: spec.potency, turns: spec.turns, source }),
        }
    }

    pub fn has(&self, kind: EffectKind) -> bool {
        self.list.iter().any(|e| e.kind == kind)
    }

    pub fn total(&self, kind: EffectKind) -> i32 {
        self.list.iter().filter(|e| e.kind == kind).map(|e| e.potency).sum()
    }

    pub fn attr_mod(&self, a: Attribute) -> i32 {
        self.total(EffectKind::Boost(a))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Effect> {
        self.list.iter()
    }

    pub fn clear_harmful(&mut self) {
        self.list.retain(|e| !e.kind.is_harmful());
    }

    /// Advance one turn: apply damage/healing and expire finished effects.
    pub fn tick(&mut self) -> TickReport {
        self.clock += 1;
        let even = self.clock.is_multiple_of(2);
        let mut report = TickReport::default();
        for e in &mut self.list {
            match e.kind {
                EffectKind::Bleed => report.damage += e.potency,
                EffectKind::Poison if even => report.damage += e.potency,
                EffectKind::Regen if even => report.healing += e.potency,
                _ => {}
            }
            e.turns = e.turns.saturating_sub(1);
        }
        self.list.retain(|e| {
            if e.turns == 0 {
                report.expired.push(e.kind.label());
                false
            } else {
                true
            }
        });
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boosts_from_different_sources_stack() {
        let mut fx = Effects::default();
        fx.add(EffectSpec::new(EffectKind::Boost(Attribute::Reflexes), 2, 10), "juice");
        fx.add(EffectSpec::new(EffectKind::Boost(Attribute::Reflexes), -1, 10), "moonshine");
        assert_eq!(fx.attr_mod(Attribute::Reflexes), 1);
    }

    #[test]
    fn ailments_merge_and_expire() {
        let mut fx = Effects::default();
        fx.add(EffectSpec::new(EffectKind::Bleed, 1, 1), "a");
        fx.add(EffectSpec::new(EffectKind::Bleed, 2, 1), "b");
        let r = fx.tick();
        assert_eq!(r.damage, 2);
        assert_eq!(fx.tick().damage, 2);
        assert!(!fx.has(EffectKind::Bleed));
    }
}
