use core::ops::Not;
use std::{collections::HashMap, convert::TryFrom, mem, ops::Range};

use serde::{Deserialize, Serialize};
use specs::{Component, DerefFlaggedStorage};
use specs_idvs::IdvStorage;
use tracing::{debug, trace, warn};

use crate::{
    comp::{
        inventory::{
            item::{ItemDef, MaterialStatManifest},
            loadout::Loadout,
            slot::{EquipSlot, Slot, SlotError},
        },
        slot::{InvSlotId, SlotId},
        Item,
    },
    recipe::{Recipe, RecipeInput},
    LoadoutBuilder,
};

pub mod item;
pub mod loadout;
pub mod loadout_builder;
pub mod slot;
#[cfg(test)] mod test;
#[cfg(test)] mod test_helpers;
pub mod trade_pricing;

pub type InvSlot = Option<Item>;
const DEFAULT_INVENTORY_SLOTS: usize = 18;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inventory {
    loadout: Loadout,
    /// The "built-in" slots belonging to the inventory itself, all other slots
    /// are provided by equipped items
    slots: Vec<InvSlot>,
}

/// Errors which the methods on `Inventory` produce
#[derive(Debug)]
pub enum Error {
    /// The inventory is full and items could not be added. The extra items have
    /// been returned.
    Full(Vec<Item>),
}

/// Represents the Inventory of an entity. The inventory has 18 "built-in"
/// slots, with further slots being provided by items equipped in the Loadout
/// sub-struct. Inventory slots are indexed by `InvSlotId` which is
/// comprised of `loadout_idx` - the index of the loadout item that provides the
/// slot, 0 being the built-in inventory slots, and `slot_idx` - the index of
/// the slot within that loadout item.
///
/// Currently, it is not supported for inventories to contain items that have
/// items inside them. This is due to both game balance purposes, and the lack
/// of a UI to show such items. Because of this, any action that would result in
/// such an item being put into the inventory (item pickup, unequipping an item
/// that contains items etc) must first ensure items are unloaded from the item.
/// This is handled in `inventory\slot.rs`
impl Inventory {
    pub fn new_empty() -> Inventory { Self::new_with_loadout(LoadoutBuilder::new().build()) }

    pub fn new_with_loadout(loadout: Loadout) -> Inventory {
        Inventory {
            loadout,
            slots: vec![None; DEFAULT_INVENTORY_SLOTS],
        }
    }

    /// Total number of slots in in the inventory.
    pub fn capacity(&self) -> usize { self.slots().count() }

    /// An iterator of all inventory slots
    pub fn slots(&self) -> impl Iterator<Item = &InvSlot> {
        self.slots
            .iter()
            .chain(self.loadout.inv_slots_with_id().map(|(_, slot)| slot))
    }

    /// A mutable iterator of all inventory slots
    fn slots_mut(&mut self) -> impl Iterator<Item = &mut InvSlot> {
        self.slots.iter_mut().chain(self.loadout.inv_slots_mut())
    }

    /// An iterator of all inventory slots and their position
    pub fn slots_with_id(&self) -> impl Iterator<Item = (InvSlotId, &InvSlot)> {
        self.slots
            .iter()
            .enumerate()
            .map(|(i, slot)| ((InvSlotId::new(0, u16::try_from(i).unwrap())), slot))
            .chain(
                self.loadout
                    .inv_slots_with_id()
                    .map(|(loadout_slot_id, inv_slot)| (loadout_slot_id.into(), inv_slot)),
            )
    }

    /// Adds a new item to the first fitting group of the inventory or starts a
    /// new group. Returns the item again if no space was found.
    pub fn push(&mut self, item: Item) -> Option<Item> {
        if item.is_stackable() {
            if let Some(slot_item) = self
                .slots_mut()
                .filter_map(Option::as_mut)
                .find(|s| *s == &item)
            {
                return slot_item
                    .increase_amount(item.amount())
                    .err()
                    .and(Some(item));
            }
        }

        // No existing item to stack with or item not stackable, put the item in a new
        // slot
        self.insert(item)
    }

    /// Add a series of items to inventory, returning any which do not fit as an
    /// error.
    pub fn push_all<I: Iterator<Item = Item>>(&mut self, items: I) -> Result<(), Error> {
        // Vec doesn't allocate for zero elements so this should be cheap
        let mut leftovers = Vec::new();
        for item in items {
            if let Some(item) = self.push(item) {
                leftovers.push(item);
            }
        }
        if !leftovers.is_empty() {
            Err(Error::Full(leftovers))
        } else {
            Ok(())
        }
    }

