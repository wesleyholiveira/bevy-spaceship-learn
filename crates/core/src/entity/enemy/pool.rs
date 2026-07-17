use crate::GameConfig;
use crate::entity::enemy::Enemy;
use crate::entity::enemy::PatternEmitter;
use crate::entity::enemy::Health;
use crate::entity::projectile::{Active, Inactive};

use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EnemyPool {
    available: Vec<Entity>,
}

impl EnemyPool {
    pub fn acquire(&mut self) -> Option<Entity> {
        self.available.pop()
    }

    pub fn release(&mut self, entity: Entity) {
        self.available.push(entity);
    }

    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    pub fn push(&mut self, entity: Entity) {
        self.available.push(entity);
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct EnemyPoolStats {
    pub failed_spawns: u64,
    pub total_releases: u64,
}

pub fn init_enemy_pool(
    mut commands: Commands,
    config: Res<GameConfig>,
    mut pool: ResMut<EnemyPool>,
) {
    for _ in 0..config.max_enemies {
        let entity = commands
            .spawn((Enemy, Inactive, Visibility::Hidden, Transform::default()))
            .id();
        pool.push(entity);
    }
}

pub fn spawn_enemy(
    world: &mut World,
    position: Vec2,
    pattern: PatternEmitter,
    health: f32,
) -> Option<Entity> {
    let entity = match world.resource_mut::<EnemyPool>().acquire() {
        Some(entity) => entity,
        None => {
            world.resource_mut::<EnemyPoolStats>().failed_spawns += 1;
            return None;
        }
    };
    world
        .entity_mut(entity)
        .remove::<Inactive>()
        .insert(Active)
        .insert(Visibility::Inherited)
        .insert(Transform::from_translation(position.extend(0.0)))
        .insert(pattern)
        .insert(Health {
            current: health,
            max: health,
        });
    Some(entity)
}

pub fn release_enemy(world: &mut World, entity: Entity) {
    world
        .entity_mut(entity)
        .remove::<Active>()
        .remove::<PatternEmitter>()
        .remove::<Health>()
        .insert(Inactive)
        .insert(Visibility::Hidden)
        .insert(Transform::default());
    world.resource_mut::<EnemyPool>().release(entity);
    world.resource_mut::<EnemyPoolStats>().total_releases += 1;
}
