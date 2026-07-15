use bevy::math::Vec2;
use bevy::prelude::{Commands, Component, Entity, Query, Res, Transform};
use bevy::time::Time;
use crate::entity::projectile::path_pattern::{update_paths, PathPattern};
use crate::entity::projectile::Projectile;

#[derive(Component)]
pub struct LinearPath {
    pub origin: Vec2,
    pub dir: Vec2,
    pub speed: f32,
    pub spawn_time: f32,
}

impl PathPattern for LinearPath {
    fn spawn_time(&self) -> f32 { self.spawn_time }
    fn evaluate(&self, t: f32) -> Vec2 {
        self.origin + self.dir * self.speed * t
    }
}

fn update_linear(
    c: Commands,
    t: Res<Time>,
    q: Query<(Entity, &mut Transform, &LinearPath, &mut Projectile)>,
) {
    update_paths::<LinearPath>(c, t, q);
}