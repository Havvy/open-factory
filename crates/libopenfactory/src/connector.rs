//! Connection point between output slots and input slots of things.

use crate::item_stack::{InsertItemStackResult, ItemSlot, ItemSlotBuilder, ItemStack};

/// Whether the connector is traveling towards its input slot or output slot.
#[derive(Debug)]
pub enum ConnectorDirection {
    Input,
    Output,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectorStatus {
    WaitingOnInput,
    WaitingOnOutput,
    Traveling,
}

#[derive(Debug)]
pub struct Connector {
    direction: ConnectorDirection,

    /// How long the connector is between the input and output.
    length: f32,
    position: f32,

    item: ItemSlot,
}

impl Connector {
    /// How far the connector moves in one frame.
    const SPEED: f32 = 5.0;

    pub fn new(length: f32) -> Self {
        Self {
            direction: ConnectorDirection::Input,
            length,
            position: 0.0,
            item: ItemSlotBuilder::new().with_capacity(1).build(),
        }
    }

    pub fn status(&self) -> ConnectorStatus {
        if self.position == 0.0 && self.item.is_empty() {
            ConnectorStatus::WaitingOnInput
        } else if self.position == self.length && !self.item.is_empty() {
            ConnectorStatus::WaitingOnOutput
        } else {
            ConnectorStatus::Traveling
        }
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn tick(&mut self) {
        if self.status() != ConnectorStatus::Traveling {
            return;
        }

        match self.direction {
            ConnectorDirection::Input => {
                self.position = f32::max(0.0, self.position - Self::SPEED);
            },

            ConnectorDirection::Output => {
                self.position = f32::min(self.length, self.position + Self::SPEED);
            }
        }
    }

    pub fn insert_stack(&mut self, stack: ItemStack) -> InsertItemStackResult {
        // Can be called putting the itemstack back into the connector after taking it out at output.
        // As such,this debug_assert! is bad.
        // debug_assert!(matches!(self.status(), ConnectorStatus::WaitingOnInput));

        let res = self.item.insert_item_stack(stack);

        if res.is_change() {
            self.direction = ConnectorDirection::Output;
        }

        res
    }

    pub fn take_stack(&mut self) -> ItemStack {
        self.direction = ConnectorDirection::Input;
        self.item.stack.take().unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::{kinds::ItemKindBuilder, local_string::LocalString, registry::Table};

    use super::*;

    #[test]
    fn test_connector() {
        let mut connector: Connector = Connector::new(50.0);

        assert_eq!(connector.status(), ConnectorStatus::WaitingOnInput);

        let mut items = Table::new();
        let item = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("item")).build(), "item".to_string());

        let stack = ItemStack { item, quantity: 1 };

        let stack_insert_result = connector.insert_stack(stack);
        assert!(matches!(stack_insert_result, InsertItemStackResult::StackConsumed));

        for _ in 0..10 {
            // This assertion is before the tick because we want to check its state
            // before moving.
            assert_eq!(connector.status(), ConnectorStatus::Traveling);
            connector.tick();
        }

        assert_eq!(connector.status(), ConnectorStatus::WaitingOnOutput);

        let _ = connector.take_stack();
        
        for _ in 0..10 {
            // Same as before, but the opposite direction.
            assert_eq!(connector.status(), ConnectorStatus::Traveling);
            connector.tick();
        }

        // And to complete the trip!
        assert_eq!(connector.status(), ConnectorStatus::WaitingOnInput);
    }
}