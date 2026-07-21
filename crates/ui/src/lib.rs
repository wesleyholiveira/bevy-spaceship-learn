use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use spaceship_core::GameConfig;
use spaceship_core::enemy::{EnemyPool, EnemyPoolStats};
use spaceship_core::projectile::{Active, Inactive, Projectile};

#[derive(Resource)]
struct FpsCounter {
    frames: usize,
    elapsed: f32,
    fps: f32,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            frames: 0,
            elapsed: 0.0,
            fps: 0.0,
        }
    }
}

#[derive(Resource, Default)]
struct OverlayState {
    visible: bool,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<FpsCounter>()
            .init_resource::<OverlayState>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_fps_counter, toggle_overlay))
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                (fps_overlay_system, debug_overlay_system),
            );
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("Controls"),
        Text::new("WASD | F1: Debug Overlay"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.92, 0.96)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Px(16.0),
            ..default()
        },
    ));
}

fn update_fps_counter(time: Res<Time>, mut counter: ResMut<FpsCounter>) {
    counter.frames += 1;
    counter.elapsed += time.delta_secs();
    if counter.elapsed >= 0.5 {
        counter.fps = counter.frames as f32 / counter.elapsed;
        counter.frames = 0;
        counter.elapsed = 0.0;
    }
}

fn toggle_overlay(keyboard: Res<ButtonInput<KeyCode>>, mut state: ResMut<OverlayState>) {
    if keyboard.just_pressed(KeyCode::F1) {
        state.visible = !state.visible;
    }
}

fn fps_overlay_system(mut contexts: EguiContexts, counter: Res<FpsCounter>) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Area::new(egui::Id::new("fps_overlay"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(format!("FPS: {:.1}", counter.fps)).size(18.0));
        });
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn debug_overlay_system(
    mut contexts: EguiContexts,
    state: Res<OverlayState>,
    mut config: ResMut<GameConfig>,
    active_query: Query<Entity, With<Active>>,
    inactive_query: Query<Entity, With<Inactive>>,
    projectile_query: Query<(Entity, &Transform), (With<Active>, With<Projectile>)>,
    pool: Res<EnemyPool>,
    pool_stats: Res<EnemyPoolStats>,
) {
    if !state.visible {
        return;
    }

    let ctx = contexts.ctx_mut().unwrap();
    let active_count = active_query.iter().count();
    let inactive_count = inactive_query.iter().count();

    egui::Window::new("Debug Overlay")
        .resizable(true)
        .default_width(400.0)
        .show(ctx, |ui| {
            ui.heading("Configuration");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Ship Speed:");
                ui.add(egui::DragValue::new(&mut config.ship_speed).speed(10.0));
            });

            ui.horizontal(|ui| {
                ui.label("Max Bullets:");
                let mut max_bullets = config.max_bullets as i32;
                if ui
                    .add(
                        egui::DragValue::new(&mut max_bullets)
                            .range(1..=10000)
                            .speed(10.0),
                    )
                    .changed()
                {
                    config.max_bullets = max_bullets as usize;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Cull Margin:");
                ui.add(egui::DragValue::new(&mut config.cull_margin).speed(5.0));
            });

            ui.add_space(10.0);
            ui.heading("Entity Statistics");
            ui.separator();

            ui.label(format!("Active: {}", active_count));
            ui.label(format!("Inactive: {}", inactive_count));
            ui.label(format!("Total: {}", active_count + inactive_count));

            ui.add_space(10.0);
            ui.heading("Enemy Pool");
            ui.separator();
            ui.label(format!("Available: {}", pool.available_count()));
            ui.label(format!("Failed spawns: {}", pool_stats.failed_spawns));
            ui.label(format!("Total releases: {}", pool_stats.total_releases));

            ui.add_space(10.0);
            ui.heading("Active Projectiles");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (entity, transform) in projectile_query.iter() {
                        let pos = transform.translation;
                        ui.collapsing(
                            format!("Entity {} ({:.1}, {:.1})", entity.index(), pos.x, pos.y),
                            |ui| {
                                ui.label(format!("ID: {}", entity.index()));
                                ui.label(format!(
                                    "Position: ({:.2}, {:.2}, {:.2})",
                                    pos.x, pos.y, pos.z
                                ));
                            },
                        );
                    }
                });
        });
}
