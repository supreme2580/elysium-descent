use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;
use rand::prelude::*;

use super::{Screen, despawn_scene};
use super::pregame_loading::EnvironmentPreload;
use crate::assets::{FontAssets, ModelAssets, UiAssets};
use crate::constants::collectibles::{MAX_COINS, MIN_DISTANCE_BETWEEN_COINS};
use crate::keybinding;
use crate::systems::character_controller::{
    CharacterController, CharacterControllerBundle, CharacterControllerPlugin, setup_idle_animation,
};
use crate::systems::book_interaction::BookInteractionPlugin;
use crate::systems::collectibles::{CollectiblesPlugin, NavigationBasedSpawner, CollectibleSpawner, CoinStreamingManager};
use crate::systems::level_manager::LevelManagerPlugin;
use crate::systems::objectives::ObjectivesPlugin;
use crate::ui::dialog::DialogPlugin;
use crate::ui::inventory::spawn_inventory_ui;
use crate::ui::styles::ElysiumDescentColorPalette;
use crate::ui::widgets::{HudPosition, player_hud_widget};
use crate::ui::modal::despawn_modal;
use bevy_enhanced_input::prelude::*;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::GamePlay),
        (
            reveal_preloaded_environment,
            debug_streaming_manager_state,
            PlayingScene::spawn_player_and_camera,
            set_gameplay_clear_color,
        ),
    )
    .add_systems(
        Update,
        (
            camera_follow_player,
            // Fallback systems that run if preloaded entities weren't found
            fallback_spawn_environment,
            fallback_spawn_collectibles,
        ).run_if(in_state(Screen::GamePlay)),
    )
    .add_systems(
        OnExit(Screen::GamePlay),
        (despawn_scene::<PlayingScene>, despawn_gameplay_hud, cleanup_preloaded_environment, despawn_modal, despawn_collectibles),
    )
    .add_plugins(PhysicsPlugins::default())
    .add_plugins(CharacterControllerPlugin)
    .add_plugins(GltfAnimationPlugin)
    .add_plugins(CollectiblesPlugin)
    .add_plugins(LevelManagerPlugin)
    .add_plugins(ObjectivesPlugin)
    .add_plugins(DialogPlugin)
    .add_plugins(BookInteractionPlugin);
}

// ===== SYSTEMS =====

fn set_gameplay_clear_color(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::srgb(0.529, 0.808, 0.922))); // Sky blue color
}

fn debug_streaming_manager_state(streaming_manager: Res<CoinStreamingManager>) {
    
    if streaming_manager.positions.is_empty() {

    } else {

        
        // Show first few positions for debugging

    }
}

fn camera_follow_player(
    player_query: Query<&Transform, With<CharacterController>>,
    mut camera_query: Query<
        &mut Transform,
        (
            With<Camera3d>,
            With<PlayingScene>,
            Without<CharacterController>,
        ),
    >,
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
pub struct PlayingScene;

#[derive(Component)]
struct EnvironmentMarker;

#[derive(Component)]
struct GameplayHud;

fn spawn_player_hud(
    commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    ui_assets: &Res<UiAssets>,
) {
    // Example values, replace with actual player data
    let avatar = ui_assets.player_avatar.clone();
    let name = "0XJEHU";
    let level = 2;
    let health = (105, 115);
    let xp = (80, 100);
    let font = font_assets.rajdhani_bold.clone();

    commands.spawn((
        player_hud_widget(avatar, name, level, health, xp, font, HudPosition::Left),
        GameplayHud,
    ));
}

fn spawn_objectives_ui(
    commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    _ui_assets: &Res<UiAssets>,
) {

    
    // This will be populated by the ObjectiveManager, but for now we'll create a simple UI structure
    let font = font_assets.rajdhani_bold.clone();
    
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(32.0),
            right: Val::Px(32.0),
            width: Val::Px(480.0),
            height: Val::Auto, // Auto-size based on content
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::DARK_GLASS),
        BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.6)),
        BorderRadius::all(Val::Px(24.0)),
        Name::new("Objectives UI"),
        crate::systems::objectives::ObjectiveUI,
        GameplayHud,
        children![
            // Title
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(48.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(18.0)),
                    padding: UiRect::left(Val::Px(8.0)),
                    ..default()
                },
                children![(
                    Text::new("OBJECTIVES"),
                    TextFont {
                        font: font.clone(),
                        font_size: 27.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                )]
            ),
            // Objectives List Container - will be populated dynamically
            (
                Node {
                    width: Val::Percent(100.0),
                    // height: Val::Px(220.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Name::new("ObjectivesList"),
            )
        ],
    ));
}