    /// Add a series of items to an inventory without giving duplicates.
    /// (n * m complexity)
    ///
    /// Error if inventory cannot contain the items (is full), returning the
    /// un-added items. This is a lazy inefficient implementation, as it
    /// iterates over the inventory more times than necessary (n^2) and with
    /// the proper structure wouldn't need to iterate at all, but because
    /// this should be fairly cold code, clarity has been favored over
    /// efficiency.
    pub fn push_all_unique<I: Iterator<Item = Item>>(&mut self, mut items: I) -> Result<(), Error> {
        let mut leftovers = Vec::new();
        for item in &mut items {
            if self.contains(&item).not() {
                self.push(item).map(|overflow| leftovers.push(overflow));
            } // else drop item if it was already in
        }
        if !leftovers.is_empty() {
            Err(Error::Full(leftovers))
        } else {
            Ok(())
        }
    }

    /// Replaces an item in a specific slot of the inventory. Returns the old
    /// item or the same item again if that slot was not found.
    pub fn insert_at(&mut self, inv_slot_id: InvSlotId, item: Item) -> Result<Option<Item>, Item> {
        match self.slot_mut(inv_slot_id) {
            Some(slot) => Ok(core::mem::replace(slot, Some(item))),
            None => Err(item),
        }
    }

    /// Merge the stack of items at src into the stack at dst if the items are
    /// compatible and stackable, and return whether anything was changed
    pub fn merge_stack_into(&mut self, src: InvSlotId, dst: InvSlotId) -> bool {
        let mut amount = None;
        if let (Some(srcitem), Some(dstitem)) = (self.get(src), self.get(dst)) {
            // The equality check ensures the items have the same definition, to avoid e.g.
            // transmuting coins to diamonds, and the stackable check avoids creating a
            // stack of swords
            if srcitem == dstitem && srcitem.is_stackable() {
                amount = Some(srcitem.amount());
            }
        }
        if let Some(amount) = amount {
            self.remove(src);
            let dstitem = self
                .get_mut(dst)
                .expect("self.get(dst) was Some right above this");
            dstitem
                .increase_amount(amount)
                .expect("already checked is_stackable");
            true
        } else {
            false
        }
    }

    /// Checks if inserting item exists in given cell. Inserts an item if it
    /// exists.
    pub fn insert_or_stack_at(
        &mut self,
        inv_slot_id: InvSlotId,
        item: Item,
    ) -> Result<Option<Item>, Item> {
        if item.is_stackable() {
            match self.slot_mut(inv_slot_id) {
                Some(Some(slot_item)) => {
                    Ok(if slot_item == &item {
                        slot_item
                            .increase_amount(item.amount())
                            .err()
                            .and(Some(item))
                    } else {
                        let old_item = core::mem::replace(slot_item, item);
                        // No need to recount--we know the count is the same.
                        Some(old_item)
                    })
                },
                Some(None) => self.insert_at(inv_slot_id, item),
                None => Err(item),
            }
        } else {
            self.insert_at(inv_slot_id, item)
        }
    }

    /// Attempts to equip the item into a compatible, unpopulated loadout slot.
    /// If no slot is available the item is returned.
    #[must_use = "Returned item will be lost if not used"]
    pub fn try_equip(&mut self, item: Item) -> Result<(), Item> { self.loadout.try_equip(item) }

    pub fn populated_slots(&self) -> usize { self.slots().filter_map(|slot| slot.as_ref()).count() }

    fn free_slots(&self) -> usize { self.slots().filter(|slot| slot.is_none()).count() }

    /// Check if an item is in this inventory.
    pub fn contains(&self, item: &Item) -> bool {
        self.slots().any(|slot| slot.as_ref() == Some(item))
    }

    /// Get content of a slot
    pub fn get(&self, inv_slot_id: InvSlotId) -> Option<&Item> {
        self.slot(inv_slot_id).and_then(Option::as_ref)
    }

    /// Mutably get content of a slot
    fn get_mut(&mut self, inv_slot_id: InvSlotId) -> Option<&mut Item> {
        self.slot_mut(inv_slot_id).and_then(Option::as_mut)
    }

