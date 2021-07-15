use bevy::prelude::*;

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
        .insert_resource(TickTimer(Timer::from_seconds(1.0 / 20.0, true)))
        .insert_resource(Tick(false))
        .add_system(update_tick_system.system())
        ;
    }
}

struct TickTimer(Timer);

/// Whether or not a logic tick has occured.
///
/// Occurs every 1/20th of a second.
///
/// Use `**tick` to get to a bool.
pub struct Tick(bool);

impl std::ops::Deref for Tick {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn update_tick_system(
    time: Res<Time>,
    mut tick_timer: ResMut<TickTimer>,
    mut tick: ResMut<Tick>,
) {
    *tick = Tick(tick_timer.0.tick(time.delta()).just_finished());
}