fn despawn_gameplay_hud(mut commands: Commands, query: Query<Entity, With<GameplayHud>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn cleanup_preloaded_environment(
    mut commands: Commands, 
    environment_query: Query<Entity, With<EnvironmentPreload>>
) {
    for entity in environment_query.iter() {
        commands.entity(entity).despawn();
    }
    let count = environment_query.iter().count();
    if count > 0 {

    }
}

fn despawn_collectibles(mut commands: Commands, query: Query<Entity, With<crate::systems::collectibles::Collectible>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== PLAYING SCENE IMPLEMENTATION =====

impl PlayingScene {
    fn spawn_player_and_camera(
        mut commands: Commands,
        assets: Res<ModelAssets>,
        font_assets: Res<FontAssets>,
        ui_assets: Res<UiAssets>,
        windows: Query<&Window>,
    ) {
    

        // Add directional light (if not already added by preload)
        commands.spawn((
            Name::new("Directional Light"),
            DirectionalLight {
                illuminance: 80_000.0,
                shadows_enabled: true,
                ..default()
            },
            Transform::from_rotation(Quat::from_euler(
                EulerRot::XYZ,
                -std::f32::consts::FRAC_PI_3,
                std::f32::consts::FRAC_PI_4,
                0.0,
            )),
            PlayingScene,
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
                CollisionEventsEnabled, // Enable collision events for coin collection
                Actions::<keybinding::Player>::default(),
                PlayingScene,
            ))
            .observe(setup_idle_animation);

        // Add camera
        commands.spawn((
            Name::new("Gameplay Camera"),
            Camera3d::default(),
            Camera {
                order: 1,
                ..default()
            },
            Transform::from_xyz(0.0, 4.0, -12.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
            PlayingScene,
        ));

        spawn_inventory_ui::<PlayingScene>(&mut commands);
        spawn_player_hud(&mut commands, &font_assets, &ui_assets);
        spawn_objectives_ui(&mut commands, &font_assets, &ui_assets);
        crate::ui::modal::spawn_objectives_modal(&mut commands, &font_assets, &ui_assets);
        
        // Spawn the 'Press E to Open' dialog for Mystery Boxes
        use crate::ui::dialog::{spawn_dialog, DialogConfig, DialogPosition};
        spawn_dialog(
            &mut commands,
            &font_assets,
            windows,
            DialogConfig {
                text: "Press E to Open".to_string(),
                position: DialogPosition::BottomCenter { bottom_margin: 4.0 },
                ..Default::default()
            },
            PlayingScene,
        );

    
    }
}

fn reveal_preloaded_environment(
    mut commands: Commands,
    environment_query: Query<Entity, With<EnvironmentPreload>>,
) {

    
    let mut revealed_count = 0;
    for entity in environment_query.iter() {
        commands.entity(entity)
            .insert(Visibility::Visible)
            .insert(PlayingScene);
        revealed_count += 1;
        
    }
    
    if revealed_count == 0 {

    } else {
    
    }
}

// Removed: No longer using preloaded collectibles - using streaming system instead



fn fallback_spawn_environment(
    mut commands: Commands,
    assets: Option<Res<ModelAssets>>,
    environment_query: Query<Entity, With<PlayingScene>>,
    environment_preload_query: Query<Entity, With<EnvironmentPreload>>,
    mut fallback_spawned: Local<bool>,
) {
    // Only run once, and only if no environment entities exist (neither preloaded nor PlayingScene)
    if *fallback_spawned || !environment_query.is_empty() || !environment_preload_query.is_empty() {
        return;
    }

    if let Some(assets) = assets {

        
        // Set up ambient light
        commands.insert_resource(AmbientLight {
            color: Color::srgb_u8(68, 71, 88),
            brightness: 120.0,
            ..default()
        });

        // Environment
        commands.spawn((
            Name::new("Fallback Environment"),
            SceneRoot(assets.environment.clone()),
            Transform {
                translation: Vec3::new(0.0, -1.5, 0.0),
                rotation: Quat::from_rotation_y(-core::f32::consts::PI * 0.5),
                scale: Vec3::splat(0.05),
            },
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
            PlayingScene,
        ));

        *fallback_spawned = true;

    }
}

fn fallback_spawn_collectibles(
    mut commands: Commands,
    assets: Option<Res<ModelAssets>>,
    nav_spawner: Option<Res<NavigationBasedSpawner>>,
    mut collectible_spawner: ResMut<CollectibleSpawner>,
    collectible_query: Query<Entity, With<crate::systems::collectibles::Collectible>>,
    _spatial_query: SpatialQuery,
    mut fallback_spawned: Local<bool>,
) {
    // Only run once, and only if no collectible entities exist
    if *fallback_spawned || !collectible_query.is_empty() || collectible_spawner.coins_spawned > 0 {
        return;
    }

    if let (Some(assets), Some(nav_spawner)) = (assets, nav_spawner) {
        if nav_spawner.loaded {
    
            
            let mut rng = rand::rng();
            let mut spawned_positions = Vec::new();
            let mut coins_spawned = 0;

            for nav_pos in &nav_spawner.nav_positions {
                if rng.random::<f32>() > nav_spawner.spawn_probability {
                    continue;
                }

                let angle = rng.random::<f32>() * std::f32::consts::TAU;
                let distance = rng.random::<f32>() * 8.0; // Use reasonable default radius
                let offset = Vec3::new(
                    angle.cos() * distance,
                    0.0,
                    angle.sin() * distance,
                );
                let potential_pos = *nav_pos + offset;

                let too_close = spawned_positions.iter().any(|&other_pos: &Vec3| {
                    potential_pos.distance(other_pos) < MIN_DISTANCE_BETWEEN_COINS
                });

                if too_close {
                    continue;
                }

                let coin_y = if potential_pos.y + 2.5 <= -1.5 {
                    1.0
                } else {
                    potential_pos.y + 2.5
                };
                let coin_pos = Vec3::new(potential_pos.x, coin_y, potential_pos.z);
                
                spawn_fallback_collectible(
                    &mut commands,
                    &assets,
                    coin_pos,
                );

                spawned_positions.push(coin_pos);
                coins_spawned += 1;

                if coins_spawned >= MAX_COINS {
                    break;
                }
            }

            collectible_spawner.coins_spawned = coins_spawned;
            *fallback_spawned = true;
    
        }
    }
}

fn spawn_fallback_collectible(
    commands: &mut Commands,
    assets: &Res<ModelAssets>,
    position: Vec3,
) {
    use crate::systems::collectibles::{
        Collectible, CollectibleType, FloatingItem, CollectibleRotation, 
        Sensor
    };

    // Create a compound collider that better approximates a coin shape
    // This is more performant than mesh-fitted colliders while still being more accurate than a single sphere
    let coin_collider = Collider::compound(vec![
        // Main body - slightly flattened sphere
        (Vec3::ZERO, Quat::IDENTITY, Collider::sphere(0.4)),
        // Edge rings for better coin-like collision
        (Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY, Collider::cylinder(0.4, 0.1)),
    ]);

    commands.spawn((
        Name::new("Fallback Coin"),
        SceneRoot(assets.coin.clone()),
        Transform {
            translation: position,
            scale: Vec3::splat(0.75),
            ..default()
        },
        coin_collider,
        RigidBody::Kinematic,
        Visibility::Visible,
        Collectible,
        CollectibleType::Coin,
        FloatingItem {
            base_height: position.y,
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        CollectibleRotation {
            enabled: true,
            clockwise: true,
            speed: 1.0,
        },
        Sensor,
        CollisionEventsEnabled, // Enable collision events for this coin
        PlayingScene,
    ));
}
