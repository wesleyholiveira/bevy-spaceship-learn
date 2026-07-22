pub mod movement;

use crate::{CullBoundary, GameConfig};

use bevy::prelude::*;
use bevy::time::Timer;

#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProjectileOwner {
    #[default]
    Player,
    Enemy,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: Timer,
    pub owner: ProjectileOwner,
}

#[derive(Component)]
#[component(immutable)]
pub struct Active;

#[derive(Component)]
#[component(immutable)]
pub struct Inactive;

pub fn init_projectile_pool(mut commands: Commands, config: Res<GameConfig>) {
    for _ in 0..config.max_bullets {
        commands.spawn((
            Projectile {
                damage: 0.0,
                lifetime: Timer::from_seconds(0.0, TimerMode::Once),
                owner: ProjectileOwner::default(),
            },
            Inactive,
            Visibility::Hidden,
            Transform::default(),
        ));
    }
}

#[allow(clippy::type_complexity)]
pub fn cull_projectiles(
    boundary: Res<CullBoundary>,
    config: Res<GameConfig>,
    mut commands: Commands,
    active: Query<(Entity, &Transform), (With<Active>, With<Projectile>)>,
) {
    let limit_x = boundary.half_width + config.cull_margin;
    let limit_y = boundary.half_height + config.cull_margin;

    for (entity, transform) in &active {
        let pos = transform.translation.truncate();
        if pos.x.abs() > limit_x || pos.y.abs() > limit_y {
            commands
                .entity(entity)
                .remove::<Active>()
                .insert(Inactive)
                .insert(Visibility::Hidden);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_owner_defaults_to_player() {
        assert_eq!(ProjectileOwner::default(), ProjectileOwner::Player);
    }
}
