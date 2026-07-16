use crate::entity::projectile::Projectile;
use crate::entity::projectile::path_pattern::linear::LinearPath;

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
) {
    for (_entity, transform, mut emitter) in &mut emitters {
        emitter.fire_rate.tick(time.delta());
        if emitter.fire_rate.just_finished() {
            let origin = transform.translation.truncate();
            let dir = Vec2::Y;
            let now = time.elapsed_secs();

            commands.spawn((
                Name::new("Player Projectile"),
                Projectile {
                    damage: 1.0,
                    lifetime: Timer::from_seconds(3.0, TimerMode::Once),
                },
                LinearPath {
                    origin,
                    dir,
                    speed: 600.0,
                    spawn_time: now,
                },
                Sprite::from_color(Color::srgb(1.0, 0.8, 0.2), Vec2::new(8.0, 16.0)),
                Transform::from_translation(origin.extend(0.0)),
            ));
        }
    }
}
