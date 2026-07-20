use crate::CullBoundary;
use crate::GameConfig;
use crate::entity::enemy::Enemy;
use crate::entity::enemy::Health;
use crate::entity::enemy::PatternEmitter;
use crate::entity::projectile::{Active, Inactive};

use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub fn cull_enemies(
    boundary: Res<CullBoundary>,
    config: Res<GameConfig>,
    mut commands: Commands,
    mut pool: ResMut<crate::entity::enemy::pool::EnemyPool>,
    mut stats: ResMut<crate::entity::enemy::pool::EnemyPoolStats>,
    active: Query<(Entity, &Transform), (With<Active>, With<Enemy>)>,
) {
    let limit_x = boundary.half_width + config.cull_margin;
    let limit_y = boundary.half_height + config.cull_margin;

    for (entity, transform) in &active {
        let pos = transform.translation.truncate();
        if pos.x.abs() > limit_x || pos.y.abs() > limit_y {
            // Inline release logic: identical to release_enemy(&mut world, entity)
            // from pool.rs. Cannot call the helper here because it takes &mut World
            // and would conflict with the Res<>/ResMut<>/Query system params.
            commands
                .entity(entity)
                .remove::<Active>()
                .remove::<PatternEmitter>()
                .remove::<Health>()
                .insert(Inactive)
                .insert(Visibility::Hidden)
                .insert(Transform::default());
            pool.release(entity);
            stats.total_releases += 1;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn release_dead_enemies(
    mut commands: Commands,
    mut pool: ResMut<crate::entity::enemy::pool::EnemyPool>,
    mut stats: ResMut<crate::entity::enemy::pool::EnemyPoolStats>,
    enemies: Query<(Entity, &Health), (With<Active>, With<Enemy>)>,
) {
    for (entity, health) in &enemies {
        if health.current <= 0.0 {
            // Inline release logic: see cull_enemies for the reason.
            commands
                .entity(entity)
                .remove::<Active>()
                .remove::<PatternEmitter>()
                .remove::<Health>()
                .insert(Inactive)
                .insert(Visibility::Hidden)
                .insert(Transform::default());
            pool.release(entity);
            stats.total_releases += 1;
        }
    }
}
