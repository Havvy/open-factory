use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use open_factory::item_stack::ItemStackQuanity;
use open_factory::registry::Table;
use open_factory::producer::{Producer, ProductionStatus};
use open_factory::kinds::RecipeKind;
use open_factory::connector::{Connector, ConnectorStatus};

use crate::ui::Action;
use crate::mouse_interaction::{Extents, MouseInteraction};
use crate::camera::MousePositionInWorld;
use crate::Colors;
use crate::tick::Tick;

pub struct FactoryProducerPlugin;

impl Plugin for FactoryProducerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .insert_resource::<Option<PartialConnector>>(None)
        .add_system(producer_tick_system.system())
        .add_system(producer_color_system.system())
        .add_system(click_system.system())
        .add_system(connector_transfer_system.system())
        .add_system(producer_entry_exit_color_system.system())
        .add_system(producer_io_count_text_system.system())
        ;
    }
}

#[derive(Debug)]
struct PartialConnector {
    giver: Entity,
    giver_position: GlobalTransform,
}

// This is one massive system because prototype.
// It would probably be more editable if it wasn't a single massive function.
//
// For what it's worth, what it does is check if the mouse is in a window,
// and if it is, if a button is released. If the left click is released,
// it compares against the Action defined in `ui.rs` and does the logic for
// that. For most actions, it just places a producer in the world tied to
// a specific recipe. For `Connect`, check the comment near it's match arm.
fn click_system(
    mut commands: Commands,
    action: Res<Action>,
    mouse_button_input: Res<Input<MouseButton>>,
    mouse_position: Res<Option<MousePositionInWorld>>,
    recipes: Res<Table<RecipeKind>>,
    colors: Res<Colors>,
    font: Res<crate::GameFont>,
    mut partial_connector: ResMut<Option<PartialConnector>>,
    mut connector_giver_query: Query<(Entity, &MouseInteraction, &GlobalTransform, &mut ConnectorGiver, &mut Handle<ColorMaterial>)>,
    mut connector_taker_query: Query<(Entity, &MouseInteraction, &GlobalTransform, &mut ConnectorTaker,)>,
) {
    let mouse_position = if let Some(ref mouse_position) = *mouse_position {
        mouse_position
    } else {
        return;
    };

    if mouse_button_input.just_released(MouseButton::Left) {
        match *action {
            Action::Connect => {
                match partial_connector.take() {
                    // Has not previously clicked on a giver.
                    None => {
                        let possibly_clicked_giver = connector_giver_query
                        .iter_mut()
                        .filter(|(_, mouse_interaction, _, _, _,)| {
                            **mouse_interaction == MouseInteraction::Hovered
                        })
                        .next();

                        match possibly_clicked_giver {
                            None => {
                                eprintln!("Not hovering over a connection giver!");
                                return;
                            },

                            Some((entity, _, position, _, mut color)) => {
                                *color = colors.grey.clone();
                                *partial_connector = Some(PartialConnector { giver: entity, giver_position: *position });
                            }
                        }
                    },

                    // Has previously clicked on a giver.
                    Some(PartialConnector { giver, giver_position: giver_transform }) => {
                        let possibly_clicked_taker = connector_taker_query
                        .iter_mut()
                        .filter(|(_, mouse_interaction, _, _,)| {
                            **mouse_interaction == MouseInteraction::Hovered
                        })
                        .next();

                        let (taker, taker_transform) = 
                        if let Some((entity, &MouseInteraction::Hovered, position, _)) = possibly_clicked_taker {
                            (entity, position)
                        } else {
                            eprintln!("Not hovering over a production entry!");
                            return;
                        };

                        // Set all connector givers to white, unconditionally.
                        // Yes, this is wasteful.
                        connector_giver_query.iter_mut().for_each(|(_, _, _, _, mut color,)| {
                            *color = colors.white.clone();
                        });

                        let connector_vector = taker_transform.translation.xy() - giver_transform.translation.xy();
                        let connector_length = dbg!(connector_vector.length());
                        let angle = connector_vector.angle_between(Vec2::X);
                        let center = (giver_transform.translation + taker_transform.translation) / 2.0;

                        let connector = Connector::new(connector_length);

                        // The position is the center point between the 
                        let mut connector_position = Transform::from_translation(center);
                        connector_position.rotate(Quat::from_rotation_z(-angle));

                        let connector_entity = commands
                        .spawn_bundle(SpriteBundle {
                            transform: connector_position,
                            material: colors.black.clone(),
                            sprite: Sprite::new(Vec2::new(connector_length, 3.0)),
                            ..Default::default()
                        })
                        .insert(connector)
                        .with_children(|parent| {
                            parent.spawn_bundle(SpriteBundle {
                                transform: Transform::from_xyz(-connector_length / 2.0, 0.0, 4.0),
                                material: colors.blue.clone(),
                                sprite: Sprite::new(Vec2::new(3.0, 16.0)),
                                ..Default::default()
                            })
                            .insert(ConnectorLine)
                            ;
                        })
                        .id();

                        connector_taker_query.get_component_mut::<ConnectorTaker>(taker).unwrap().takers.push(connector_entity);
                        connector_giver_query.get_component_mut::<ConnectorGiver>(giver).unwrap().givers.push(connector_entity);
                        *partial_connector = Default::default();
                    },
                }
            },
            Action::Copper => {
                let (recipe, handle) = recipes.get_ref_and_handle_from_name("generate-copper");
                let producer = Producer::new_with_recipe(recipe, handle);
                spawn_producer(&mut commands, &*colors, mouse_position.transform(), producer, font.0.clone(), "Copper")
            },

            Action::Tin => {
                let (recipe, handle) = recipes.get_ref_and_handle_from_name("generate-tin");
                let producer = Producer::new_with_recipe(recipe, handle);
                spawn_producer(&mut commands, &*colors, mouse_position.transform(), producer, font.0.clone(), "Tin")
            },

            Action::Bronze => {
                let (recipe, handle) = recipes.get_ref_and_handle_from_name("bronze");
                let producer = Producer::new_with_recipe(recipe, handle);
                spawn_producer(&mut commands, &*colors, mouse_position.transform(), producer, font.0.clone(), "Bronze")
            },

            Action::Trash => {
                let (recipe, handle) = recipes.get_ref_and_handle_from_name("destroy-bronze");
                let producer = Producer::new_with_recipe(recipe, handle);
                spawn_producer(&mut commands, &*colors, mouse_position.transform(), producer, font.0.clone(), "Delete Bronze")
            },
        }

        // Undo setting a partial connection. It doesn't matter which UI mode we are in
        // because only this mode has any state and resetting doesn't crate/destroy data.
        if mouse_button_input.just_released(MouseButton::Right) {
            *partial_connector = None;

            // Set all connector exits to white, unconditionally.
            // Yes, this is wasteful.
            connector_giver_query.iter_mut().for_each(|(_, _, _, _, mut color,)| {
                *color = colors.white.clone();
            });
        }
    }
}

