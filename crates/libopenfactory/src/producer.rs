use crate::{item_stack::{InsertItemStackResult, ItemSlot, ItemStack, ItemStackQuanity}, kinds::{RecipeInput, RecipeKind, RecipeOutput}, registry::{Handle, Table}};

pub struct Producer {
    recipe: Option<Handle<RecipeKind>>,
    input_slots: Vec<ItemSlot>,
    pub output_slots: Vec<ItemSlot>,
    production: ProductionState,
}

enum ProductionState {
    /// Waiting on the required inputs.
    Idle,

    /// Currently producing the output.
    Producing {
        progress: f32, 
        time: crate::Time,
    },

    #[allow(dead_code)]
    /// Waiting for the output to have more capacity.
    Full,
}

impl Default for ProductionState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProductionStatus {
    Idle,
    Producing,
    Full,
}

impl From<&'_ ProductionState> for ProductionStatus {
    fn from(state: &ProductionState) -> ProductionStatus {
        match *state {
            ProductionState::Idle => ProductionStatus::Idle,
            ProductionState::Producing{..} => ProductionStatus::Producing,
            ProductionState::Full => ProductionStatus::Full,
        }
    }
} 

/// Constructors
impl Producer {
    pub fn new() -> Self {
        Self {
            recipe: Default::default(),
            production: Default::default(),
            input_slots: vec![],
            output_slots: vec![],
        }
    }

    pub fn new_with_recipe(recipe: &RecipeKind, handle: Handle<RecipeKind>) -> Self {
        let mut producer = Self::new();
        producer.set_recipe(recipe, handle);
        producer
    }
}

impl Default for Producer {
    fn default() -> Self {
        Self::new()
    }
}

impl Producer {
    pub fn set_recipe(&mut self, recipe: &RecipeKind, handle: Handle<RecipeKind>) {
        self.input_slots = recipe.input_items
        .iter()
        .map(<ItemSlot as From<&RecipeInput>>::from)
        .collect();

        self.output_slots = recipe.output
        .iter()
        .map(<ItemSlot as From<&RecipeOutput>>::from)
        .collect();

        self.recipe = Some(handle);
    }

    pub fn tick(&mut self, recipes: &Table<RecipeKind>) {
        let mut reset_production = false;

        if let ProductionState::Producing { progress, time } = &mut self.production {
            let production_completion = {
                const PROGRESS_DIFF: f32 = 1.0 / (24.0 * 3600.0 * 20.0);

                *progress += 1.0 / *time as f32;
                *progress >= 1.0 - PROGRESS_DIFF
            };

            if production_completion {
                Iterator::zip(
                    recipes[self.recipe.unwrap()].output.iter(),
                    &mut self.output_slots
                )
                .for_each(|(recipe_output, output_item_slot) | {
                    let _ = output_item_slot.insert_item_stack(ItemStack::from_recipe_output(recipe_output));
                });

                reset_production = true;
            }
        }

        if reset_production {
            self.production = ProductionState::Idle;
            self.attempt_to_start_production(recipes);
        }
    }

    pub fn try_insert_ingredient(&mut self, stack: ItemStack) -> InsertItemStackResult {
        let ingredient_slot = self.input_slots.iter_mut().find(|slot| slot.filter == Some(stack.item));

        if let Some(ingredient_slot) = ingredient_slot {
            ingredient_slot.insert_item_stack(stack)
        } else {
            InsertItemStackResult::FilterError(stack)
        }
    }

    /// Take a single item from the first output slot that has an item in it.
    ///
    /// # Panics
    ///
    /// If the producer doesn't have an item slot with an item in it,
    /// this function panics. You can call [`has_output`] first to check.
    pub fn take_single_item(&mut self) -> ItemStack {
        self.output_slots
        .iter_mut()
        .filter(|slot| !slot.is_empty())
        .map(|slot| slot.take_single_item())
        .next()
        .flatten()
        .expect("At least one non-empty item slot exists.")
    }

    pub fn attempt_to_start_production(&mut self, recipes: &Table<RecipeKind>) {
        if !self.can_start_production(recipes) {
            if let Some(recipe_handle) = self.recipe {
                if self.is_output_full(&recipes[recipe_handle]) {
                    self.production = ProductionState::Full;
                }
            }
            return;
        }

        let recipe = self.recipe.expect("Recipe must exist if can start production is true");

        // Consume inputs.
        Iterator::zip(
            self.input_slots.iter_mut(), 
            &recipes[recipe].input_items
        )
        .for_each(|(item_slot, recipe_input)| {
            item_slot.destroy_quantity(recipe_input.quantity);
        });

        self.production = ProductionState::Producing {
            progress: 0.0,
            time: recipes[recipe].time,
        };
    }
}

/// Queries
impl Producer {
    pub fn is_producing(&self) -> bool {
        matches!(self.production, ProductionState::Producing{ .. })
    }

    pub fn has_output(&self) -> bool {
        self.output_slots.iter().any(|slot| {
            !slot.is_empty()
        })
    }

    /// Whether or not the producer has input items slots.
    ///
    /// If it doesn't, that means that the recipe will always run as long
    /// as there is output space available.
    pub fn takes_input(&self) -> bool {
        !self.input_slots.is_empty()
    }

    /// Whether or not the producer has output item slots.
    ///
    /// If it doesn't, the recipe consumes the input items without actually
    /// producing anything useful. This is useful if you want a recipe
    /// that destroys something.
    pub fn gives_output(&self) -> bool {
        !self.output_slots.is_empty()
    }

    pub fn status(&self) -> ProductionStatus {
        (&self.production).into()
    }

