use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use spaceship_core::emitter::{Emitter, PlayerEmitter};
use spaceship_core::enemy::Enemy;
use spaceship_core::projectile::Projectile;
use spaceship_core::{CullBoundary, PlayerTarget, Ship};

const SHIP_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const ENEMY_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const PROJECTILE_SIZE: Vec2 = Vec2::new(8.0, 16.0);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);
const ENEMY_COLOR: Color = Color::srgb(1.0, 0.2, 0.2);

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.015, 0.02, 0.04)))
            .add_systems(Startup, setup_scene)
            .add_systems(Update, (sync_cull_boundary, ensure_projectile_sprite, ensure_enemy_sprite));
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Name::new("Main Camera"), Camera2d));
    commands.spawn((
        Name::new("Ship"),
        Ship,
        PlayerTarget,
        Emitter {
            fire_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
        PlayerEmitter,
        Sprite::from_color(Color::srgb(0.2, 0.75, 1.0), SHIP_SIZE),
        Transform::default(),
    ));
}

fn ensure_projectile_sprite(
    mut commands: Commands,
    projectiles: Query<Entity, (With<Projectile>, Without<Sprite>)>,
) {
    for entity in &projectiles {
        commands
            .entity(entity)
            .insert(Sprite::from_color(PROJECTILE_COLOR, PROJECTILE_SIZE));
    }
}

fn ensure_enemy_sprite(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<Sprite>)>,
) {
    for entity in &enemies {
        commands
            .entity(entity)
            .insert(Sprite::from_color(ENEMY_COLOR, ENEMY_SIZE));
    }
}

fn sync_cull_boundary(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut boundary: ResMut<CullBoundary>,
) {
    if let Ok(window) = windows.single() {
        boundary.half_width = window.width() / 2.0;
        boundary.half_height = window.height() / 2.0;
    }
}
