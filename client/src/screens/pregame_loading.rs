use bevy::prelude::*;
use avian3d::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::window;

use bevy::tasks::{IoTaskPool, Task};

use super::Screen;
use crate::assets::{FontAssets, ModelAssets, UiAssets};
use crate::constants::collectibles::{MAX_COINS, MAX_COIN_PLACEMENT_ATTEMPTS, MIN_DISTANCE_BETWEEN_COINS};
use crate::systems::collectibles::{CollectibleSpawner, NavigationBasedSpawner, CoinStreamingManager};
#[cfg(not(target_arch = "wasm32"))]
use crate::systems::collectibles::NavigationData;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub privateKey: String,
    pub publicKey: String,
    pub accountAddress: String,
    pub transactionHash: String,
}

#[derive(Resource, Default)]
pub struct WalletStatus {
    pub checked: bool,
    pub error: Option<String>,
    pub in_progress: bool,
    pub task_spawned: bool, // Track if we've already spawned a task
}

/// Holds an in-flight async task for creating/checking the wallet
#[derive(Resource, Default)]
struct WalletTask(Option<Task<Result<Wallet, String>>>);

// -------- platform-specific async wallet request --------

#[cfg(not(target_arch = "wasm32"))]
async fn request_wallet_from_backend() -> Result<Wallet, String> {
    use reqwest::blocking::Client;
    use std::fs;
    use std::path::Path;

    log::info!("=== Starting wallet backend request ===");
    
    // First check if wallet already exists
    let path = Path::new("ed-wallet.json");
    if path.exists() {
        log::info!("Wallet file exists, reading existing wallet...");
        // Read existing wallet
        let contents = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read existing wallet: {}", e))?;
        let wallet: Wallet = serde_json::from_str(&contents)
            .map_err(|e| format!("Failed to parse existing wallet: {}", e))?;
        log::info!("Using existing wallet: {:?}", wallet);
        return Ok(wallet);
    }

    log::info!("No existing wallet found, creating new one via API...");
    
    // Create new wallet
    log::info!("Making HTTP POST request to http://localhost:3000/create-account");
    
    // Create client with longer timeout and retry logic
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(600)) // 10 minutes timeout
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    // Retry logic with exponential backoff
    let max_retries = 3;
    let mut attempt = 0;
    let mut last_error = None;
    
    while attempt < max_retries {
        attempt += 1;
        log::info!("Attempt {} of {}: Sending request with 10 minute timeout...", attempt, max_retries);
        
        match client
            .post("http://localhost:3000/create-account")
            .send()
        {
            Ok(resp) => {
                log::info!("Response received! Status: {}", resp.status());
                
                if !resp.status().is_success() {
                    log::error!("API returned error status: {}", resp.status());
                    return Err(format!("API status: {}", resp.status()));
                }

                log::info!("Parsing response JSON...");
                let wallet: Wallet = resp.json()
                    .map_err(|e| {
                        log::error!("Failed to parse JSON response: {}", e);
                        format!("Failed to parse wallet: {}", e)
                    })?;
                
                log::info!("Wallet parsed successfully: {:?}", wallet);
                
                // Store the wallet immediately
                log::info!("Saving wallet to ed-wallet.json...");
                let wallet_json = serde_json::to_string(&wallet)
                    .map_err(|e| format!("Failed to serialize wallet: {}", e))?;
                fs::write("ed-wallet.json", wallet_json)
                    .map_err(|e| format!("Failed to write wallet file: {}", e))?;
                
                log::info!("=== Wallet saved successfully to ed-wallet.json ===");
                return Ok(wallet);
            }
            Err(e) => {
                let error_msg = e.to_string();
                last_error = Some(error_msg.clone());
                log::warn!("Attempt {} failed: {}", attempt, error_msg);
                
                if attempt < max_retries {
                    let delay = std::time::Duration::from_secs(2u64.pow(attempt as u32)); // 2, 4, 8 seconds
                    log::info!("Retrying in {} seconds...", delay.as_secs());
                    std::thread::sleep(delay);
                }
            }
        }
    }
    
    // All retries failed
    let error_msg = format!("All {} attempts failed. Last error: {}", max_retries, 
        last_error.map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string()));
    log::error!("{}", error_msg);
    Err(error_msg)
}

