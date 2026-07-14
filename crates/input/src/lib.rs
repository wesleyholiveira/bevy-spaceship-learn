use bevy::prelude::*;
use spaceship_core::{GameplaySet, MovementIntent};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, read_movement_intent.in_set(GameplaySet::Input));
    }
}

fn read_movement_intent(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut movement_intent: ResMut<MovementIntent>,
) {
    movement_intent.0 = movement_direction(
        keyboard.pressed(KeyCode::KeyW),
        keyboard.pressed(KeyCode::KeyS),
        keyboard.pressed(KeyCode::KeyA),
        keyboard.pressed(KeyCode::KeyD),
    );
}

fn movement_direction(up: bool, down: bool, left: bool, right: bool) -> Vec2 {
    let mut direction = Vec2::ZERO;

    if up {
        direction.y += 1.0;
    }
    if down {
        direction.y -= 1.0;
    }
    if left {
        direction.x -= 1.0;
    }
    if right {
        direction.x += 1.0;
    }

    direction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_wasd_to_direction() {
        assert_eq!(
            movement_direction(true, false, true, false),
            Vec2::new(-1.0, 1.0)
        );
    }

    #[test]
    fn cancels_opposite_keys() {
        assert_eq!(movement_direction(true, true, true, true), Vec2::ZERO);
    }
}
