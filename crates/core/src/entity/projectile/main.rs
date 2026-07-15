use bevy::prelude::Component;
use bevy::time::Timer;

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: Timer,
}