mod entity;

use bevy::prelude::*;

pub use entity::emitter;
pub use entity::enemy;
pub use entity::projectile;

pub const DEFAULT_SHIP_SPEED: f32 = 320.0;
pub const DEFAULT_MAX_BULLETS: usize = 256;
pub const DEFAULT_CULL_MARGIN: f32 = 100.0;
pub const DEFAULT_MAX_ENEMIES: usize = 64;
pub const DEFAULT_SPATIAL_HASH_CELL_SIZE: f32 = 128.0;

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

#[derive(Resource, Clone, Copy)]
pub struct SpatialHashConfig {
    pub cell_size: f32,
}

impl Default for SpatialHashConfig {
    fn default() -> Self {
        Self {
            cell_size: DEFAULT_SPATIAL_HASH_CELL_SIZE,
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
            .init_resource::<SpatialHashConfig>()
            .init_resource::<enemy::pool::EnemyPool>()
            .init_resource::<enemy::pool::EnemyPoolStats>()
            .add_systems(Startup, projectile::init_projectile_pool)
            .add_systems(Startup, enemy::pool::init_enemy_pool)
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
                (
                    move_ship,
                    emitter::player_emit,
                    enemy::enemy_emit,
                    projectile::movement::update_movement,
                    enemy::lifecycle::release_dead_enemies,
                )
                    .chain()
                    .in_set(GameplaySet::Simulation),
            )
            .add_systems(
                Update,
                (projectile::cull_projectiles, enemy::lifecycle::cull_enemies)
                    .in_set(GameplaySet::Presentation),
            );
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
