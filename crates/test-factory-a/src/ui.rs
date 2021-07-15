use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .init_resource::<ButtonMaterials>()
        .insert_resource(Action::default())
        .add_startup_system(setup.system())
        .add_system(button_system.system())
        ;
    }
}

/// Whichever button is currently pressed.
struct ActiveButton(Entity);

/// The button that was pressed on the previous frame.
struct PreviousActiveButton(Entity);

impl From<Entity> for ActiveButton {
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

impl From<Entity> for PreviousActiveButton {
    fn from(entity: Entity) -> Self {
        Self(entity)
    }
}

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    active: Handle<ColorMaterial>,
    background: Handle<ColorMaterial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Connect,
    Copper,
    Tin,
    Bronze,
    Trash,
}

impl Default for Action {
    fn default() -> Self {
        Action::Connect
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Action {
    pub fn iter_variants() -> <[Action; 5] as IntoIterator>::IntoIter {
        use Action::*;
        IntoIterator::into_iter([Connect, Copper, Tin, Bronze, Trash])
    }
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            active: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
            background: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
        }
    }
}

fn setup(
    mut commands: Commands,
    font: Res<crate::GameFont>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
    .spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            size: Size::new(Val::Percent(100.0), Val::Px(100.0)),
            ..Default::default()
        },
        material: button_materials.background.clone(),
        ..Default::default()
    })
    .with_children(|parent| {
        for action in Action::iter_variants() {
            let button_entity = parent.spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(180.0), Val::Px(65.0)),
                    // center button
                    margin: Rect::all(Val::Auto),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                material: button_materials.normal.clone(),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(TextBundle {
                    text: Text::with_section(
                        action.to_string(),
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                ;
            })
            .insert(action)
            .id();

            if action == Action::Connect {
                // All these items exist because `bevy::ecs::system::command::InsertResource`
                // cannot be constructed nor can I access the `Commands` struct from a `ChildBuilder`.
                // Sure, I could have stored the entity in a local and added the resource afterwards,
                // but sometimes I'm stubborn. Why use a local when three items will suffice?
                //
                // This can be cleaned up when bevy 0.6 is released. I added a `commands` method
                // and the Command structs are publicly constructable now.
                use bevy::ecs::{component::Component, system::Command};

                struct InsertResource<T: Component> {
                    resource: T,
                }
                
                impl<T: Component> Command for InsertResource<T> {
                    fn write(self: Box<Self>, world: &mut World) {
                        world.insert_resource(self.resource);
                    }
                }

                parent.add_command(InsertResource { resource: ActiveButton(button_entity) });
                parent.add_command(InsertResource { resource: PreviousActiveButton(button_entity) });
            }
        }
    });
}

fn button_system(
    mut active_button: ResMut<ActiveButton>,
    mut previous_active_button: ResMut<PreviousActiveButton>,
    button_materials: Res<ButtonMaterials>,
    mut interaction_query: Query<
        (Entity, &Interaction, &mut Handle<ColorMaterial>, &Action),
        (Changed<Interaction>, With<Button>),
    >,
    mut current_action: ResMut<Action>,
) {
    for (button_entity, interaction, mut material, action) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *current_action = *action;
                *material = button_materials.active.clone();

                *active_button = button_entity.into();
            },

            Interaction::Hovered if *current_action == *action => {
                // Do nothing.
            },

            Interaction::Hovered => {
                *material = button_materials.hovered.clone();
            },

            Interaction::None => {
                *material = if *current_action == *action {
                    button_materials.active.clone()
                } else {
                    button_materials.normal.clone()
                }
            },
        }
    }

    if active_button.0 != previous_active_button.0 {
        let mut material = interaction_query.get_component_mut(previous_active_button.0).expect("Button entity exists");
        *material = button_materials.normal.clone();
        previous_active_button.0 = active_button.0;
    }
}