    /// Returns if the output of the recipe would not fit in the
    /// output slots of the producer.
    fn is_output_full(&self, recipe: &RecipeKind) -> bool {
        Iterator::zip(
            self.output_slots.iter(),
            &recipe.output
        )
        .map(|(item_slot, output)| (item_slot.available_capacity(), output.quantity))
        .any(|(available_capacity, output_quantity)| available_capacity < output_quantity)
    }

    pub fn item_counts(&self) -> (Vec<(ItemStackQuanity, ItemStackQuanity)>, Vec<(ItemStackQuanity, ItemStackQuanity)>) {
        let input_stacks = self.input_slots.iter().map(|slot| (slot.quantity(), slot.capacity)).collect();
        let output_stacks = self.output_slots.iter().map(|slot| (slot.quantity(), slot.capacity)).collect();

        (input_stacks, output_stacks)
    }

    fn can_start_production(&self, recipes: &Table<RecipeKind>) -> bool {
        // Production is already started.
        if self.is_producing() {
            return false;
        }

        let recipe = match self.recipe {
            Some(recipe) => recipe,
            _ => return false,
        };

        if self.recipe.is_none() {
            return false;
        }

        // Production must have enough ingredients to start.
        if
            Iterator::zip(
                self.input_slots.iter(),
                &recipes[recipe].input_items
            )
            .map(|(item_slot, recipe_input)| (item_slot.quantity(), recipe_input.quantity))
            .any(|(item_slot_quantity, recipe_input_quantity)| item_slot_quantity < recipe_input_quantity)
        {
            return false;
        }

        // Output must not be full.
        if self.is_output_full(&recipes[recipe]) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{kinds::{ItemKind, ItemKindBuilder}, local_string::LocalString};

    fn make_item(items: &mut Table<ItemKind>, name: &str) -> Handle<ItemKind> {
        items
        .insert(
            ItemKindBuilder::new()
            .with_name(LocalString::from_str(name))
            .build(),

            name.to_string(),
        )
    }

    #[test]
    fn assemble() {
        let mut items = Table::new();
        let iron_plate = make_item(&mut items, "iron-plate");
        let iron_pipe = make_item(&mut items, "iron-pipe");

        let mut recipes = Table::new(); 

        let make_iron_pipe = recipes.insert(RecipeKind {
            name: LocalString::from_str("iron-pipe"),
            input_items: vec![RecipeInput { item: iron_plate, quantity: 1 }],
            output: vec![RecipeOutput { item: iron_pipe, quantity: 1 }],
            time: 20,
        }, "iron-pipe".to_string());

        let mut producer = Producer::new();
        producer.set_recipe(&recipes[make_iron_pipe.clone()], make_iron_pipe);

        let iron_plate_stack = ItemStack {
            item: iron_plate,
            quantity: 1,
        };

        let insert_result = producer.try_insert_ingredient(iron_plate_stack);
        assert!(matches!(insert_result, InsertItemStackResult::StackConsumed));

        producer.attempt_to_start_production(&recipes);

        for _ in 0..19 {
            producer.tick(&recipes);
        }

        assert_eq!(0, producer.output_slots[0].quantity());

        producer.tick(&recipes);

        assert_eq!(1, producer.output_slots[0].quantity());
        assert!(!producer.is_producing());

        assert_eq!(ProductionStatus::Idle, producer.status());

        let iron_plate_stack = ItemStack {
            item: iron_plate,
            quantity: 1,
        };

        let insert_result = producer.try_insert_ingredient(iron_plate_stack);
        assert!(matches!(insert_result, InsertItemStackResult::StackConsumed));

        producer.attempt_to_start_production(&recipes);

        for _ in 0..20 {
            producer.tick(&recipes);
        }
        

        assert_eq!(2, producer.output_slots[0].quantity());
        assert_eq!(ProductionStatus::Full, producer.status());
    }

    #[test]
    fn producer_without_inputs_does_not_take_input() {
        let mut items = Table::new();
        let test_item = make_item(&mut items, "test-item");

        let mut recipes = Table::new();

        let generate_test_item = recipes.insert(RecipeKind {
            name: LocalString::from_str("generate-test-item"),
            input_items: vec![],
            output: vec![RecipeOutput { item: test_item, quantity: 1}],
            time: 20,
        }, "generate-test-item".to_string());

        let producer = Producer::new_with_recipe(&recipes[generate_test_item], generate_test_item);
        assert!(!producer.takes_input());
    }

    #[test]
    #[ignore] // was trying to test why connectors aren't working after placing a second one,
    // and thought it was in the item slots. But it's not.
    fn producer_with_multiple_inputs() {
        let mut items = Table::new();
        let item_1 = make_item(&mut items, "1");
        let item_2 = make_item(&mut items, "2");
        let item_out = make_item(&mut items, "out");

        let mut recipes = Table::new();
        let out_recipe = recipes.insert(RecipeKind {
            name: LocalString::from_str("out"),
            input_items: vec![
                RecipeInput {
                    item: item_1,
                    quantity: 1
                },
                RecipeInput {
                    item: item_2,
                    quantity: 2
                },
            ],
            output: vec![RecipeOutput {
                item: item_out,
                quantity: 3
            }],

            time: 20,
        }, "out".into());

        let mut producer = Producer::new_with_recipe(&recipes[out_recipe], out_recipe);

        let _ = producer.try_insert_ingredient(ItemStack {
            item: item_1,
            quantity: 1,
        });

        let _ = producer.try_insert_ingredient(ItemStack {
            item: item_2,
            quantity: 1,
        });

        let res = producer.try_insert_ingredient(ItemStack {
            item: item_1,
            quantity: 1,
        });

        println!("{:?}", res);
        panic!();
    }
}