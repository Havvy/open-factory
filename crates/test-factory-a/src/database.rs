use bevy::prelude::*;
use open_factory::{kinds::{ItemKind, ItemKindBuilder, RecipeInput, RecipeKind, RecipeOutput}, local_string::LocalString, registry::Table};

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app
        .insert_resource(Table::<ItemKind>::new())
        .insert_resource(Table::<RecipeKind>::new())
        .add_startup_system_to_stage(StartupStage::Startup, setup.system())
        ;
    }
}

fn setup(
    mut items: ResMut<Table<ItemKind>>,
    mut recipes: ResMut<Table<RecipeKind>>,
) {
    let iron_plate = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("iron-plate")).build(), "iron-plate".to_string());
    let iron_pipe = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("iron-pipe")).build(), "iron-pipe".to_string());
    let copper = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("copper")).build(), "copper".to_string());
    let tin = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("tin")).build(), "tin".to_string());
    let bronze = items.insert(ItemKindBuilder::new().with_name(LocalString::from_str("bronze")).build(), "bronze".to_string());

    recipes.insert(RecipeKind {
        name: LocalString::from_str("recipe:generate-copper"),
        input_items: vec![],
        output: vec![RecipeOutput { item: copper, quantity: 1 }],
        time: 20,
    }, "generate-copper".to_string());

    recipes.insert(RecipeKind {
        name: LocalString::from_str("recipe:generate-tin"),
        input_items: vec![],
        output: vec![RecipeOutput { item: tin, quantity: 1 }],
        time: 20,
    }, "generate-tin".to_string());

    recipes.insert(RecipeKind {
        name: LocalString::from_str("recipe:bronze"),
        input_items: vec![RecipeInput { item: copper, quantity: 2 }, RecipeInput { item: tin, quantity: 1 }],
        output: vec![RecipeOutput { item: bronze, quantity: 3 }],
        time: 20,
    }, "bronze".to_string());

    recipes.insert(RecipeKind {
        name: LocalString::from_str("recipe:destroy-bronze"),
        input_items: vec![RecipeInput { item: bronze, quantity: 1 }],
        output: vec![],
        time: 5,
    }, "destroy-bronze".to_string());

    recipes.insert(RecipeKind {
        name: LocalString::from_str("recipe:iron-pipe"),
        input_items: vec![RecipeInput { item: iron_plate, quantity: 1 }],
        output: vec![RecipeOutput { item: iron_pipe, quantity: 1 }],
        time: 20,
    }, "iron-pipe".to_string());
}
