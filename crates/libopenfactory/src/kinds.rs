use crate::{Time, item_stack::{ItemSlot, ItemStackQuanity}, local_string::LocalString, registry::Handle};

#[derive(Debug, PartialEq, Eq)]
pub struct ItemKind {
    name: LocalString,
    stack_size: u16,
}

pub struct ItemKindBuilder {
    name: Option<LocalString>,
    stack_size: u16,
}

impl ItemKindBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            stack_size: 50,
        }
    }

    pub fn with_name(mut self, name: LocalString) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> ItemKind {
        match self {
            ItemKindBuilder { name: Some(name), stack_size } => {
                ItemKind { name, stack_size }
            },

            _ => panic!("Item Kind Builder built without all required fields")
        }
    }
}

pub struct RecipeKind {
    pub name: LocalString,
    pub input_items: Vec<RecipeInput>,
    pub output: Vec<RecipeOutput>,
    pub time: Time,
}

pub struct RecipeInput {
    pub item: Handle<ItemKind>,
    pub quantity: ItemStackQuanity,
}

pub struct RecipeOutput {
    pub item: Handle<ItemKind>,
    pub quantity: ItemStackQuanity,
}

impl From<RecipeInput> for ItemSlot {
    fn from(input: RecipeInput) -> Self {
        ItemSlot {
            stack: None,
            capacity: input.quantity * 2,
            filter: Some(input.item),
        }
    }
}

impl From<RecipeOutput> for ItemSlot {
    fn from(output: RecipeOutput) -> Self {
        ItemSlot {
            stack: None,
            capacity: output.quantity * 2,
            filter: Some(output.item),
        }
    }
}

impl From<&RecipeInput> for ItemSlot {
    fn from(input: &RecipeInput) -> Self {
        ItemSlot {
            stack: None,
            capacity: input.quantity * 2,
            filter: Some(input.item),
        }
    }
}

impl From<&RecipeOutput> for ItemSlot {
    fn from(output: &RecipeOutput) -> Self {
        ItemSlot {
            stack: None,
            capacity: output.quantity * 2,
            filter: Some(output.item),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::registry::Table;
    use super::*;
    
    #[test]
    fn make_a_recipe() {
        let mut items = Table::new();
        let input_kind = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("input")).build(), "input".to_string());
        let output_kind = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("output")).build(), "output".to_string());

        let _recipe = RecipeKind {
            name: LocalString::from_str("generic-recipe"),
            input_items: vec![RecipeInput { item: input_kind, quantity: 1 }],
            output: vec![RecipeOutput { item: output_kind, quantity: 1 }],
            time: 20,
        };
    }
}