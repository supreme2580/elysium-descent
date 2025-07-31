use bevy::prelude::*;
use avian3d::prelude::*;
use rand::Rng;
use std::fs;

use super::Screen;
use crate::assets::{FontAssets, ModelAssets, UiAssets};
use crate::constants::collectibles::{MAX_COINS, MAX_COIN_PLACEMENT_ATTEMPTS, MIN_DISTANCE_BETWEEN_COINS};
use crate::systems::collectibles::{CollectibleSpawner, NavigationBasedSpawner, NavigationData, CoinStreamingManager};

#[derive(Component)]
struct PreGameLoadingScreen;

#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub assets_loaded: bool,
    pub environment_spawned: bool,
    pub navigation_loaded: bool,
    pub collectibles_spawned: bool,
    pub game_initialized: bool,
    pub loading_complete: bool,
    pub loading_start_time: Option<f32>,
    pub minimum_loading_time: f32,
    pub stage_durations: [f32; 5], // Duration for each loading stage
}

impl LoadingProgress {
    pub fn new() -> Self {
        Self {
            minimum_loading_time: 5.0, // 5 second total loading time
            stage_durations: [1.0, 1.0, 1.0, 1.0, 1.0], // 1 second per stage
            ..Default::default()
        }
    }

    pub fn is_ready(&self) -> bool {
        self.assets_loaded 
            && self.environment_spawned 
            && self.navigation_loaded 
            && self.collectibles_spawned 
            && self.game_initialized
    }

    pub fn can_transition(&self, current_time: f32) -> bool {
        if let Some(start_time) = self.loading_start_time {
            let elapsed = current_time - start_time;
            self.is_ready() && elapsed >= self.minimum_loading_time
        } else {
            false
        }
    }

    pub fn should_load_stage(&self, stage: usize, current_time: f32) -> bool {
        if let Some(start_time) = self.loading_start_time {
            let elapsed = current_time - start_time;
            let stage_start_time: f32 = self.stage_durations.iter().take(stage).sum();
            elapsed >= stage_start_time
        } else {
            false
        }
    }

    pub fn get_progress_percentage(&self, current_time: f32) -> f32 {
        if let Some(start_time) = self.loading_start_time {
            let elapsed = current_time - start_time;
            let progress = (elapsed / self.minimum_loading_time).min(1.0);
            progress * 100.0
        } else {
            0.0
        }
    }

    pub fn get_current_task(&self, current_time: f32) -> &'static str {
        if let Some(start_time) = self.loading_start_time {
            let elapsed = current_time - start_time;
            match elapsed {
                t if t < 1.0 => "Loading Assets...",
                t if t < 2.0 => "Spawning Environment...",
                t if t < 3.0 => "Loading Navigation Data...",
                t if t < 4.0 => "Spawning Collectibles...",
                t if t < 5.0 => "Initializing Game...",
                _ => "Ready! Starting game...",
            }
        } else {
            "Loading Assets..."
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<LoadingProgress>()
        .init_resource::<CoinStreamingManager>()  // Initialize here so it persists between screens
        .add_systems(OnEnter(Screen::PreGameLoading), setup_pregame_loading_screen)
        .add_systems(
            Update,
            (
                check_assets_loaded,
                spawn_environment_system,
                load_navigation_system,
                spawn_collectibles_system,
                initialize_game_system,
                check_loading_complete,
                update_loading_ui,
            ).run_if(in_state(Screen::PreGameLoading))
        )
        .add_systems(OnExit(Screen::PreGameLoading), cleanup_pregame_loading_only);
}

fn setup_pregame_loading_screen(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    mut loading_progress: ResMut<LoadingProgress>,
    mut streaming_manager: ResMut<CoinStreamingManager>,
    mut collectible_spawner: ResMut<CollectibleSpawner>,
    mut nav_spawner: ResMut<NavigationBasedSpawner>,
    time: Res<Time>,
) {
    // Reset all loading-related resources to prevent hanging on re-entry
    *loading_progress = LoadingProgress::new();
    loading_progress.loading_start_time = Some(time.elapsed_secs());
    
    // Reset streaming manager to clear old coin positions and spawned state
    *streaming_manager = CoinStreamingManager::default();
    
    // Reset collectible spawner
    collectible_spawner.coins_spawned = 0;
    
    // Reset navigation spawner loaded state to force reload
    nav_spawner.loaded = false;



    commands
        .spawn((
            PreGameLoadingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            // Background image
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode::new(ui_assets.background.clone()),
                BackgroundColor(Color::WHITE.with_alpha(0.3)),
            ));

            // Title
            parent.spawn((
                Text::new("ELYSIUM DESCENT"),
                TextFont {
                    font: font_assets.rajdhani_bold.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(50.0)),
                    ..default()
                },
            ));

