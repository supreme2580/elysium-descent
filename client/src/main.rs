use bevy::window::{PresentMode, WindowMode, WindowResolution};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;
use dojo_bevy_plugin::{DojoResource, TokioRuntime};
// Removed unused import - PhysicsDebugPlugin is currently disabled

mod constants;
mod game;
mod keybinding;
mod rendering;
mod resources;
mod screens;
mod systems;
mod ui;

pub use resources::assets;
pub use resources::audio;

pub use resources::audio::{GameAudioPlugin, SfxPlugin};

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
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Dark background initially
        .add_plugins(UiLunexPlugins)
        .add_plugins(AudioPlugin)
        // .add_plugins(PhysicsDebugPlugin::default()) // Temporarily disabled for performance
        .init_resource::<DojoResource>()
        .init_resource::<TokioRuntime>()
        // .add_plugins(DojoPlugin) // Temporarily disabled for testing
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins(SfxPlugin)
        .add_plugins(game::LevelManagerPlugin)
        .add_event::<systems::dojo::pickup_item::PickupItemEvent>()
        .add_event::<systems::dojo::pickup_item::ItemPickedUpEvent>()
        .add_event::<systems::dojo::pickup_item::ItemPickupFailedEvent>()
        .add_plugins((screens::plugin, keybinding::plugin, /* dojo::plugin, */ ui::modal::ModalPlugin))
        .run()
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0,
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]),
        UiSourceCamera::<0>,
        Transform::from_translation(Vec3::Z * 1000.0),
        Name::new("UI Camera"),
    ));
}
