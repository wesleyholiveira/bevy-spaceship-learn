use crate::entity::projectile::Projectile;
use crate::entity::projectile::path_pattern::{PathPattern, update_paths};

use bevy::prelude::*;

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
    fn evaluate(&self, now: f32) -> Vec2 {
        let t = now - self.spawn_time;
        let forward = self.dir * self.speed * t;
        let perp = Vec2::new(-self.dir.y, self.dir.x);
        let wave = perp * (t * self.frequency).sin() * self.amplitude;
        self.origin + forward + wave
    }
}

pub fn update_sine_wave(
    commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut Transform, &SineWavePath, &mut Projectile)>,
) {
    update_paths::<SineWavePath>(commands, time, query);
}
