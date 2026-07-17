pub mod pool;

use crate::PlayerTarget;
use crate::entity::projectile::{Active, Inactive, Projectile};

use bevy::prelude::*;

pub use pool::{EnemyPool, EnemyPoolStats};

#[derive(Component)]
pub struct Enemy;

#[derive(Component, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct PatternEmitter {
    pub fire_rate: Timer,
    pub cooldown: Timer,
    pub pattern: PatternType,
    pub state: PatternState,
}

#[derive(Clone, Copy)]
pub enum PatternType {
    Ring {
        bullet_count: usize,
        speed: f32,
        rotation_speed: f32,
    },
    Spinning {
        pairs: usize,
        spacing: f32,
        angular_deviation: f32,
        pair_offset: f32,
        orbit_radius: f32,
        orbit_speed: f32,
    },
}

#[derive(Clone, Copy, Default)]
pub struct PatternState {
    pub pairs_released: usize,
    pub current_angle: f32,
}

impl Default for PatternType {
    fn default() -> Self {
        Self::Ring {
            bullet_count: 24,
            speed: 400.0,
            rotation_speed: 0.0,
        }
    }
}

pub fn enemy_emit(
    time: Res<Time>,
    mut commands: Commands,
    mut emitters: Query<(Entity, &Transform, &mut PatternEmitter), With<Enemy>>,
    inactive: Query<Entity, With<Inactive>>,
    player: Query<&Transform, With<PlayerTarget>>,
) {
    let player_pos = player
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (_entity, transform, mut emitter) in &mut emitters {
        emitter.fire_rate.tick(time.delta());
        emitter.cooldown.tick(time.delta());

        if emitter.cooldown.just_finished() {
            emitter.state = PatternState::default();
        }

        if !emitter.cooldown.is_finished() || !emitter.fire_rate.just_finished() {
            continue;
        }

        let origin = transform.translation.truncate();
        let now = time.elapsed_secs();

        match emitter.pattern {
            PatternType::Ring {
                bullet_count,
                speed,
                rotation_speed,
            } => {
                let angle_offset = now * rotation_speed;

                for i in 0..bullet_count {
                    let Some(pool_entity) = inactive.iter().next() else {
                        break;
                    };

                    let angle =
                        (i as f32 / bullet_count as f32) * std::f32::consts::TAU + angle_offset;
                    let dir = Vec2::new(angle.cos(), angle.sin());
                    let velocity = dir * speed;

                    commands
                        .entity(pool_entity)
                        .remove::<Inactive>()
                        .insert(Active)
                        .insert(Visibility::Inherited)
                        .insert(Transform::from_translation(origin.extend(0.0)))
                        .insert(crate::entity::projectile::movement::Movement::linear(
                            velocity,
                        ))
                        .insert(Projectile {
                            damage: 1.0,
                            lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                        });
                }
            }
            PatternType::Spinning {
                pairs,
                angular_deviation,
                pair_offset,
                orbit_radius,
                orbit_speed,
                ..
            } => {
                if emitter.state.pairs_released >= pairs {
                    emitter.cooldown.reset();
                    continue;
                }

                let dir_to_player = (player_pos - origin).normalize();
                let angle_to_player = dir_to_player.y.atan2(dir_to_player.x);
                let available: Vec<Entity> = inactive.iter().take(2).collect();

                for (i, pool_entity) in available.into_iter().enumerate() {
                    // Each bullet in the pair has an angular offset
                    let angle_offset = (i as f32 - 0.5) * angular_deviation;
                    let bullet_angle = angle_to_player + angle_offset;

                    // Calculate spawn position with radial offset perpendicular to player direction
                    let perpendicular = dir_to_player.perp();
                    let offset = perpendicular * pair_offset * (i as f32 - 0.5);

                    let spawn_pos = origin + offset;

                    // Calculate initial velocity based on bullet angle
                    let initial_velocity =
                        Vec2::new(bullet_angle.cos(), bullet_angle.sin()) * 400.0;
                    // Target velocity orbits around the player
                    let target_velocity = Vec2::new(
                        (bullet_angle + std::f32::consts::FRAC_PI_2).cos(),
                        (bullet_angle + std::f32::consts::FRAC_PI_2).sin(),
                    ) * orbit_speed;

                    commands
                        .entity(pool_entity)
                        .remove::<Inactive>()
                        .insert(Active)
                        .insert(Visibility::Inherited)
                        .insert(Transform::from_translation(spawn_pos.extend(0.0)))
                        .insert(crate::entity::projectile::movement::Movement::asymptotic(
                            initial_velocity,
                            target_velocity,
                            0.95,
                        ))
                        .insert(Projectile {
                            damage: 1.0,
                            lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                        });
                }

                emitter.state.pairs_released += 1;
                emitter.state.current_angle += angular_deviation;
            }
        }
    }
}
