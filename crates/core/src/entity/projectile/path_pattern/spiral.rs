use crate::entity::projectile::path_pattern::{PathPattern, update_paths};
use crate::entity::projectile::{Active, Projectile};

use bevy::prelude::*;

#[derive(Component)]
pub struct SpiralPath {
    pub origin: Vec2,
    pub start_angle: f32,
    pub radial_speed: f32,
    pub angular_speed: f32,
    pub spawn_time: f32,
}

impl PathPattern for SpiralPath {
    fn evaluate(&self, now: f32) -> Vec2 {
        let t = now - self.spawn_time;
        let angle = self.start_angle + self.angular_speed * t;
        let radius = self.radial_speed * t;
        self.origin + Vec2::from_angle(angle) * radius
    }
}

pub fn update_spiral(
    commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut Transform, &SpiralPath, &mut Projectile), With<Active>>,
) {
    update_paths::<SpiralPath>(commands, time, query);
}
