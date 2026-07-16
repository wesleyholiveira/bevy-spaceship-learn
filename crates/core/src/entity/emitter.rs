use crate::entity::projectile::{Active, Inactive, Projectile};

use bevy::prelude::*;

#[derive(Component)]
pub struct Emitter {
    pub fire_rate: Timer,
}

#[derive(Component)]
pub struct PlayerEmitter;

pub fn player_emit(
    time: Res<Time>,
    mut commands: Commands,
    mut emitters: Query<(Entity, &Transform, &mut Emitter), With<PlayerEmitter>>,
    inactive: Query<Entity, With<Inactive>>,
) {
    for (_entity, transform, mut emitter) in &mut emitters {
        emitter.fire_rate.tick(time.delta());
        if !emitter.fire_rate.just_finished() {
            continue;
        }

        let Some(pool_entity) = inactive.iter().next() else {
            continue;
        };

        let origin = transform.translation.truncate();
        let dir = Vec2::Y;
        let velocity = dir * 600.0;

        commands
            .entity(pool_entity)
            .remove::<Inactive>()
            .insert(Active)
            .insert(Visibility::Inherited)
            .insert(Transform::from_translation(origin.extend(0.0)))
            .insert(crate::entity::projectile::movement::Movement::linear(velocity))
            .insert(Projectile {
                damage: 1.0,
                lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            });
    }
}