struct ProducerIOCountText;

fn spawn_producer(commands: &mut Commands, colors: &Colors, location: Transform, producer: Producer, font: Handle<Font>, label: &str) {
    let takes_input = producer.takes_input();
    let gives_output = producer.gives_output();

    commands
    .spawn_bundle(SpriteBundle {
        material: colors.red.clone(),
        transform: location,
        sprite: Sprite::new(Vec2::new(120.0, 30.0)),
        ..Default::default()
    })
    .insert(Extents(Vec2::new(60.0, 15.0)))
    .insert(MouseInteraction::default())
    .with_children(|parent| {
        parent.spawn_bundle(Text2dBundle {
            text: Text::with_section(
                label.to_string(), 
                TextStyle {
                    font: font.clone(),
                    font_size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },
                TextAlignment {
                    vertical: VerticalAlign::Top,
                    horizontal: HorizontalAlign::Right,
                },
            ),

            transform: Transform::from_xyz(-35.0, -10.0, 1.0),

            ..Default::default()
        });

        let (input_text, output_text) = producer_counts(&producer);

        parent.spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: input_text,
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 12.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    },
                    TextSection {
                        value: " | ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 12.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    },
                    TextSection {
                        value: output_text,
                        style: TextStyle {
                            font,
                            font_size: 12.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        }
                    },
                ],
                alignment: TextAlignment {
                    vertical: VerticalAlign::Top,
                    horizontal: HorizontalAlign::Right,
                },
                ..Default::default()
            },
            transform: Transform::from_xyz(-35.0, 2.0, 1.0),

            ..Default::default()
        })
        .insert(ProducerIOCountText);

        if takes_input {
            parent.spawn_bundle(SpriteBundle {
                material: colors.white.clone(),
                transform: Transform::from_xyz(-45.0, 0.0, 2.0),
                sprite: Sprite::new(Vec2::new(15.0, 15.0)),
                ..Default::default()
            })
            .insert(ConnectorTaker::new())
            .insert(Extents(Vec2::new(15.0, 15.0)))
            .insert(MouseInteraction::default())
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    material: colors.black.clone(),
                    sprite: Sprite::new(Vec2::new(17.0, 17.0)),
                    ..Default::default()
                });
            })
            ;
        }

        if gives_output {
            parent.spawn_bundle(SpriteBundle {
                material: colors.white.clone(),
                transform: Transform::from_xyz(45.0, 0.0, 2.0),
                sprite: Sprite::new(Vec2::new(15.0, 15.0)),
                ..Default::default()
            })
            .insert(ConnectorGiver::new())
            .insert(Extents(Vec2::new(15.0, 15.0)))
            .insert(MouseInteraction::default())
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    material: colors.black.clone(),
                    sprite: Sprite::new(Vec2::new(17.0, 17.0)),
                    ..Default::default()
                });
            })
            ;
        }
    })
    .insert(producer)
    ;
}

fn producer_tick_system(
    tick: Res<Tick>,
    mut query: Query<&mut Producer>,
    recipes: Res<Table<RecipeKind>>,
) {
    if **tick {
        for mut producer in query.iter_mut() {
            producer.attempt_to_start_production(&recipes);
            producer.tick(&recipes);
        }
    }
}

