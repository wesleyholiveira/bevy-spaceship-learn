use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};

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

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<FpsCounter>()
            .add_systems(Startup, setup_ui)
            .add_systems(Update, update_fps_counter)
            .add_systems(bevy_egui::EguiPrimaryContextPass, fps_overlay_system);
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("Controls"),
        Text::new("WASD"),
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

fn fps_overlay_system(mut contexts: EguiContexts, counter: Res<FpsCounter>) {
    let ctx = contexts.ctx_mut().unwrap();
    egui::Area::new(egui::Id::new("fps_overlay"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(format!("FPS: {:.1}", counter.fps)).size(18.0));
        });
}
