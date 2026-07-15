use bevy::math::Vec2;
use bevy::prelude::{Commands, Component, Entity, Query, Res, Transform};
use bevy::time::Time;
use crate::entity::projectile::path_pattern::{update_paths, PathPattern};
use crate::entity::projectile::Projectile;
#[derive(Component)]
pub struct SpiralPath {
    pub origin: Vec2,
    pub start_angle: f32,
    pub radial_speed: f32,
    pub angular_speed: f32,
    pub spawn_time: f32,
}

impl PathPattern for SpiralPath {
    fn spawn_time(&self) -> f32 { self.spawn_time }
    fn evaluate(&self, t: f32) -> Vec2 {
        let angle = self.start_angle + self.angular_speed * t;
        let radius = self.radial_speed * t;
        self.origin + Vec2::from_angle(angle) * radius
    }
}
pub fn update_spiral(
    c: Commands,
    t: Res<Time>,
    q: Query<(Entity, &mut Transform, &SpiralPath, &mut Projectile)>,
) {
    update_paths::<SpiralPath>(c, t, q);
}

