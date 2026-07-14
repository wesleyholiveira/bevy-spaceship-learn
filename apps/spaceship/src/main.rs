use bevy::prelude::*;
use bevy::window::WindowPlugin;
use spaceship_core::CorePlugin;
use spaceship_input::InputPlugin;
use spaceship_render::RenderPlugin;
use spaceship_ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Spaceship".into(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((CorePlugin, InputPlugin, RenderPlugin, UiPlugin))
        .run();
}
