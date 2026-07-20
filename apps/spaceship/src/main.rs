use bevy::prelude::*;
use bevy::window::WindowPlugin;
use spaceship_core::enemy::pool::{init_enemy_pool, spawn_enemy};
use spaceship_core::enemy::{PatternEmitter, PatternState, PatternType};
use spaceship_core::CorePlugin;
use spaceship_input::InputPlugin;
use spaceship_render::RenderPlugin;
use spaceship_ui::UiPlugin;
use std::time::Duration;

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
        .add_systems(Startup, spawn_test_enemy.after(init_enemy_pool))
        .run();
}

fn spawn_test_enemy(world: &mut World) {
    let pattern = PatternEmitter {
        fire_rate: Timer::from_seconds(0.1, TimerMode::Repeating),
        cooldown: {
            let mut t = Timer::from_seconds(3.0, TimerMode::Once);
            t.tick(Duration::from_secs_f32(3.0));
            t
        },
        pattern: PatternType::Spinning {
            pairs: 25,
            spacing: 0.1,
            angular_deviation: 0.1,
            pair_offset: 20.0,
            orbit_radius: 30.0,
            orbit_speed: 3.0,
        },
        state: PatternState::default(),
    };

    spawn_enemy(world, Vec2::new(0.0, 200.0), pattern, 100.0);
}
