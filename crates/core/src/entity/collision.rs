use crate::PlayerTarget;
use crate::SpatialHashConfig;
use crate::entity::enemy::{Enemy, Health};
use crate::entity::projectile::{Active, Inactive, Projectile, ProjectileOwner};
use crate::{ENEMY_HALF_SIZE, PROJECTILE_HALF_SIZE, SHIP_HALF_SIZE};

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Component, Clone, Copy)]
pub struct HitFlash {
    pub remaining: f32,
    pub total: f32,
}

impl HitFlash {
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
            total: duration,
        }
    }
}

#[derive(Resource, Default)]
pub struct CellQueryDebug(pub Vec<IVec2>);

pub fn cell_index(pos: Vec2, cell_size: f32) -> IVec2 {
    let safe_cell = cell_size.max(1.0);
    let cell = pos / safe_cell;
    IVec2::new(cell.x.floor() as i32, cell.y.floor() as i32)
}

fn aabb_overlap(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    (a_center.x - b_center.x).abs() < a_half.x + b_half.x
        && (a_center.y - b_center.y).abs() < a_half.y + b_half.y
}

const QUERY_RADIUS: i32 = 1;

#[allow(clippy::type_complexity)]
pub fn detect_collisions(
    mut commands: Commands,
    config: Res<SpatialHashConfig>,
    mut cell_query: ResMut<CellQueryDebug>,
    projectiles: Query<(Entity, &Transform, &Projectile), (With<Active>, With<Projectile>)>,
    player: Query<(Entity, &Transform), With<PlayerTarget>>,
    mut enemies: Query<(Entity, &Transform, &mut Health), (With<Enemy>, With<Active>)>,
) {
    cell_query.0.clear();

    let safe_cell = config.cell_size.max(1.0);

    struct ProjEntry {
        entity: Entity,
        position: Vec2,
        owner: ProjectileOwner,
        damage: f32,
    }

    let mut cell_map: HashMap<IVec2, Vec<ProjEntry>> = HashMap::new();

    for (entity, transform, projectile) in &projectiles {
        let pos = transform.translation.truncate();
        let cell = cell_index(pos, safe_cell);
        cell_map.entry(cell).or_default().push(ProjEntry {
            entity,
            position: pos,
            owner: projectile.owner,
            damage: projectile.damage,
        });
    }

    let mut queried = HashSet::new();

    // Player collisions — only enemy-owned projectiles hit the player
    if let Ok((player_entity, player_transform)) = player.single() {
        let player_pos = player_transform.translation.truncate();
        let player_cell = cell_index(player_pos, safe_cell);

        for dx in -QUERY_RADIUS..=QUERY_RADIUS {
            for dy in -QUERY_RADIUS..=QUERY_RADIUS {
                let cell = IVec2::new(player_cell.x + dx, player_cell.y + dy);
                queried.insert(cell);

                let Some(entries) = cell_map.get(&cell) else {
                    continue;
                };

                for entry in entries {
                    if entry.owner == ProjectileOwner::Player {
                        continue;
                    }
                    if !aabb_overlap(
                        player_pos,
                        SHIP_HALF_SIZE,
                        entry.position,
                        PROJECTILE_HALF_SIZE,
                    ) {
                        continue;
                    }
                    commands.entity(player_entity).insert(HitFlash::new(0.15));
                    commands
                        .entity(entry.entity)
                        .remove::<Active>()
                        .insert((Inactive, Visibility::Hidden));
                }
            }
        }
    }

    // Enemy collisions — only player-owned projectiles hit enemies
    for (enemy_entity, enemy_transform, mut health) in &mut enemies {
        let enemy_pos = enemy_transform.translation.truncate();
        let enemy_cell = cell_index(enemy_pos, safe_cell);

        for dx in -QUERY_RADIUS..=QUERY_RADIUS {
            for dy in -QUERY_RADIUS..=QUERY_RADIUS {
                let cell = IVec2::new(enemy_cell.x + dx, enemy_cell.y + dy);
                queried.insert(cell);

                let Some(entries) = cell_map.get(&cell) else {
                    continue;
                };

                for entry in entries {
                    if entry.owner == ProjectileOwner::Enemy {
                        continue;
                    }
                    if !aabb_overlap(
                        enemy_pos,
                        ENEMY_HALF_SIZE,
                        entry.position,
                        PROJECTILE_HALF_SIZE,
                    ) {
                        continue;
                    }
                    health.current -= entry.damage;
                    commands.entity(enemy_entity).insert(HitFlash::new(0.15));
                    commands
                        .entity(entry.entity)
                        .remove::<Active>()
                        .insert((Inactive, Visibility::Hidden));
                }
            }
        }
    }

    cell_query.0 = queried.into_iter().collect();
}

pub fn tick_hit_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HitFlash)>,
) {
    for (entity, mut flash) in &mut query {
        flash.remaining -= time.delta_secs();
        if flash.remaining <= 0.0 {
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_index_centers_origin_at_zero() {
        let idx = cell_index(Vec2::ZERO, 128.0);
        assert_eq!(idx, IVec2::new(0, 0));
    }

    #[test]
    fn cell_index_positive_position() {
        let idx = cell_index(Vec2::new(150.0, 300.0), 128.0);
        assert_eq!(idx, IVec2::new(1, 2));
    }

    #[test]
    fn cell_index_negative_position() {
        let idx = cell_index(Vec2::new(-150.0, -300.0), 128.0);
        assert_eq!(idx, IVec2::new(-2, -3));
    }

    #[test]
    fn cell_index_on_boundary() {
        // Exactly at 128.0 → cell x = 1.0, floor = 1
        let idx = cell_index(Vec2::new(128.0, 0.0), 128.0);
        assert_eq!(idx, IVec2::new(1, 0));
    }

    #[test]
    fn cell_index_handles_zero_cell_size() {
        let idx = cell_index(Vec2::new(150.0, 0.0), 0.0);
        // Should not panic; safe_cell = 1.0
        assert_eq!(idx, IVec2::new(150, 0));
    }

    #[test]
    fn aabb_overlap_detects_intersection() {
        assert!(aabb_overlap(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            Vec2::new(5.0, 0.0),
            Vec2::new(10.0, 10.0),
        ));
    }

    #[test]
    fn aabb_overlap_distant_does_not_intersect() {
        assert!(!aabb_overlap(
            Vec2::ZERO,
            Vec2::new(10.0, 10.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(10.0, 10.0),
        ));
    }

    #[test]
    fn hit_flash_new_creates_timer_with_correct_duration() {
        let flash = HitFlash::new(0.15);
        assert!((flash.remaining - 0.15).abs() < f32::EPSILON);
        assert!((flash.total - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn cell_query_debug_defaults_to_empty() {
        let debug = CellQueryDebug::default();
        assert!(debug.0.is_empty());
    }
}
