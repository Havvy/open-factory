use bevy::prelude::*;

use crate::camera::MousePositionInWorld;

pub struct Extents(pub Vec2);

fn point_in_area(center: &GlobalTransform, extents: &Extents, point: &Vec2) -> bool {
    let center = center.translation.truncate();
    let min = center - extents.0;
    let max = center + extents.0;

    (min.x..max.x).contains(&point.x)
    && (min.y..max.y).contains(&point.y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseInteraction {
    None,
    Hovered,
}

impl MouseInteraction {
    fn set(&mut self, value: MouseInteraction) {
        match (*self, value) {
            (MouseInteraction::None, MouseInteraction::Hovered) => {
                *self = value;
            },

            (MouseInteraction::Hovered, MouseInteraction::None) => {
                *self = value;
            },

            _ => { /* values already match. Nothing to do. */}
        }
    }
}

impl Default for MouseInteraction {
    fn default() -> Self {
        Self::None
    }
}

pub fn update_interaction_system(
    mouse_position_in_world: Res<Option<MousePositionInWorld>>,
    mut query: Query<(
        &Extents,
        &GlobalTransform,
        &mut MouseInteraction,
    )>,
) {
    let mouse_position_in_world = match *mouse_position_in_world {
        Some(ref mouse_position_in_world) => mouse_position_in_world,
        None => {
            for (_, _, mut interaction) in query.iter_mut() {
                interaction.set(MouseInteraction::None);
            }
            return;
        }
    };

    for (extents, global_transform, mut interaction) in query.iter_mut() {
        if point_in_area(global_transform, extents, mouse_position_in_world) {
            interaction.set(MouseInteraction::Hovered);
        } else {
            interaction.set(MouseInteraction::None);
        }
    }
}