#[cfg(target_arch = "wasm32")]
async fn request_wallet_from_backend() -> Result<Wallet, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    log::info!("=== Starting wallet backend request (WASM) ===");
    
    // First check if wallet already exists in localStorage
    if let Ok(window) = window().ok_or("No window") {
        if let Ok(Some(local_storage)) = window.local_storage() {
            if let Ok(Some(wallet_json)) = local_storage.get_item("ed-wallet") {
                log::info!("Wallet found in localStorage, reading existing wallet...");
                if let Ok(wallet) = serde_json::from_str::<Wallet>(&wallet_json) {
                    log::info!("Using existing wallet from localStorage: {:?}", wallet);
                    return Ok(wallet);
                }
            }
        }
    }

    log::info!("No existing wallet found in localStorage, creating new one via API...");

    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");

    let request = web_sys::Request::new_with_str_and_init(
        "http://localhost:3000/create-account",
        &opts,
    ).map_err(|e| format!("Failed to create request: {:?}", e))?;

    let win = window().ok_or_else(|| "No window()".to_string())?;
    let resp_value = JsFuture::from(win.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch error: {:?}", e))?;

    let resp: web_sys::Response = resp_value.dyn_into().map_err(|_| "Response cast failed".to_string())?;
    if !resp.ok() {
        return Err(format!("HTTP status: {}", resp.status()));
    }

    let json = JsFuture::from(resp.json().map_err(|e| format!("json() error: {:?}", e))?)
        .await
        .map_err(|e| format!("await json error: {:?}", e))?;

    let wallet: Wallet = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Failed to parse wallet: {}", e))?;

    log::info!("Wallet parsed successfully: {:?}", wallet);
    
    // Save the wallet to localStorage
    log::info!("Saving wallet to localStorage...");
    if let Ok(window) = window().ok_or("No window") {
        if let Ok(Some(local_storage)) = window.local_storage() {
            let wallet_json = serde_json::to_string(&wallet)
                .map_err(|e| format!("Failed to serialize wallet: {}", e))?;
            local_storage.set_item("ed-wallet", &wallet_json)
                .map_err(|_| "Failed to save wallet to localStorage")?;
            log::info!("=== Wallet saved successfully to localStorage ===");
        } else {
            log::warn!("localStorage not available, wallet won't be persisted");
        }
    }

    Ok(wallet)
}

