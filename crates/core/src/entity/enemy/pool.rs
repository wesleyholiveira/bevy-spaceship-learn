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
