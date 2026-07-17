mod entity;

use bevy::prelude::*;

pub use entity::emitter::{Emitter, PlayerEmitter, player_emit};
pub use entity::enemy::pool::{init_enemy_pool, release_enemy, spawn_enemy};
pub use entity::enemy::{
    Enemy, EnemyPool, EnemyPoolStats, Health, PatternEmitter, PatternState, PatternType,
    cull_enemies, enemy_emit, release_dead_enemies,
};
pub use entity::projectile::movement::{Attraction, Movement, update_movement};
pub use entity::projectile::{
    Active, Inactive, Projectile, cull_projectiles, init_projectile_pool,
};

pub const DEFAULT_SHIP_SPEED: f32 = 320.0;
pub const DEFAULT_MAX_BULLETS: usize = 256;
pub const DEFAULT_CULL_MARGIN: f32 = 100.0;
pub const DEFAULT_MAX_ENEMIES: usize = 64;

#[derive(Resource, Clone, Copy)]
pub struct GameConfig {
    pub ship_speed: f32,
    pub max_bullets: usize,
    pub cull_margin: f32,
    pub max_enemies: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            ship_speed: DEFAULT_SHIP_SPEED,
            max_bullets: DEFAULT_MAX_BULLETS,
            cull_margin: DEFAULT_CULL_MARGIN,
            max_enemies: DEFAULT_MAX_ENEMIES,
        }
    }
}

#[derive(Component)]
pub struct Ship;

#[derive(Component)]
pub struct PlayerTarget;

#[derive(Resource, Default)]
pub struct MovementIntent(pub Vec2);

#[derive(Resource, Clone, Copy)]
pub struct CullBoundary {
    pub half_width: f32,
    pub half_height: f32,
}

impl Default for CullBoundary {
    fn default() -> Self {
        Self {
            half_width: 640.0,
            half_height: 360.0,
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Simulation,
    Presentation,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementIntent>()
            .init_resource::<CullBoundary>()
            .init_resource::<GameConfig>()
            .add_systems(Startup, init_projectile_pool)
            .configure_sets(
                Update,
                (
                    GameplaySet::Input,
                    GameplaySet::Simulation,
                    GameplaySet::Presentation,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (move_ship, player_emit, enemy_emit, update_movement)
                    .chain()
                    .in_set(GameplaySet::Simulation),
            )
            .add_systems(Update, cull_projectiles.in_set(GameplaySet::Presentation));
    }
}

fn move_ship(
    time: Res<Time>,
    movement_intent: Res<MovementIntent>,
    config: Res<GameConfig>,
    mut ships: Query<&mut Transform, With<Ship>>,
) {
    for mut transform in &mut ships {
        let displacement =
            movement_intent.0.normalize_or_zero() * config.ship_speed * time.delta_secs();
        transform.translation += displacement.extend(0.0);
    }
}
