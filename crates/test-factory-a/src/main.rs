use bevy::{prelude::*};

mod camera;
mod ui;
mod factory;
mod mouse_interaction;
mod tick;
mod database;
struct Colors {
    green: Handle<ColorMaterial>,
    yellow: Handle<ColorMaterial>,
    red: Handle<ColorMaterial>,
    white: Handle<ColorMaterial>,
    grey: Handle<ColorMaterial>,
    black: Handle<ColorMaterial>,
    blue: Handle<ColorMaterial>,
}

struct GameFont(Handle<Font>);

fn setup_handles(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>, asset_server: Res<AssetServer>,) {
    let red = materials.add(Color::rgb(0.6, 0.0, 0.0).into());
    let yellow = materials.add(Color::rgb(0.6, 0.6, 0.0).into());
    let green = materials.add(Color::rgb(0.0, 0.6, 0.0).into());
    let white = materials.add(Color::rgb(1.0, 1.0, 1.0).into());
    let grey = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let black = materials.add(Color::rgb(0.0, 0.0, 0.0).into());
    let blue = materials.add(Color::rgb(0.11764705882352941, 0.5372549019607843, 0.7019607843137254).into());

    let font = GameFont(asset_server.load::<Font, _>("fonts/FiraSans-Bold.ttf"));

    commands.insert_resource(Colors {
        green, yellow, red, white, grey, black, blue,
    });
    commands.insert_resource(font)
}

fn main() {
    App::build()
    .insert_resource(WindowDescriptor {
        title: "Test Factory".to_string(),
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system_to_stage(StartupStage::PreStartup, setup_handles.system())
    .add_system(mouse_interaction::update_interaction_system.system())
    .add_plugin(database::DatabasePlugin)
    .add_plugin(camera::CameraPlugin)
    .add_plugin(tick::TickPlugin)
    .add_plugin(factory::FactoryProducerPlugin)
    .add_plugin(ui::UiPlugin)
    .add_system(bevy::input::system::exit_on_esc_system.system())
    .run();
}