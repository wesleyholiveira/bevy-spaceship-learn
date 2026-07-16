use crate::entity::projectile::Projectile;
use crate::entity::projectile::path_pattern::{PathPattern, update_paths};

use bevy::prelude::*;

#[derive(Component)]
pub struct LinearPath {
    pub origin: Vec2,
    pub dir: Vec2,
    pub speed: f32,
    pub spawn_time: f32,
}

impl PathPattern for LinearPath {
    fn evaluate(&self, now: f32) -> Vec2 {
        let t = now - self.spawn_time;
        self.origin + self.dir * self.speed * t
    }
}

pub fn update_linear(
    commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut Transform, &LinearPath, &mut Projectile)>,
) {
    update_paths::<LinearPath>(commands, time, query);
}
