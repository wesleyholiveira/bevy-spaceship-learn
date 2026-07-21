use bevy::color::palettes::basic::GREEN;
use bevy::math::Isometry2d;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};
use spaceship_core::emitter::{Emitter, PlayerEmitter};
use spaceship_core::enemy::Enemy;
use spaceship_core::projectile::Projectile;
use spaceship_core::{CullBoundary, PlayerTarget, Ship, SpatialHashConfig};

const SHIP_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const ENEMY_SIZE: Vec2 = Vec2::new(64.0, 64.0);
const PROJECTILE_SIZE: Vec2 = Vec2::new(8.0, 16.0);
const PROJECTILE_COLOR: Color = Color::srgb(1.0, 0.8, 0.2);
const ENEMY_COLOR: Color = Color::srgb(1.0, 0.2, 0.2);

/// Computes the number of spatial-hash cells needed to cover the play area.
/// `half_width` / `half_height` describe the visible half-extents; the grid is
/// centered at the origin, so total coverage is `2 * half` per axis. Partial
/// cells are rounded up so the grid always fully tiles the play area. Each axis
/// is floored to a minimum of 1 cell.
fn grid_dimensions(half_width: f32, half_height: f32, cell_size: f32) -> UVec2 {
    let safe_cell = cell_size.max(1.0);
    let cells_x = ((2.0 * half_width) / safe_cell).ceil() as u32;
    let cells_y = ((2.0 * half_height) / safe_cell).ceil() as u32;
    UVec2::new(cells_x.max(1), cells_y.max(1))
}

#[derive(Resource, Default)]
struct GridOverlay {
    visible: bool,
}

fn toggle_grid_overlay(keyboard: Res<ButtonInput<KeyCode>>, mut overlay: ResMut<GridOverlay>) {
    if keyboard.just_pressed(KeyCode::F2) {
        overlay.visible = !overlay.visible;
    }
}

fn draw_grid_overlay(
    overlay: Res<GridOverlay>,
    boundary: Res<CullBoundary>,
    config: Res<SpatialHashConfig>,
    mut gizmos: Gizmos,
) {
    if !overlay.visible {
        return;
    }

    let cells = grid_dimensions(boundary.half_width, boundary.half_height, config.cell_size);
    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            cells,
            Vec2::splat(config.cell_size),
            GREEN,
        )
        .outer_edges();
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.015, 0.02, 0.04)))
            .init_resource::<GridOverlay>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    sync_cull_boundary,
                    ensure_projectile_sprite,
                    ensure_enemy_sprite,
                ),
            )
            .add_systems(Update, (toggle_grid_overlay, draw_grid_overlay).chain());
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn((Name::new("Main Camera"), Camera2d));
    commands.spawn((
        Name::new("Ship"),
        Ship,
        PlayerTarget,
        Emitter {
            fire_rate: Timer::from_seconds(0.2, TimerMode::Repeating),
        },
        PlayerEmitter,
        Sprite::from_color(Color::srgb(0.2, 0.75, 1.0), SHIP_SIZE),
        Transform::default(),
    ));
}

fn ensure_projectile_sprite(
    mut commands: Commands,
    projectiles: Query<Entity, (With<Projectile>, Without<Sprite>)>,
) {
    for entity in &projectiles {
        commands
            .entity(entity)
            .insert(Sprite::from_color(PROJECTILE_COLOR, PROJECTILE_SIZE));
    }
}

fn ensure_enemy_sprite(
    mut commands: Commands,
    enemies: Query<Entity, (With<Enemy>, Without<Sprite>)>,
) {
    for entity in &enemies {
        commands
            .entity(entity)
            .insert(Sprite::from_color(ENEMY_COLOR, ENEMY_SIZE));
    }
}

fn sync_cull_boundary(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut boundary: ResMut<CullBoundary>,
) {
    if let Ok(window) = windows.single() {
        boundary.half_width = window.width() / 2.0;
        boundary.half_height = window.height() / 2.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec2;

    #[test]
    fn grid_dimensions_covers_play_area() {
        // 1280x720 play area, 128px cells -> 10 x 6 cells
        let dims = grid_dimensions(640.0, 360.0, 128.0);
        assert_eq!(dims, UVec2::new(10, 6));
    }

    #[test]
    fn grid_dimensions_rounds_up_partial_cells() {
        // 720 / 130 = 5.538 -> ceil = 6
        let dims = grid_dimensions(640.0, 360.0, 130.0);
        assert_eq!(dims.x, 10); // 1280/130 = 9.846 -> 10
        assert_eq!(dims.y, 6); // 720/130 = 5.538 -> 6
    }

    #[test]
    fn grid_dimensions_floors_to_minimum_one_cell() {
        let dims = grid_dimensions(10.0, 10.0, 128.0);
        assert_eq!(dims, UVec2::new(1, 1));
    }

    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn toggle_flips_visible_on_f2() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::F2);
        world.insert_resource(keyboard);
        world.init_resource::<GridOverlay>();
        world.run_system_once(toggle_grid_overlay).unwrap();
        assert!(
            world.resource::<GridOverlay>().visible,
            "F2 should make the grid overlay visible"
        );
    }

    #[test]
    fn toggle_ignores_other_keys() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::F1);
        world.insert_resource(keyboard);
        world.init_resource::<GridOverlay>();
        world.run_system_once(toggle_grid_overlay).unwrap();
        assert!(
            !world.resource::<GridOverlay>().visible,
            "F1 must not toggle the grid overlay"
        );
    }

    #[test]
    fn toggle_off_after_two_presses() {
        let mut world = World::new();
        let mut keyboard = ButtonInput::<KeyCode>::default();
        world.init_resource::<GridOverlay>();
        // First press
        keyboard.press(KeyCode::F2);
        world.insert_resource(keyboard.clone());
        world.run_system_once(toggle_grid_overlay).unwrap();
        assert!(
            world.resource::<GridOverlay>().visible,
            "first press should show"
        );
        // Update to clear just_pressed, then press again
        keyboard.release(KeyCode::F2);
        keyboard.clear_just_released(KeyCode::F2);
        keyboard.press(KeyCode::F2);
        world.insert_resource(keyboard);
        world.run_system_once(toggle_grid_overlay).unwrap();
        assert!(
            !world.resource::<GridOverlay>().visible,
            "second F2 press should hide the grid overlay"
        );
    }
}
