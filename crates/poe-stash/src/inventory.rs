//! Per-map loot capture via GGG character-inventory diffing (Phase 6.3).
//!
//! The character-window API returns the player's items; diffing two snapshots
//! taken around a map yields what dropped: brand-new items plus stack-size
//! growth on existing stacks (how stacked currency pickups are measured). Only
//! `MainInventory` items count — equipped gear and socketed gems are excluded so
//! gear/gem swaps aren't mistaken for loot. (Model: Exile Diary's InventoryGetter.)

use serde::Deserialize;
use std::collections::HashMap;

/// Minimal item shape from `character-window/get-items` needed for diffing.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct InventoryItem {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "typeLine", default)]
    pub type_line: String,
    #[serde(rename = "stackSize", default)]
    pub stack_size: Option<u32>,
    #[serde(rename = "inventoryId", default)]
    pub inventory_id: Option<String>,
    #[serde(rename = "frameType", default)]
    pub frame_type: Option<u32>,
}

/// A loot delta between two inventory snapshots.
#[derive(Debug, Clone, PartialEq)]
pub struct LootDelta {
    pub name: String,
    pub type_line: String,
    /// Count gained: full stack for a new item, the increase for a grown stack.
    pub stack_size: u32,
    pub frame_type: Option<u32>,
}

fn is_main_inventory(i: &InventoryItem) -> bool {
    // Equipped gear uses slot names (Weapon, Helm, Ring…); only loose items in
    // the backpack are MainInventory.
    i.inventory_id.as_deref() == Some("MainInventory")
}

/// Items gained between `prev` and `curr` (MainInventory only):
/// - new items (id absent from `prev`) → full stack size (1 if not stackable)
/// - grown stacks (curr stack > prev stack) → the delta
///
/// Removed items and equipment changes are ignored.
pub fn diff_inventory(prev: &[InventoryItem], curr: &[InventoryItem]) -> Vec<LootDelta> {
    let prev_by_id: HashMap<&str, &InventoryItem> = prev
        .iter()
        .filter(|i| is_main_inventory(i))
        .map(|i| (i.id.as_str(), i))
        .collect();

    let mut out = Vec::new();
    for item in curr.iter().filter(|i| is_main_inventory(i)) {
        let curr_stack = item.stack_size.unwrap_or(1);
        let gained = match prev_by_id.get(item.id.as_str()) {
            None => curr_stack,
            Some(prev_item) => {
                let prev_stack = prev_item.stack_size.unwrap_or(1);
                curr_stack.saturating_sub(prev_stack)
            }
        };
        if gained > 0 {
            out.push(LootDelta {
                name: item.name.clone(),
                type_line: item.type_line.clone(),
                stack_size: gained,
                frame_type: item.frame_type,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, name: &str, stack: Option<u32>, inv: &str) -> InventoryItem {
        InventoryItem {
            id: id.into(),
            name: name.into(),
            type_line: name.into(),
            stack_size: stack,
            inventory_id: Some(inv.into()),
            frame_type: Some(5),
        }
    }

    #[test]
    fn new_item_is_full_stack() {
        let prev = vec![];
        let curr = vec![item("a", "Chaos Orb", Some(5), "MainInventory")];
        let d = diff_inventory(&prev, &curr);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "Chaos Orb");
        assert_eq!(d[0].stack_size, 5);
    }

    #[test]
    fn grown_stack_is_the_delta() {
        let prev = vec![item("a", "Chaos Orb", Some(5), "MainInventory")];
        let curr = vec![item("a", "Chaos Orb", Some(12), "MainInventory")];
        let d = diff_inventory(&prev, &curr);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].stack_size, 7);
    }

    #[test]
    fn unchanged_and_removed_yield_nothing() {
        let prev = vec![
            item("a", "Chaos Orb", Some(5), "MainInventory"),
            item("b", "Divine Orb", Some(1), "MainInventory"),
        ];
        let curr = vec![item("a", "Chaos Orb", Some(5), "MainInventory")]; // b removed
        assert!(diff_inventory(&prev, &curr).is_empty());
    }

    #[test]
    fn equipped_items_are_ignored() {
        let prev = vec![];
        let curr = vec![
            item("w", "Some Sword", None, "Weapon"),
            item("g", "Awakened Gem", None, "Helm"),
            item("c", "Chaos Orb", Some(3), "MainInventory"),
        ];
        let d = diff_inventory(&prev, &curr);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].name, "Chaos Orb");
    }

    #[test]
    fn non_stackable_new_item_counts_as_one() {
        let curr = vec![item("r", "Rare Ring", None, "MainInventory")];
        let d = diff_inventory(&[], &curr);
        assert_eq!(d[0].stack_size, 1);
    }
}