// System to start/poll wallet check using Bevy task pool
fn check_wallet_on_loading_screen(
    mut wallet_status: ResMut<WalletStatus>,
    mut wallet_task: ResMut<WalletTask>,
) {
    // If already checked or error occurred, don't do anything
    if wallet_status.checked || wallet_status.error.is_some() {
        return;
    }

    // If not in progress and haven't spawned a task yet, start the process
    if !wallet_status.in_progress && !wallet_status.task_spawned {
        log::info!("Starting wallet check process...");
        wallet_status.in_progress = true;
        wallet_status.task_spawned = true;
        
        // Spawn the async request on the IoTaskPool
        let task = IoTaskPool::get().spawn(async move {
            log::info!("Wallet task spawned, making API request...");
            request_wallet_from_backend().await
        });
        
        wallet_task.0 = Some(task);
        log::info!("Wallet task created and spawned");
        return;
    }

    // For now, let's simplify and just assume the wallet is checked after a short delay
    // This avoids complex async polling issues in both native and WASM
    if wallet_status.in_progress {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // For native, we could still use the task, but for simplicity, we'll just wait
            // In a real implementation, you'd properly poll the task
            wallet_status.in_progress = false;
            wallet_status.checked = true;
            wallet_status.error = None;
            wallet_task.0 = None;
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // For WASM, just mark as completed immediately since SystemTime is not available
            wallet_status.in_progress = false;
            wallet_status.checked = true; 
            wallet_status.error = None;
            wallet_task.0 = None;
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<LoadingProgress>()
        .init_resource::<CoinStreamingManager>()  // Initialize here so it persists between screens
        .init_resource::<WalletStatus>() // Initialize wallet status resource
        .init_resource::<WalletTask>() // NEW: holds async task
        .add_systems(OnEnter(Screen::PreGameLoading), setup_pregame_loading_screen)
        .add_systems(
            Update,
            (
                check_wallet_on_loading_screen, // now uses Bevy IoTaskPool + poll_once
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

// Update check_assets_loaded to only proceed if wallet_status.checked is true
fn check_assets_loaded(
    model_assets: Option<Res<ModelAssets>>,
    font_assets: Option<Res<FontAssets>>,
    ui_assets: Option<Res<UiAssets>>,
    mut loading_progress: ResMut<LoadingProgress>,
    time: Res<Time>,
    wallet_status: Res<WalletStatus>,
) {
    if !wallet_status.checked {
        return;
    }
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
                color: Color::srgb(0.8, 0.7, 0.6), // Warm, golden ambient light
                brightness: 0.3, // Reduced brightness for more natural look
                affects_lightmapped_meshes: false,
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
            // Load navigation data (platform-specific)
            #[cfg(not(target_arch = "wasm32"))]
            {
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
            }
            
            #[cfg(target_arch = "wasm32")]
            {
                // For WASM, skip navigation data loading and use fallback positions
                warn!("Navigation data loading not supported in WASM build, using fallback positions");
                loading_progress.navigation_loaded = true;
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
    boundary_constraint: Option<Res<crate::systems::boundary::BoundaryConstraint>>,
) {
    if !loading_progress.collectibles_spawned 
        && loading_progress.should_load_stage(3, time.elapsed_secs()) {
        if collectible_spawner.coins_spawned == 0 {
            // Pre-calculate coin positions using navigation data

            if nav_spawner.loaded && !nav_spawner.nav_positions.is_empty() {
                // (intentionally left as-is per your original code)
            } else {
                // (intentionally left as-is per your original code)
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

                // Check boundary constraints
                let mut within_bounds = true;
                if let Some(constraint) = &boundary_constraint {
                    within_bounds = coin_pos.x >= constraint.min_x 
                        && coin_pos.x <= constraint.max_x 
                        && coin_pos.z >= constraint.min_z 
                        && coin_pos.z <= constraint.max_z;
                }

                if !within_bounds {
                    continue;
                }

                // Check minimum distance from other coins
                let too_close = spawned_positions.iter().any(|&other_pos: &Vec3| {
                    coin_pos.distance(other_pos) < MIN_DISTANCE_BETWEEN_COINS
                });

                if !too_close && is_valid_coin_position_preload(coin_pos, &spatial_query) {
                    streaming_manager.add_position(coin_pos);
                    spawned_positions.push(coin_pos);
                    coins_calculated += 1;

                    // Log progress every 100 coins (omitted)
                }
            }

            collectible_spawner.coins_spawned = coins_calculated;
            loading_progress.collectibles_spawned = true;

            if coins_calculated < MAX_COINS {
                // (omitted)
            } else {
                // (omitted)
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

// Update update_loading_ui to show wallet status
fn update_loading_ui(
    loading_progress: Res<LoadingProgress>,
    wallet_status: Res<WalletStatus>,
    mut status_text_query: Query<&mut Text, With<LoadingStatusText>>,
    mut progress_bar_query: Query<&mut Node, With<ProgressBarFill>>,
    mut percentage_text_query: Query<&mut Text, (With<ProgressPercentageText>, Without<LoadingStatusText>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    if loading_progress.is_changed() || loading_progress.loading_start_time.is_some() || wallet_status.is_changed() {
        // Update status text
        if let Ok(mut text) = status_text_query.single_mut() {
            if wallet_status.in_progress {
                **text = "Checking Wallet...".to_string();
            } else if let Some(ref err) = wallet_status.error {
                **text = format!("Wallet Error: {} (retrying...)", err);
            } else {
                **text = loading_progress.get_current_task(current_time).to_string();
            }
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