            // Loading status container
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
            )).with_children(|parent| {
                // Loading text
                parent.spawn((
                    Text::new("Loading Assets..."),
                    TextFont {
                        font: font_assets.rajdhani_medium.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    LoadingStatusText,
                ));

                // Progress bar background
                parent.spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(20.0),
                        margin: UiRect::top(Val::Px(20.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::WHITE),
                    BackgroundColor(Color::BLACK.with_alpha(0.5)),
                )).with_children(|parent| {
                    // Progress bar fill
                    parent.spawn((
                        Node {
                            width: Val::Percent(0.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
                        ProgressBarFill,
                    ));
                });

                // Progress percentage
                parent.spawn((
                    Text::new("0%"),
                    TextFont {
                        font: font_assets.rajdhani_medium.clone(),  // Use rajdhani_medium instead
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::top(Val::Px(10.0)),
                        ..default()
                    },
                    ProgressPercentageText,
                ));
            });
        });
}

#[derive(Component)]
struct LoadingStatusText;

#[derive(Component)]
struct ProgressBarFill;

#[derive(Component)]
struct ProgressPercentageText;

fn check_assets_loaded(
    model_assets: Option<Res<ModelAssets>>,
    font_assets: Option<Res<FontAssets>>,
    ui_assets: Option<Res<UiAssets>>,
    mut loading_progress: ResMut<LoadingProgress>,
    time: Res<Time>,
) {
    if !loading_progress.assets_loaded && loading_progress.should_load_stage(0, time.elapsed_secs()) {
        if model_assets.is_some() && font_assets.is_some() && ui_assets.is_some() {
            loading_progress.assets_loaded = true;

        }
    }
}

fn spawn_environment_system(
    mut commands: Commands,
    assets: Option<Res<ModelAssets>>,
    mut loading_progress: ResMut<LoadingProgress>,
    time: Res<Time>,
) {
    if loading_progress.assets_loaded 
        && !loading_progress.environment_spawned 
        && loading_progress.should_load_stage(1, time.elapsed_secs()) {
        if let Some(assets) = assets {
            // Pre-spawn environment in background (hidden)


            // Set up ambient light
            commands.insert_resource(AmbientLight {
                color: Color::srgb_u8(68, 71, 88),
                brightness: 120.0,
                ..default()
            });

            // Environment
            commands.spawn((
                Name::new("PreLoaded Environment"),
                SceneRoot(assets.environment.clone()),
                Transform {
                    translation: Vec3::new(0.0, -1.5, 0.0),
                    rotation: Quat::from_rotation_y(-core::f32::consts::PI * 0.5),
                    scale: Vec3::splat(0.05),
                },
                ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
                RigidBody::Static,
                Visibility::Hidden, // Hide until gameplay starts
                EnvironmentPreload,
            ));

            loading_progress.environment_spawned = true;

        }
    }
}

#[derive(Component)]
pub struct EnvironmentPreload;

fn load_navigation_system(
    mut nav_spawner: ResMut<NavigationBasedSpawner>,
    mut loading_progress: ResMut<LoadingProgress>,
    time: Res<Time>,
) {
    if loading_progress.environment_spawned 
        && !loading_progress.navigation_loaded 
        && loading_progress.should_load_stage(2, time.elapsed_secs()) {
        if !nav_spawner.loaded {
            match fs::read_to_string("nav.json") {
                Ok(contents) => {
                                            match serde_json::from_str::<NavigationData>(&contents) {
                        Ok(nav_data) => {
                            nav_spawner.nav_positions = nav_data.positions
                                .iter()
                                .map(|point| Vec3::new(point.position[0], point.position[1], point.position[2]))
                                .collect();
                            
                            nav_spawner.loaded = true;
                            loading_progress.navigation_loaded = true;

                        }
                        Err(e) => {
                            error!("Failed to parse nav.json: {}", e);
                            // Continue without navigation data
                            loading_progress.navigation_loaded = true;
                        }
                    }
                }
                Err(e) => {
                    warn!("Could not load nav.json (file may not exist yet): {}", e);
                    // Continue without navigation data
                    loading_progress.navigation_loaded = true;
                }
            }
        } else {
            loading_progress.navigation_loaded = true;
        }
    }
}

