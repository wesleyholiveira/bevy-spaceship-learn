use bevy::prelude::*;
use bevy::window::WindowPlugin;
use spaceship_core::{CorePlugin, Enemy, PatternEmitter, PatternState, PatternType};
use spaceship_input::InputPlugin;
use spaceship_render::RenderPlugin;
use spaceship_ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spaceship".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((CorePlugin, InputPlugin, RenderPlugin, UiPlugin))
        .add_systems(Startup, spawn_test_enemy)
        .run();
}

fn spawn_test_enemy(mut commands: Commands) {
    commands.spawn((
        Enemy,
        PatternEmitter {
            fire_rate: Timer::from_seconds(3.0, TimerMode::Repeating),
            pattern: PatternType::Spinning {
                pairs: 25,
                spacing: 0.1,
                angular_deviation: 0.1,
                pair_offset: 20.0,
                orbit_radius: 30.0,
                orbit_speed: 3.0,
            },
            state: PatternState::default(),
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
    ));
}
