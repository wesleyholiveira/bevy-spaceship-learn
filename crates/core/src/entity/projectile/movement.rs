use crate::entity::projectile::{Active, Inactive, Projectile};

use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Movement {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub retention: f32,
    pub attraction: Option<Attraction>,
}

#[derive(Clone, Copy)]
pub struct Attraction {
    pub target: Vec2,
    pub strength: f32,
    pub exponent: f32,
}

impl Movement {
    pub fn linear(velocity: Vec2) -> Self {
        Self::builder().velocity(velocity).build()
    }

    pub fn accelerated(velocity: Vec2, acceleration: Vec2) -> Self {
        Self::builder()
            .velocity(velocity)
            .acceleration(acceleration)
            .build()
    }

    pub fn asymptotic(initial: Vec2, target: Vec2, retention: f32) -> Self {
        Self::builder()
            .velocity(initial)
            .acceleration(target * (1.0 - retention))
            .retention(retention)
            .build()
    }

    pub fn towards(velocity: Vec2, target: Vec2, strength: f32) -> Self {
        Self::builder()
            .velocity(velocity)
            .attraction(Attraction {
                target,
                strength,
                exponent: 1.0,
            })
            .build()
    }

    pub fn builder() -> MovementBuilder {
        MovementBuilder::default()
    }
}

pub struct MovementBuilder {
    velocity: Vec2,
    acceleration: Vec2,
    retention: f32,
    attraction: Option<Attraction>,
}

impl Default for MovementBuilder {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            retention: 1.0,
            attraction: None,
        }
    }
}

impl MovementBuilder {
    pub fn velocity(mut self, v: Vec2) -> Self {
        self.velocity = v;
        self
    }

    pub fn acceleration(mut self, a: Vec2) -> Self {
        self.acceleration = a;
        self
    }

    pub fn retention(mut self, r: f32) -> Self {
        self.retention = r;
        self
    }

    pub fn attraction(mut self, a: Attraction) -> Self {
        self.attraction = Some(a);
        self
    }

    pub fn build(self) -> Movement {
        Movement {
            velocity: self.velocity,
            acceleration: self.acceleration,
            retention: self.retention,
            attraction: self.attraction,
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Movement, &mut Projectile), With<Active>>,
) {
    let dt = time.delta();
    for (entity, mut transform, mut movement, mut projectile) in &mut query {
        let mut new_velocity = movement.acceleration + movement.velocity * movement.retention;

        if let Some(attraction) = movement.attraction {
            let to_target = attraction.target - transform.translation.truncate();
            let distance = to_target.length();

            if distance > 0.01 {
                let direction = to_target / distance;
                let force_magnitude = attraction.strength * distance.powf(attraction.exponent - 1.0);
                new_velocity += direction * force_magnitude * time.delta_secs();
            }
        }

        movement.velocity = new_velocity;
        transform.translation += movement.velocity.extend(0.0) * time.delta_secs();

        projectile.lifetime.tick(dt);
        if projectile.lifetime.is_finished() {
            commands
                .entity(entity)
                .remove::<Active>()
                .insert(Inactive)
                .insert(Visibility::Hidden);
        }
    }
}
