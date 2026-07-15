pub mod linear;
pub mod spiral;
pub mod sine_wave;

use bevy::math::Vec2;
use bevy::prelude::{Commands, Component, Entity, Query, Res, Transform};
use bevy::time::Time;
use crate::entity::projectile::Projectile;

pub trait PathPattern: Component {
    fn evaluate(&self, now: f32) -> Vec2;
}

pub fn update_paths<P: PathPattern>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &P, &mut Projectile)>,
) {
    let now = time.elapsed_secs();

    for (entity, mut transform, path, mut projectile) in query.iter_mut() {
        transform.translation = path.evaluate(now).extend(0.0);

        projectile.lifetime.tick(time.delta());
        if projectile.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}