    /// Returns a reference to the item (if any) equipped in the given EquipSlot
    pub fn equipped(&self, equip_slot: EquipSlot) -> Option<&Item> {
        self.loadout.equipped(equip_slot)
    }

    pub fn loadout_items_with_persistence_key(
        &self,
    ) -> impl Iterator<Item = (&str, Option<&Item>)> {
        self.loadout.items_with_persistence_key()
    }

    /// Returns the range of inventory slot indexes that a particular equipped
    /// item provides (used for UI highlighting of inventory slots when hovering
    /// over a loadout item)
    pub fn get_slot_range_for_equip_slot(&self, equip_slot: EquipSlot) -> Option<Range<usize>> {
        // The slot range returned from `Loadout` must be offset by the number of slots
        // that the inventory itself provides.
        let offset = self.slots.len();
        self.loadout
            .slot_range_for_equip_slot(equip_slot)
            .map(|loadout_range| (loadout_range.start + offset)..(loadout_range.end + offset))
    }

    /// Swap the items inside of two slots
    pub fn swap_slots(&mut self, a: InvSlotId, b: InvSlotId) {
        if self.slot(a).is_none() || self.slot(b).is_none() {
            warn!("swap_slots called with non-existent inventory slot(s)");
            return;
        }

        let slot_a = mem::take(self.slot_mut(a).unwrap());
        let slot_b = mem::take(self.slot_mut(b).unwrap());
        *self.slot_mut(a).unwrap() = slot_b;
        *self.slot_mut(b).unwrap() = slot_a;
    }

    /// Remove an item from the slot
    pub fn remove(&mut self, inv_slot_id: InvSlotId) -> Option<Item> {
        self.slot_mut(inv_slot_id).and_then(|item| item.take())
    }

    /// Remove just one item from the slot
    pub fn take(&mut self, inv_slot_id: InvSlotId, msm: &MaterialStatManifest) -> Option<Item> {
        if let Some(Some(item)) = self.slot_mut(inv_slot_id) {
            let mut return_item = item.duplicate(msm);

            if item.is_stackable() && item.amount() > 1 {
                item.decrease_amount(1).ok()?;
                return_item
                    .set_amount(1)
                    .expect("Items duplicated from a stackable item must be stackable.");
                Some(return_item)
            } else {
                self.remove(inv_slot_id)
            }
        } else {
            None
        }
    }

    /// Takes half of the items from a slot in the inventory
    pub fn take_half(
        &mut self,
        inv_slot_id: InvSlotId,
        msm: &MaterialStatManifest,
    ) -> Option<Item> {
        if let Some(Some(item)) = self.slot_mut(inv_slot_id) {
            if item.is_stackable() && item.amount() > 1 {
                let mut return_item = item.duplicate(msm);
                let returning_amount = item.amount() / 2;
                item.decrease_amount(returning_amount).ok()?;
                return_item
                    .set_amount(returning_amount)
                    .expect("Items duplicated from a stackable item must be stackable.");
                Some(return_item)
            } else {
                self.remove(inv_slot_id)
            }
        } else {
            None
        }
    }

