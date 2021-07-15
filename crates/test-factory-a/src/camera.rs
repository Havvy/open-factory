use std::ops::Deref;

use bevy::{math::Vec4Swizzles, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .insert_resource(Some(MousePositionInWorld(Default::default())))
        .add_startup_system(setup_camera.system())
        .add_system(update_mouse_location.system())
        ;
    }
}

struct MainCamera;
pub struct MousePositionInWorld(Vec2);

impl MousePositionInWorld {
    pub fn transform(&self) -> Transform {
        Transform::from_xyz(self.x, self.y, 0.0)
    }
}

impl Deref for MousePositionInWorld {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d()).insert(MainCamera);
    commands.spawn_bundle(UiCameraBundle::default());
}

// cursor2world in Unofficial Bevy Cookbook
fn update_mouse_location(
    windows: Res<Windows>,
    camera: Query<&Transform, With<MainCamera>>,
    mut mouse_position_in_world: ResMut<Option<MousePositionInWorld>>,
) {
    let window = windows.get_primary().unwrap();

    if let Some(pos) = window.cursor_position() {
        if pos.y < 100.0 {
            *mouse_position_in_world = None;
            return;
        }

        // get the size of the window
        let size = Vec2::new(window.width() as f32, window.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let pos = pos - size / 2.0;

        // assuming there is exactly one main camera entity, so this is OK
        let camera_transform = camera.single().unwrap();

        // apply the camera transform
        let pos = camera_transform.compute_matrix() * pos.extend(0.0).extend(1.0);

        // store it
        *mouse_position_in_world = Some(MousePositionInWorld(pos.xy()));
    } else {
        *mouse_position_in_world = None;
    }
}