fn spawn_collectibles_system(
    nav_spawner: Res<NavigationBasedSpawner>,
    mut collectible_spawner: ResMut<CollectibleSpawner>,
    mut streaming_manager: ResMut<CoinStreamingManager>,
    mut loading_progress: ResMut<LoadingProgress>,
    spatial_query: SpatialQuery,
    time: Res<Time>,
) {
    if !loading_progress.collectibles_spawned 
        && loading_progress.should_load_stage(3, time.elapsed_secs()) {
        if collectible_spawner.coins_spawned == 0 {
            // Pre-calculate coin positions using navigation data

            
            if nav_spawner.loaded && !nav_spawner.nav_positions.is_empty() {

            } else {

            }
            
            let mut rng = rand::rng();
            let mut spawned_positions = Vec::new();
            let mut coins_calculated = 0;
            let mut attempts = 0;

            while coins_calculated < MAX_COINS && attempts < MAX_COIN_PLACEMENT_ATTEMPTS {
                attempts += 1;

                // Use navigation positions if available, otherwise generate fallback positions
                let base_pos = if nav_spawner.loaded && !nav_spawner.nav_positions.is_empty() {
                    // Use actual navigation data
                    nav_spawner.nav_positions[rng.random_range(0..nav_spawner.nav_positions.len())]
                } else {
                    // Generate fallback positions closer to spawn
                    Vec3::new(
                        rng.random_range(-60.0..60.0), // Reasonable range around spawn
                        2.0, // Above ground for visibility
                        rng.random_range(-60.0..60.0), // Reasonable range around spawn
                    )
                };
                
                // Add some randomness around the navigation position
                let offset_x = rng.random_range(-5.0..5.0);
                let offset_z = rng.random_range(-5.0..5.0);
                let coin_pos = Vec3::new(
                    base_pos.x + offset_x,
                    base_pos.y.max(1.5), // Ensure above ground
                    base_pos.z + offset_z,
                );

                // Check minimum distance from other coins
                let too_close = spawned_positions.iter().any(|&other_pos: &Vec3| {
                    coin_pos.distance(other_pos) < MIN_DISTANCE_BETWEEN_COINS
                });

                if !too_close && is_valid_coin_position_preload(coin_pos, &spatial_query) {
                    streaming_manager.add_position(coin_pos);
                    spawned_positions.push(coin_pos);
                    coins_calculated += 1;

                    // Log progress every 100 coins

                }
            }

            collectible_spawner.coins_spawned = coins_calculated;
            loading_progress.collectibles_spawned = true;

            if coins_calculated < MAX_COINS {

            } else {

            }
            

        }
    }
}

// Removed: No longer pre-spawning collectible entities

fn is_valid_coin_position_preload(
    position: Vec3,
    spatial_query: &SpatialQuery,
) -> bool {
    let coin_radius = 0.2;
    let check_radius = coin_radius + 0.05;
    
    let intersection_filter = SpatialQueryFilter::default()
        .with_mask(LayerMask::ALL);
    
    let intersections = spatial_query.shape_intersections(
        &Collider::sphere(check_radius),
        position,
        Quat::IDENTITY,
        &intersection_filter,
    );
    
    intersections.len() <= 5
}

fn initialize_game_system(
    mut loading_progress: ResMut<LoadingProgress>,
    time: Res<Time>,
) {
    if loading_progress.collectibles_spawned 
        && !loading_progress.game_initialized 
        && loading_progress.should_load_stage(4, time.elapsed_secs()) {
        // Perform any final game initialization

        
        // Add any additional initialization logic here
        
        loading_progress.game_initialized = true;

    }
}

fn check_loading_complete(
    mut loading_progress: ResMut<LoadingProgress>,
    mut next_state: ResMut<NextState<Screen>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    if loading_progress.can_transition(current_time) && !loading_progress.loading_complete {
        loading_progress.loading_complete = true;
        
        if let Some(start_time) = loading_progress.loading_start_time {
            let _elapsed = current_time - start_time;
            // Removed logging statement
        }
        
        next_state.set(Screen::GamePlay);
    } else if loading_progress.is_ready() && loading_progress.loading_start_time.is_some() {
        let start_time = loading_progress.loading_start_time.unwrap();
        let elapsed = current_time - start_time;
        let remaining = loading_progress.minimum_loading_time - elapsed;
        
        if remaining > 0.0 && !loading_progress.loading_complete {
            // Show "Ready!" but still waiting for minimum time

        }
    }
}

fn cleanup_pregame_loading_only(
    mut commands: Commands,
    loading_ui_query: Query<Entity, With<PreGameLoadingScreen>>,
) {
    // Only clean up the loading UI, NOT the preloaded game entities
    for entity in loading_ui_query.iter() {
        commands.entity(entity).despawn();
    }

}

fn update_loading_ui(
    loading_progress: Res<LoadingProgress>,
    mut status_text_query: Query<&mut Text, With<LoadingStatusText>>,
    mut progress_bar_query: Query<&mut Node, With<ProgressBarFill>>,
    mut percentage_text_query: Query<&mut Text, (With<ProgressPercentageText>, Without<LoadingStatusText>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    if loading_progress.is_changed() || loading_progress.loading_start_time.is_some() {
        // Update status text
        if let Ok(mut text) = status_text_query.single_mut() {
            **text = loading_progress.get_current_task(current_time).to_string();
        }

        // Update progress bar
        if let Ok(mut node) = progress_bar_query.single_mut() {
            node.width = Val::Percent(loading_progress.get_progress_percentage(current_time));
        }

        // Update percentage text
        if let Ok(mut text) = percentage_text_query.single_mut() {
            **text = format!("{:.0}%", loading_progress.get_progress_percentage(current_time));
        }
    }
} 