    /// Takes all items from the inventory
    pub fn drain(&mut self) -> impl Iterator<Item = Item> + '_ {
        self.slots_mut()
            .filter(|x| x.is_some())
            .filter_map(mem::take)
    }

    /// Determine how many of a particular item there is in the inventory.
    pub fn item_count(&self, item_def: &ItemDef) -> u64 {
        self.slots()
            .flatten()
            .filter(|it| it.is_same_item_def(item_def))
            .map(|it| u64::from(it.amount()))
            .sum()
    }

    /// Determine whether the inventory contains the ingredients for a recipe.
    /// If it does, return a vector of numbers, where is number corresponds
    /// to an inventory slot, along with the number of items that need
    /// removing from it. It items are missing, return the missing items, and
    /// how many are missing.
    pub fn contains_ingredients<'a>(
        &self,
        recipe: &'a Recipe,
    ) -> Result<HashMap<InvSlotId, u32>, Vec<(&'a RecipeInput, u32)>> {
        let mut slot_claims = HashMap::<InvSlotId, u32>::new();
        let mut missing = Vec::<(&RecipeInput, u32)>::new();

        for (input, mut needed) in recipe.inputs() {
            let mut contains_any = false;

            for (inv_slot_id, slot) in self.slots_with_id() {
                if let Some(item) = slot
                    .as_ref()
                    .filter(|item| item.matches_recipe_input(&*input))
                {
                    let claim = slot_claims.entry(inv_slot_id).or_insert(0);
                    let can_claim = (item.amount() - *claim).min(needed);
                    *claim += can_claim;
                    needed -= can_claim;
                    contains_any = true;
                }
            }

            if needed > 0 || !contains_any {
                missing.push((input, needed));
            }
        }

        if missing.is_empty() {
            Ok(slot_claims)
        } else {
            Err(missing)
        }
    }

    /// Adds a new item to the first empty slot of the inventory. Returns the
    /// item again if no free slot was found.
    fn insert(&mut self, item: Item) -> Option<Item> {
        match self.slots_mut().find(|slot| slot.is_none()) {
            Some(slot) => {
                *slot = Some(item);
                None
            },
            None => Some(item),
        }
    }

    pub fn slot(&self, inv_slot_id: InvSlotId) -> Option<&InvSlot> {
        match SlotId::from(inv_slot_id) {
            SlotId::Inventory(slot_idx) => self.slots.get(slot_idx),
            SlotId::Loadout(loadout_slot_id) => self.loadout.inv_slot(loadout_slot_id),
        }
    }

    pub fn slot_mut(&mut self, inv_slot_id: InvSlotId) -> Option<&mut InvSlot> {
        match SlotId::from(inv_slot_id) {
            SlotId::Inventory(slot_idx) => self.slots.get_mut(slot_idx),
            SlotId::Loadout(loadout_slot_id) => self.loadout.inv_slot_mut(loadout_slot_id),
        }
    }

    /// Returns the number of free slots in the inventory ignoring any slots
    /// granted by the item (if any) equipped in the provided EquipSlot.
    pub fn free_slots_minus_equipped_item(&self, equip_slot: EquipSlot) -> usize {
        if let Some(mut equip_slot_idx) = self.loadout.loadout_idx_for_equip_slot(equip_slot) {
            // Offset due to index 0 representing built-in inventory slots
            equip_slot_idx += 1;

            self.slots_with_id()
                .filter(|(inv_slot_id, slot)| {
                    inv_slot_id.loadout_idx() != equip_slot_idx && slot.is_none()
                })
                .count()
        } else {
            // TODO: return Option<usize> and evaluate to None here
            warn!(
                "Attempted to fetch loadout index for non-existent EquipSlot: {:?}",
                equip_slot
            );
            0
        }
    }

    pub fn equipped_items(&self) -> impl Iterator<Item = &Item> { self.loadout.items() }

    /// Replaces the loadout item (if any) in the given EquipSlot with the
    /// provided item, returning the item that was previously in the slot.
    pub fn replace_loadout_item(
        &mut self,
        equip_slot: EquipSlot,
        replacement_item: Option<Item>,
    ) -> Option<Item> {
        self.loadout.swap(equip_slot, replacement_item)
    }

    /// Equip an item from a slot in inventory. The currently equipped item will
    /// go into inventory. If the item is going to mainhand, put mainhand in
    /// offhand and place offhand into inventory.
    #[must_use = "Returned items will be lost if not used"]
    pub fn equip(&mut self, inv_slot: InvSlotId) -> Vec<Item> {
        self.get(inv_slot)
            .and_then(|item| self.loadout.get_slot_to_equip_into(item.kind()))
            .map(|equip_slot| {
                // Special case when equipping into main hand - swap with offhand first
                if equip_slot == EquipSlot::Mainhand {
                    self.loadout
                        .swap_slots(EquipSlot::Mainhand, EquipSlot::Offhand);
                }

                self.swap_inventory_loadout(inv_slot, equip_slot)
            })
            .unwrap_or_else(Vec::new)
    }

    /// Determines how many free inventory slots will be left after equipping an
    /// item (because it could be swapped with an already equipped item that
    /// provides more inventory slots than the item being equipped)
    pub fn free_after_equip(&self, inv_slot: InvSlotId) -> i32 {
        let (inv_slot_for_equipped, slots_from_equipped) = self
            .get(inv_slot)
            .and_then(|item| self.loadout.get_slot_to_equip_into(item.kind()))
            .and_then(|equip_slot| self.equipped(equip_slot))
            .map_or((1, 0), |item| (0, item.slots().len()));

        let slots_from_inv = self
            .get(inv_slot)
            .map(|item| item.slots().len())
            .unwrap_or(0);

        i32::try_from(self.capacity()).expect("Inventory with more than i32::MAX slots")
            - i32::try_from(slots_from_equipped)
                .expect("Equipped item with more than i32::MAX slots")
            + i32::try_from(slots_from_inv).expect("Inventory item with more than i32::MAX slots")
            - i32::try_from(self.populated_slots())
                .expect("Inventory item with more than i32::MAX used slots")
            + inv_slot_for_equipped // If there is no item already in the equip slot we gain 1 slot  
    }

    /// Handles picking up an item, unloading any items inside the item being
    /// picked up and pushing them to the inventory to ensure that items
    /// containing items aren't inserted into the inventory as this is not
    /// currently supported.
    pub fn pickup_item(&mut self, mut item: Item) -> Result<(), Item> {
        if item.is_stackable() {
            return self.push(item).map_or(Ok(()), Err);
        }

        if self.free_slots() < item.populated_slots() + 1 {
            return Err(item);
        }

        // Unload any items contained within the item, and push those items and the item
        // itself into the inventory. We already know that there are enough free slots
        // so push will never give us an item back.
        item.drain().for_each(|item| {
            self.push(item).unwrap_none();
        });
        self.push(item).unwrap_none();

        Ok(())
    }

    /// Unequip an item from slot and place into inventory. Will leave the item
    /// equipped if inventory has no slots available.
    #[must_use = "Returned items will be lost if not used"]
    pub fn unequip(&mut self, equip_slot: EquipSlot) -> Result<Option<Vec<Item>>, SlotError> {
        // Ensure there is enough space in the inventory to place the unequipped item
        if self.free_slots_minus_equipped_item(equip_slot) == 0 {
            return Err(SlotError::InventoryFull);
        }

        Ok(self
            .loadout
            .swap(equip_slot, None)
            .and_then(|mut unequipped_item| {
                let unloaded_items: Vec<Item> = unequipped_item.drain().collect();
                self.push(unequipped_item)
                    .expect_none("Failed to push item to inventory, precondition failed?");

                // Unload any items that were inside the equipped item into the inventory, with
                // any that don't fit to be to be dropped on the floor by the caller
                match self.push_all(unloaded_items.into_iter()) {
                    Err(Error::Full(leftovers)) => Some(leftovers),
                    Ok(()) => None,
                }
            }))
    }

    /// Determines how many free inventory slots will be left after unequipping
    /// an item
    pub fn free_after_unequip(&self, equip_slot: EquipSlot) -> i32 {
        let (inv_slot_for_unequipped, slots_from_equipped) = self
            .equipped(equip_slot)
            .map_or((0, 0), |item| (1, item.slots().len()));

        i32::try_from(self.capacity()).expect("Inventory with more than i32::MAX slots")
            - i32::try_from(slots_from_equipped)
                .expect("Equipped item with more than i32::MAX slots")
            - i32::try_from(self.populated_slots())
                .expect("Inventory item with more than i32::MAX used slots")
            - inv_slot_for_unequipped // If there is an item being unequipped we lose 1 slot 
    }

    /// Swaps items from two slots, regardless of if either is inventory or
    /// loadout.
    #[must_use = "Returned items will be lost if not used"]
    pub fn swap(&mut self, slot_a: Slot, slot_b: Slot) -> Vec<Item> {
        match (slot_a, slot_b) {
            (Slot::Inventory(slot_a), Slot::Inventory(slot_b)) => {
                self.swap_slots(slot_a, slot_b);
                Vec::new()
            },
            (Slot::Inventory(inv_slot), Slot::Equip(equip_slot))
            | (Slot::Equip(equip_slot), Slot::Inventory(inv_slot)) => {
                self.swap_inventory_loadout(inv_slot, equip_slot)
            },
            (Slot::Equip(slot_a), Slot::Equip(slot_b)) => {
                self.loadout.swap_slots(slot_a, slot_b);
                Vec::new()
            },
        }
    }

    /// Determines how many free inventory slots will be left after swapping two
    /// item slots
    pub fn free_after_swap(&self, equip_slot: EquipSlot, inv_slot: InvSlotId) -> i32 {
        let (inv_slot_for_equipped, slots_from_equipped) = self
            .equipped(equip_slot)
            .map_or((0, 0), |item| (1, item.slots().len()));
        let (inv_slot_for_inv_item, slots_from_inv_item) = self
            .get(inv_slot)
            .map_or((0, 0), |item| (1, item.slots().len()));

        // Return the number of inventory slots that will be free once this slot swap is
        // performed
        i32::try_from(self.capacity())
            .expect("inventory with more than i32::MAX slots")
            - i32::try_from(slots_from_equipped)
            .expect("equipped item with more than i32::MAX slots")
            + i32::try_from(slots_from_inv_item)
            .expect("inventory item with more than i32::MAX slots")
            - i32::try_from(self.populated_slots())
            .expect("inventory with more than i32::MAX used slots")
            - inv_slot_for_equipped // +1 inventory slot required if an item was unequipped
            + inv_slot_for_inv_item // -1 inventory slot required if an item was equipped        
    }

    /// Swap item in an inventory slot with one in a loadout slot.
    #[must_use = "Returned items will be lost if not used"]
    pub fn swap_inventory_loadout(
        &mut self,
        inv_slot_id: InvSlotId,
        equip_slot: EquipSlot,
    ) -> Vec<Item> {
        if !self.can_swap(inv_slot_id, equip_slot) {
            return Vec::new();
        }

        // Take the item from the inventory
        let from_inv = self.remove(inv_slot_id);

        // Swap the equipped item for the item from the inventory
        let from_equip = self.loadout.swap(equip_slot, from_inv);

        let unloaded_items = from_equip
            .map(|mut from_equip| {
                // Unload any items held inside the previously equipped item
                let items: Vec<Item> = from_equip.drain().collect();

                // Attempt to put the unequipped item in the same slot that the inventory item
                // was in - if that slot no longer exists (because a large container was
                // swapped for a smaller one) then push the item to the first free
                // inventory slot instead.
                if let Err(returned) = self.insert_at(inv_slot_id, from_equip) {
                    self.push(returned)
                        .expect_none("Unable to push to inventory, no slots (bug in can_swap()?)");
                }

                items
            })
            .unwrap_or_default();

        // Attempt to put any items unloaded from the unequipped item into empty
        // inventory slots and return any that don't fit to the caller where they
        // will be dropped on the ground
        match self.push_all(unloaded_items.into_iter()) {
            Err(Error::Full(leftovers)) => leftovers,
            Ok(()) => Vec::new(),
        }
    }

    /// Determines if an inventory and loadout slot can be swapped, taking into
    /// account whether there will be free space in the inventory for the
    /// loadout item once any slots that were provided by it have been
    /// removed.
    pub fn can_swap(&self, inv_slot_id: InvSlotId, equip_slot: EquipSlot) -> bool {
        // Check if loadout slot can hold item
        if !self
            .get(inv_slot_id)
            .map_or(true, |item| equip_slot.can_hold(&item.kind()))
        {
            trace!("can_swap = false, equip slot can't hold item");
            return false;
        }

        // If we're swapping an equipped item with an empty inventory slot, make
        // sure  that there will be enough space in the inventory after any
        // slots  granted by the item being unequipped have been removed.
        if let Some(inv_slot) = self.slot(inv_slot_id) {
            if inv_slot.is_none() && self.free_slots_minus_equipped_item(equip_slot) == 0 {
                // No free inventory slots after slots provided by the equipped
                //item are discounted
                trace!("can_swap = false, no free slots minus item");
                return false;
            }
        } else {
            debug!(
                "can_swap = false, tried to swap into non-existent inventory slot: {:?}",
                inv_slot_id
            );
            return false;
        }

        true
    }
}

impl Component for Inventory {
    type Storage = DerefFlaggedStorage<Self, IdvStorage<Self>>;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InventoryUpdateEvent {
    Init,
    Used,
    Consumed(String),
    Gave,
    Given,
    Swapped,
    Dropped,
    Collected(Item),
    CollectFailed,
    Possession,
    Debug,
    Craft,
}

impl Default for InventoryUpdateEvent {
    fn default() -> Self { Self::Init }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InventoryUpdate {
    event: InventoryUpdateEvent,
}

impl InventoryUpdate {
    pub fn new(event: InventoryUpdateEvent) -> Self { Self { event } }

    pub fn event(&self) -> InventoryUpdateEvent { self.event.clone() }
}

impl Component for InventoryUpdate {
    type Storage = IdvStorage<Self>;
}
