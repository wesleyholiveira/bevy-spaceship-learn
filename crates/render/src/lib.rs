use bevy::prelude::*;
use spaceship_core::{Emitter, PlayerEmitter, Ship};

const SHIP_SIZE: Vec2 = Vec2::new(64.0, 64.0);

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.015, 0.02, 0.04)))
            .add_systems(Startup, setup_scene);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Name::new("Main Camera"), Camera2d));
    commands.spawn((
        Name::new("Ship"),
        Ship,
        Emitter {
            fire_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
        PlayerEmitter,
        Sprite::from_color(Color::srgb(0.2, 0.75, 1.0), SHIP_SIZE),
        Transform::default(),
    ));
}
