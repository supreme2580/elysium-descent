use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;

use super::{Screen, despawn_scene};
use crate::assets::ModelAssets;
use crate::systems::character_controller::{
    CharacterController, CharacterControllerBundle, CharacterControllerPlugin, setup_idle_animation,
};
use crate::keybinding;
use bevy_enhanced_input::prelude::*;
use crate::systems::collectibles::{CollectiblesPlugin, spawn_collectible, spawn_interactable_book, CollectibleType};
use crate::systems::collectibles_config::COLLECTIBLES;
use crate::ui::inventory::spawn_inventory_ui;
use crate::ui::dialog::{DialogPlugin, spawn_dialog, create_book_dialog};
use crate::assets::FontAssets;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GamePlay), (PlayingScene::spawn_environment, set_gameplay_clear_color, spawn_book_dialog))
        .add_systems(Update, camera_follow_player.run_if(in_state(Screen::GamePlay)))
        .add_systems(OnExit(Screen::GamePlay), despawn_scene::<PlayingScene>)
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin::default())
        .add_plugins(CharacterControllerPlugin)
        .add_plugins(GltfAnimationPlugin)
        .add_plugins(CollectiblesPlugin)
        .add_plugins(DialogPlugin);
}

// ===== SYSTEMS =====

fn set_gameplay_clear_color(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb(0.529, 0.808, 0.922))); // Sky blue color
}

fn camera_follow_player(
    player_query: Query<&Transform, With<CharacterController>>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, With<PlayingScene>, Without<CharacterController>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            let player_pos = player_transform.translation;
            let player_rotation = player_transform.rotation;

            // Calculate camera position behind player (inverted Z)
            let camera_offset = player_rotation * Vec3::new(0.0, 4.0, -12.0);
            let target_pos = player_pos + camera_offset;

            // Smoothly move camera to new position
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, (5.0 * time.delta_secs()).min(1.0));

            // Make camera look at player
            camera_transform.look_at(player_pos + Vec3::Y * 2.0, Vec3::Y);
        }
    }
}

#[derive(Component, Default, Clone)]
struct PlayingScene;

#[derive(Component)]
struct EnvironmentMarker;

fn spawn_book_dialog(
    commands: Commands,
    font_assets: Res<FontAssets>,
    windows: Query<&Window>,
) {
    let dialog_config = create_book_dialog();
    spawn_dialog(commands, font_assets, windows, dialog_config, PlayingScene);
}

// ===== PLAYING SCENE IMPLEMENTATION =====

impl PlayingScene {
    fn spawn_environment(mut commands: Commands, assets: Res<ModelAssets>) {
        // Set up ambient light
        commands.insert_resource(AmbientLight {
            color: Color::srgb_u8(68, 71, 88),
            brightness: 120.0,
            ..default()
        });

        // Environment (see the `collider_constructors` example for creating colliders from scenes)
        let scene_handle = assets.environment.clone();
        commands.spawn((
            Name::new("Environment"),
            EnvironmentMarker,
            PlayingScene, // Add scene marker to ensure cleanup
            SceneRoot(scene_handle),
            Transform {
                translation: Vec3::new(0.0, -1.5, 0.0),
                rotation: Quat::from_rotation_y(-core::f32::consts::PI * 0.5),
                scale: Vec3::splat(0.05), // Scale environment down
            },
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
            //DebugRender::default(),
        ));

        // Add directional light
        commands.spawn((
            Name::new("Directional Light"),
            DirectionalLight {
                illuminance: 80_000.0, // bright midday sun
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_3,
                std::f32::consts::FRAC_PI_4,
                0.0,
            )),
            PlayingScene, // Add scene marker to ensure cleanup
        ));

        // Add player
        commands
            .spawn((
                Name::new("Player"),
                GltfSceneRoot::new(assets.player.clone()),
                Transform {
                    translation: Vec3::new(0.0, 2.0, 0.0),
                    scale: Vec3::splat(4.0),
                    ..default()
                },
                CharacterControllerBundle::new(),
                Friction::new(0.5),
                Restitution::new(0.0),
                GravityScale(1.0),
                // Add enhanced input actions for this player
                Actions::<keybinding::Player>::default(),
                PlayingScene, // Add scene marker to ensure cleanup
                // DebugRender::default(),
            ))
            .observe(setup_idle_animation);

        // Spawn collectibles using imported array
        for config in COLLECTIBLES.iter() {
            match config.collectible_type {
                CollectibleType::Book => {
                    // Use the special interactable book spawning function
                    spawn_interactable_book(
                        &mut commands,
                        &assets,
                        config.position,
                        config.scale,
                        config.on_collect.clone(),
                        PlayingScene,
                    );
                }
                _ => {
                    // Use normal collectible spawning for other items
                    spawn_collectible(&mut commands, &assets, config.clone(), PlayingScene);
                }
            }
        }

        // Add camera
        commands.spawn((
            Name::new("Gameplay Camera"),
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform::from_xyz(0.0, 4.0, -12.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
            PlayingScene, // Add scene marker to ensure cleanup
        ));

        spawn_inventory_ui::<PlayingScene>(commands);
    }
}
