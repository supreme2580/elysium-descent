use bevy::window::{PresentMode, WindowMode, WindowResolution};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;
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
        .add_systems(Startup, (setup_camera, setup_global_lighting))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Dark background initially
        .add_plugins(UiLunexPlugins)
        .add_plugins(AudioPlugin)
        // .add_plugins(PhysicsDebugPlugin::default()) // Temporarily disabled for performance
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(GameAudioPlugin)
        .add_plugins(SfxPlugin)
        .add_event::<systems::collectibles::PickupItemEvent>()
        .add_plugins((screens::plugin, keybinding::plugin, ui::modal::ModalPlugin))
        .run()
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0,
            viewport: None,
            is_active: true,
            computed: Default::default(),
            target: Default::default(),
            hdr: false,
            output_mode: Default::default(),
            msaa_writeback: Default::default(),
            clear_color: Default::default(),
            sub_camera_view: None,
        },
        RenderLayers::from_layers(&[0, 1]),
        UiSourceCamera::<0>,
        Transform::from_translation(Vec3::Z * 1000.0),
        Name::new("UI Camera"),
    ));
}

fn setup_global_lighting(mut commands: Commands) {
    // Set up global lighting configuration for warmer, more realistic lighting
    // This creates a foundation of warm, golden ambient light that fills the entire scene
    // with a natural, atmospheric glow instead of harsh, cold lighting
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.8, 0.7, 0.6), // Warm, golden ambient light
        brightness: 0.3, // Reduced brightness for more natural look
        affects_lightmapped_meshes: false,
    });
}
