use bevy::window::{PresentMode, WindowMode, WindowResolution};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;

mod keybinding;
mod resources;
mod screens;
mod ui;

pub use resources::assets;
pub use resources::audio;

pub use resources::audio::GameAudioPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Elysium Descent".into(),
                resolution: WindowResolution::new(1920.0, 1080.0).with_scale_factor_override(1.0),
                resizable: true,
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                mode: WindowMode::Windowed,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera)
        .add_plugins(UiLunexPlugins)
        .add_plugins(AudioPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins((screens::plugin, keybinding::plugin))
        .run()
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 2,
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
        UiSourceCamera::<0>,
        Transform::from_translation(Vec3::Z * 1000.0),
    ));
}
