use bevy::math::Vec2;
use bevy::prelude::{Commands, Component, Entity, Query, Res, Transform};
use bevy::time::Time;
use crate::entity::projectile::path_pattern::{update_paths, PathPattern};
use crate::entity::projectile::Projectile;
#[derive(Component)]
pub struct SineWavePath {
    pub origin: Vec2,
    pub dir: Vec2,
    pub speed: f32,
    pub amplitude: f32,
    pub frequency: f32,
    pub spawn_time: f32,
}

impl PathPattern for SineWavePath {
    fn spawn_time(&self) -> f32 { self.spawn_time }
    fn evaluate(&self, t: f32) -> Vec2 {
        let forward = self.dir * self.speed * t;
        let perp = Vec2::new(-self.dir.y, self.dir.x);
        let wave = perp * (t * self.frequency).sin() * self.amplitude;
        self.origin + forward + wave
    }
}

fn update_sine_wave(
    c: Commands,
    t: Res<Time>,
    q: Query<(Entity, &mut Transform, &SineWavePath, &mut Projectile)>,
) {
    update_paths::<SineWavePath>(c, t, q);
}