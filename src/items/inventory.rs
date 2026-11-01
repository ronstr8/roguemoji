//! The player's pack: a bounded list of item stacks.

use super::{Ammo, Component, Item, ItemKind};

pub const MAX_SLOTS: usize = 26;

#[derive(Clone, Debug, Default)]
pub struct Inventory {
    items: Vec<Item>,
}

impl Inventory {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn get(&self, idx: usize) -> Option<&Item> {
        self.items.get(idx)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Add an item, merging into an existing stack when possible.
    /// Returns the item back if the pack is full.
    pub fn add(&mut self, item: Item) -> Result<(), Item> {
        if item.stackable()
            && let Some(stack) = self.items.iter_mut().find(|s| s.kind == item.kind)
        {
            stack.qty += item.qty;
            return Ok(());
        }
        if self.items.len() >= MAX_SLOTS {
            return Err(item);
        }
        self.items.push(item);
        Ok(())
    }

    /// Remove up to `n` from the stack at `idx`.
    pub fn take(&mut self, idx: usize, n: u32) -> Option<Item> {
        let stack = self.items.get_mut(idx)?;
        if n >= stack.qty {
            return Some(self.items.remove(idx));
        }
        stack.qty -= n;
        Some(Item { kind: stack.kind.clone(), qty: n })
    }

    fn count(&self, kind: &ItemKind) -> u32 {
        self.items.iter().filter(|i| &i.kind == kind).map(|i| i.qty).sum()
    }

    /// Remove `n` of a stackable kind; fails without change if short.
    fn consume(&mut self, kind: &ItemKind, n: u32) -> bool {
        if self.count(kind) < n {
            return false;
        }
        if let Some(idx) = self.items.iter().position(|i| &i.kind == kind) {
            self.take(idx, n);
        }
        true
    }

    pub fn count_component(&self, c: Component) -> u32 {
        self.count(&ItemKind::Component(c))
    }

    pub fn consume_component(&mut self, c: Component, n: u32) -> bool {
        self.consume(&ItemKind::Component(c), n)
    }

    pub fn count_ammo(&self, a: Ammo) -> u32 {
        self.count(&ItemKind::Ammo(a))
    }

    pub fn consume_ammo(&mut self, a: Ammo, n: u32) -> bool {
        self.consume(&ItemKind::Ammo(a), n)
    }

    /// Ammo types carried that satisfy `pred`, in pack order.
    pub fn ammo_types(&self, pred: impl Fn(Ammo) -> bool) -> Vec<Ammo> {
        self.items
            .iter()
            .filter_map(|i| match i.kind {
                ItemKind::Ammo(a) if pred(a) => Some(a),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacks_merge_and_consume() {
        let mut inv = Inventory::default();
        inv.add(Item::component(Component::Nails, 2)).unwrap();
        inv.add(Item::component(Component::Nails, 3)).unwrap();
        assert_eq!(inv.len(), 1);
        assert!(inv.consume_component(Component::Nails, 4));
        assert_eq!(inv.count_component(Component::Nails), 1);
        assert!(!inv.consume_component(Component::Nails, 2));
        assert!(inv.consume_component(Component::Nails, 1));
        assert!(inv.is_empty());
    }
}
