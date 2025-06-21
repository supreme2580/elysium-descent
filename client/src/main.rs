use bevy::window::{PresentMode, WindowMode, WindowResolution};
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_kira_audio::prelude::*;
use bevy_lunex::prelude::*;
use bevy_yarnspinner::prelude::*;
// use bevy_yarnspinner_example_dialogue_view::ExampleYarnSpinnerDialogueViewPlugin;
use dojo_bevy_plugin::{DojoPlugin, DojoResource, TokioRuntime};

mod constants;
mod game;
mod keybinding;
mod rendering;
mod resources;
mod screens;
mod systems;
mod ui;

use systems::dialogue_view::SimpleDialogueViewPlugin;

use systems::dojo;

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
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.1))) // Dark background initially
        .add_plugins(UiLunexPlugins)
        .add_plugins(AudioPlugin)
        .add_plugins(YarnSpinnerPlugin::with_yarn_source(YarnFileSource::file(
            "dialogue/books.yarn",
        )))
        .add_plugins(SimpleDialogueViewPlugin) // Our custom dialogue view - no auto-start
        .init_resource::<DojoResource>()
        .init_resource::<TokioRuntime>()
        .add_plugins(DojoPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(GameAudioPlugin)
        .add_systems(
            Update,
            (
                // Spawn the dialogue runner once the Yarn project has finished compiling
                spawn_dialogue_runner.run_if(resource_added::<YarnProject>),
                // Debug yarn project compilation
                debug_yarn_project_loading,
            ),
        )
        .add_plugins((screens::plugin, keybinding::plugin, dojo::plugin))
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

fn spawn_dialogue_runner(mut commands: Commands, project: Res<YarnProject>) {
    // Create a dialogue runner from the project.
    let dialogue_runner = project.create_dialogue_runner(&mut commands);
    // Don't start any dialogue immediately - wait for book interaction
    commands.spawn((dialogue_runner, Name::new("Book Dialogue Runner")));
    // info!("✅ DialogueRunner created and ready for book interactions - NO DIALOGUE STARTED");
    
    // Log available dialogue nodes for debugging
    // warn!("🔍 YarnProject loaded successfully! Checking for available nodes...");
    // Note: YarnProject doesn't expose a get_node_names() method
    // Instead, we'll try to start specific nodes to test availability
    // warn!("  📄 Looking for node: 'Ancient_Tome' - this should be available for book interactions");
    
    // warn!("📚 Looking for 'Ancient_Tome' node - this should be available for book interactions");
}

/// System to debug YarnProject loading status
fn debug_yarn_project_loading(
    _yarn_project: Option<Res<YarnProject>>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
) {
    *debug_timer += time.delta_secs();
    
    // Only log every 10 seconds to avoid spam
    if *debug_timer > 10.0 {
        *debug_timer = 0.0;
        
        // match yarn_project {
        //     Some(_project) => {
        //         info!("✅ YarnProject loaded successfully");
        //         // Note: YarnProject doesn't expose node count or node iteration methods
        //         info!("   Ready to start dialogue with specific node names");
        //     }
        //     None => {
        //         warn!("❌ YarnProject not loaded yet - this is why dialogue won't work!");
        //         warn!("   Possible causes:");
        //         warn!("   - Yarn file not found at specified path");
        //         warn!("   - Yarn file has compilation errors");
        //         warn!("   - YarnSpinner plugin not properly configured");
        //     }
        // }
    }
}
