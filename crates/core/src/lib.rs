mod entity;

use bevy::prelude::*;
use entity::emitter::player_emit;
use entity::projectile::path_pattern::linear::update_linear;
use entity::projectile::path_pattern::sine_wave::update_sine_wave;
use entity::projectile::path_pattern::spiral::update_spiral;

pub use entity::emitter::{Emitter, PlayerEmitter};
pub use entity::projectile::Projectile;

pub const SHIP_SPEED: f32 = 320.0;

#[derive(Component)]
pub struct Ship;

#[derive(Resource, Default)]
pub struct MovementIntent(pub Vec2);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplaySet {
    Input,
    Simulation,
    Presentation,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementIntent>()
            .configure_sets(
                Update,
                (
                    GameplaySet::Input,
                    GameplaySet::Simulation,
                    GameplaySet::Presentation,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    move_ship,
                    player_emit,
                    update_linear,
                    update_sine_wave,
                    update_spiral,
                )
                    .chain()
                    .in_set(GameplaySet::Simulation),
            );
    }
}

fn move_ship(
    time: Res<Time>,
    movement_intent: Res<MovementIntent>,
    mut ships: Query<&mut Transform, With<Ship>>,
) {
    for mut transform in &mut ships {
        let displacement = movement_intent.0.normalize_or_zero() * SHIP_SPEED * time.delta_secs();
        transform.translation += displacement.extend(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_move_without_input() {
        let displacement = Vec2::ZERO.normalize_or_zero() * SHIP_SPEED * 1.0;
        assert_eq!(displacement, Vec2::ZERO);
    }

    #[test]
    fn keeps_diagonal_speed_normalized() {
        let displacement = Vec2::ONE.normalize_or_zero() * SHIP_SPEED * 1.0;
        assert!((displacement.length() - SHIP_SPEED).abs() < 0.001);
    }
}