fn producer_color_system(
    colors: Res<Colors>,
    mut query: Query<(&Producer, &mut Handle<ColorMaterial>,)>,
) {
    for (producer, mut color) in query.iter_mut() {
        *color = match producer.status() {
            ProductionStatus::Idle => colors.red.clone(),
            ProductionStatus::Producing => colors.green.clone(),
            ProductionStatus::Full => colors.yellow.clone(),
        }
    }
}

fn producer_counts(producer: &Producer) -> (String, String) {
    let (input, output) = producer.item_counts();

    // Note: This should all probably be a Display impl.
    fn make_string(slot_counts: Vec<(ItemStackQuanity, ItemStackQuanity)>) -> String {
        if slot_counts.is_empty() {
            return "None".to_string();
        }

        let mut string = String::new();

        for (quantity, capacity) in slot_counts.iter() {
            string.push_str(&quantity.to_string());
            string.push('/');
            string.push_str(&capacity.to_string());
            string.push(',');
            string.push(' ');
        }

        // Remove the last ", "
        string.pop();
        string.pop();

        string
    }

    (make_string(input), make_string(output))
    
}

fn producer_io_count_text_system(
    producer_query: Query<&Producer>,
    mut producer_text_query: Query<(&mut Text, &Parent), With<ProducerIOCountText>>,
) {
    for (mut text, &parent) in producer_text_query.iter_mut() {
        let producer: &Producer = producer_query.get_component::<Producer>(*parent).expect("producer must exist");
        let (input_text, output_text) = producer_counts(producer);

        text.sections[0].value = input_text;
        text.sections[2].value = output_text;
    }
}

fn producer_entry_exit_color_system(
    colors: Res<Colors>,
    mut query: Query<(&mut Handle<ColorMaterial>, &MouseInteraction), Or<(With<ConnectorGiver>, With<ConnectorTaker>,)>>
) {
    for (mut color, interaction) in query.iter_mut() {
        // Ignore the currently selected one.
        if *color == colors.grey {
            continue;
        }

        match interaction {
            MouseInteraction::None => {
                *color = colors.white.clone();
            },
            MouseInteraction::Hovered => {
                *color = colors.green.clone();
            },
        }
    }
}

fn connector_transfer_system(
    tick: Res<Tick>,
    mut producer_query: Query<(&mut Producer,)>,
    mut connector_query: Query<(&mut Connector,)>,
    mut connector_line_query: Query<(&mut Transform, &Parent), With<ConnectorLine>>,
    giver_query: Query<(&ConnectorGiver, &Parent)>,
    taker_query: Query<(&ConnectorTaker, &Parent)>,
) {
    if **tick {
        for (giver, parent) in giver_query.iter() {
            for &connector_entity in giver.givers.iter() {
                let mut connector = match connector_query.get_component_mut::<Connector>(connector_entity) {
                    Ok(connector) => connector,
                    // Possible because there's no system sync.
                    Err(_) => continue,
                };

                if connector.status() == ConnectorStatus::WaitingOnInput {
                    let mut producer = producer_query.get_component_mut::<Producer>(**parent).expect("Producer entity exists");

                    if producer.has_output() {
                        let item_stack = producer.take_single_item();

                        // We know at this point that the connector can take an item stack and that it doesn't return one.
                        // As such, we discard the return value.
                        let _ = connector.insert_stack(item_stack);
                    }
                }
            }

            for (taker, parent) in taker_query.iter() {
                for &connector_entity in taker.takers.iter() {
                    let mut connector = match connector_query.get_component_mut::<Connector>(connector_entity) {
                        Ok(connector) => connector,
                        // Possible because the connector is added one frame after the handle is
                        // given to the ConnectorGiver/ConnectorTaker.
                        Err(_) => continue,
                    };
    
                    if connector.status() == ConnectorStatus::WaitingOnOutput {
                        let mut producer = producer_query.get_component_mut::<Producer>(**parent).expect("Producer entity exists");
    
                        if producer.takes_input() {
                            let result = producer.try_insert_ingredient(connector.take_stack());
                            if let Some(stack) = result.get_item_stack() {
                                let _ = connector.insert_stack(stack);
                            }
                        }
                    }
                }
            }

            for (mut connector,) in connector_query.iter_mut() {
                connector.tick();
            }
    
            for (mut transform, parent) in connector_line_query.iter_mut() {
                let connector = connector_query.get_component::<Connector>(**parent).expect("parent has to be a connector");
                transform.translation.x = connector.position() - (connector.length() / 2.0);
            }
        }
    }
}

/// An entity that gives input to a connector.
struct ConnectorGiver {
    givers: Vec<Entity>,
}

impl ConnectorGiver {
    fn new() -> Self {
        Self {
            givers: vec![],
        }
    }
}

/// An entity that takes output from a connector.
struct ConnectorTaker {
    takers: Vec<Entity>,
}

impl ConnectorTaker {
    fn new() -> Self {
        Self {
            takers: vec![],
        }
    }
}

struct ConnectorLine;