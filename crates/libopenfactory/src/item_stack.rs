use crate::{kinds::{ItemKind, RecipeOutput}, registry::Handle};

/// Maximum number of items allowed in a single item stack.
pub type ItemStackQuanity = u16;

pub struct ItemSlotBuilder {
    capacity: ItemStackQuanity,
    filter: Option<Handle<ItemKind>>,
}

impl ItemSlotBuilder {
    pub fn new() -> Self {
        Self {
            capacity: ItemStackQuanity::MAX,
            filter: None,
        }
    }

    pub fn with_capacity(mut self, capacity: ItemStackQuanity) -> Self {
        self.capacity = capacity;
        self
    }

    pub fn with_filter(mut self, filter: Handle<ItemKind>) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn build(self) -> ItemSlot {
        ItemSlot {
            capacity: self.capacity,
            filter: self.filter,
            stack: None,
        }
    }
}

#[derive(Debug)]
pub struct ItemSlot {
    pub stack: Option<ItemStack>,
    pub(crate) capacity: ItemStackQuanity,
    pub(crate) filter: Option<Handle<ItemKind>>
}

impl ItemSlot {
    pub fn new(filter: Option<Handle<ItemKind>>) -> Self {
        Self {
            stack: None,
            capacity: ItemStackQuanity::MAX,
            filter,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_none()
    }
    
    /// Amount of items that can be added to this slot before it no longer accepts more.
    pub fn available_capacity(&self) -> ItemStackQuanity {
        self.capacity - self.quantity()
    }

    /// Amount of items held in the slot.
    pub fn quantity(&self) -> ItemStackQuanity {
        self.stack.as_ref().map(|stack| stack.quantity).unwrap_or_default()
    }

    pub fn take_single_item(&mut self) -> Option<ItemStack> {
        if let Some(stack) = self.stack.take() {
            let (new_stack, single_stack) = stack.split_single();
            self.stack = new_stack;
            Some(single_stack)
        } else {
            None
        }
    }

    pub fn insert_item_stack(&mut self, mut stack: ItemStack) -> InsertItemStackResult {
        use InsertItemStackResult::*;

        // If the item slot is filtered, the item stack must actually be same item.
        if let Some(filter) = self.filter {
            if stack.item != filter {
                return FilterError(stack);
            }
        }

        // If the item slot has a stack already, the item type must match.
        if let Some(slot_stack) = &self.stack {
            if slot_stack.item != stack.item {
                return ItemSlotTaken(stack);
            }
        }

        let stack_consumption = std::cmp::min(stack.quantity, self.available_capacity());

        if stack_consumption == 0 {
            return ItemSlotFull(stack);
        }

        stack.quantity -= stack_consumption;
        let slot_stack = self.stack.get_or_insert(ItemStack { item: stack.item, quantity: 0 });
        slot_stack.quantity += stack_consumption;

        if stack.quantity == 0 {
            StackConsumed
        } else {
            StackPartiallyConsumed(stack)
        }
    }

    pub fn destroy_quantity(&mut self, quantity: ItemStackQuanity) {
        let slot_stack = self.stack
        .as_mut()
        .expect("ItemSlot must have a stack in it.");

        if slot_stack.quantity == quantity {
            self.stack = None;
        } else {
            slot_stack.quantity -= quantity;
        }
    }
}

#[must_use]
#[derive(Debug)]
pub enum InsertItemStackResult {
    StackConsumed,
    StackPartiallyConsumed(ItemStack),
    FilterError(ItemStack),
    ItemSlotTaken(ItemStack),
    ItemSlotFull(ItemStack),
}

impl InsertItemStackResult {
    pub fn get_item_stack(self) -> Option<ItemStack> {
        match self {
            Self::StackConsumed => None,

            | Self::StackPartiallyConsumed(stack)
            | Self::FilterError(stack)
            | Self::ItemSlotTaken(stack)
            | Self::ItemSlotFull(stack)
            => Some(stack)
        }
    }

    pub fn is_change(&self) -> bool {
        match *self {
            | Self::StackConsumed
            | Self::StackPartiallyConsumed(..)
            => true,

            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct ItemStack {
    pub item: Handle<ItemKind>,
    pub quantity: ItemStackQuanity,
}

impl ItemStack {
    pub fn from_recipe_output(recipe_output: &RecipeOutput) -> Self {
        Self {
            item: recipe_output.item,
            quantity: recipe_output.quantity,
        }
    }

    pub fn split_single(mut self) -> (Option<Self>, Self) {
        if self.quantity == 1 {
            return (None, self);
        }

        let single_stack = ItemStack {
            item: self.item,
            quantity: 1,
        };

        self.quantity -= 1;

        (Some(self), single_stack)
    }
}