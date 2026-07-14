use bevy::prelude::*;

pub const SHIP_SPEED: f32 = 320.0;

#[derive(Component)]
pub struct Ship;

#[derive(Component, Default)]
pub struct Position(pub Vec2);

#[derive(Component)]
pub struct MovementSpeed(pub f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(SHIP_SPEED)
    }
}

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
            .add_systems(Update, move_ship.in_set(GameplaySet::Simulation));
    }
}

fn move_ship(
    time: Res<Time>,
    movement_intent: Res<MovementIntent>,
    mut ships: Query<(&MovementSpeed, &mut Position), With<Ship>>,
) {
    for (speed, mut position) in &mut ships {
        position.0 +=
            movement_displacement(movement_intent.0, speed.0, time.delta_secs());
    }
}

fn movement_displacement(intent: Vec2, speed: f32, delta_seconds: f32) -> Vec2 {
    intent.normalize_or_zero() * speed * delta_seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_move_without_input() {
        let displacement = movement_displacement(Vec2::ZERO, SHIP_SPEED, 1.0);

        assert_eq!(displacement, Vec2::ZERO);
    }

    #[test]
    fn keeps_diagonal_speed_normalized() {
        let displacement = movement_displacement(Vec2::ONE, SHIP_SPEED, 1.0);

        assert!((displacement.length() - SHIP_SPEED).abs() < 0.001);
    }
}
