use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use avian3d::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::constants::collectibles::COIN_STREAMING_RADIUS;
use crate::screens::Screen;
use crate::systems::character_controller::CharacterController;
use crate::assets::ModelAssets;
use crate::resources::audio::{PlaySfxEvent, SfxType};



// ===== RESOURCES =====

/// Resource to track collectible progress for objectives
#[derive(Resource, Default)]
pub struct CollectibleProgressTracker {
    pub coins_collected: u32,
    pub books_collected: u32,
    pub health_potions_collected: u32,
    pub survival_kits_collected: u32,
}

// ===== COMPONENTS & RESOURCES =====

#[derive(Component)]
pub struct Collectible;

#[derive(Component)]
pub struct Collected;

#[derive(Component, Clone)]
pub struct CollectibleRotation {
    pub enabled: bool,
    pub clockwise: bool,
    pub speed: f32,
}

#[derive(Component)]
pub struct FloatingItem {
    pub base_height: f32,
    pub hover_amplitude: f32,
    pub hover_speed: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CollectibleType {
    Coin,
    HealthPotion,
    SurvivalKit,
    Book,
}

#[derive(Resource)]
pub struct NextItemToAdd(pub CollectibleType);



#[derive(Resource)]
pub struct CollectibleSpawner {
    pub coins_spawned: usize,
}

impl Default for CollectibleSpawner {
    fn default() -> Self {
        Self {
            coins_spawned: 0,
        }
    }
}

#[derive(Component)]
pub struct Sensor;

/// Component to mark streaming collectibles with their original position ID
#[derive(Component)]
pub struct StreamingCoin {
    pub position_id: usize,
}

/// Resource containing all coin positions and their spawned state
#[derive(Resource)]
pub struct CoinStreamingManager {
    pub positions: Vec<Vec3>,
    pub spawned_coins: HashMap<usize, Entity>,
    pub collected_positions: HashSet<usize>,  // Track collected positions to prevent respawning
    pub last_update_time: f32,
    pub update_interval: f32,
    pub spawn_radius: f32,
}

impl Default for CoinStreamingManager {
    fn default() -> Self {
        Self {
            positions: Vec::new(),
            spawned_coins: HashMap::new(),
            collected_positions: HashSet::new(),
            last_update_time: 0.0,
            update_interval: 1.0,
            spawn_radius: COIN_STREAMING_RADIUS,   // Use centralized constant
        }
    }
}

impl CoinStreamingManager {

    pub fn add_position(&mut self, position: Vec3) {
        self.positions.push(position);
    }

    pub fn should_update(&self, current_time: f32) -> bool {
        // Always update on first run (when last_update_time is 0.0)
        self.last_update_time == 0.0 || current_time - self.last_update_time >= self.update_interval
    }

    pub fn mark_updated(&mut self, current_time: f32) {
        self.last_update_time = current_time;
    }
}

// Configuration for spawning collectibles - keeping for potential future use
#[derive(Clone)]
#[allow(dead_code)]
pub struct CollectibleConfig {
    pub position: Vec3,
    pub collectible_type: CollectibleType,
    pub scale: f32,
    pub rotation: Option<CollectibleRotation>,
}

#[derive(Resource, Default)]
pub struct PlayerMovementTracker {
    pub last_position: Option<Vec3>,
    pub time_stationary: f32,
    pub paused: bool,
}

// ===== PLUGIN =====

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollectibleProgressTracker>()
            .insert_resource(crate::ui::inventory::InventoryVisibilityState::default())
            .init_resource::<CollectibleSpawner>()
            .init_resource::<PlayerMovementTracker>()
            .init_resource::<NavigationBasedSpawner>()

            // CoinStreamingManager now initialized in pregame_loading to persist between screens
            .add_systems(
                Update,
                (
                    update_coin_streaming,            // Stream coins every 2-3 seconds
                    handle_coin_collisions,           // Handle collision-based coin collection
                    update_floating_items,
                    rotate_collectibles,

                    crate::ui::inventory::add_item_to_inventory,
                    crate::ui::inventory::toggle_inventory_visibility,
                    crate::ui::inventory::adjust_inventory_for_dialogs,
                    track_player_movement,
                )
                    .run_if(in_state(Screen::GamePlay)),
            );
    }
}

// ===== SYSTEMS =====

/// Streaming system that spawns/despawns coins based on player proximity every 2-3 seconds
fn update_coin_streaming(
    mut commands: Commands,
    mut streaming_manager: ResMut<CoinStreamingManager>,
    player_query: Query<&Transform, With<CharacterController>>,
    model_assets: Option<Res<ModelAssets>>,
    time: Res<Time>,
    existing_coins: Query<(Entity, &StreamingCoin)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let Some(assets) = model_assets else {
        return;
    };

    let current_time = time.elapsed_secs();
    
    // Only update every 2-3 seconds
    if !streaming_manager.should_update(current_time) {
        return;
    }


    streaming_manager.mark_updated(current_time);

    let player_pos = player_transform.translation;

    // First, despawn coins that are too far away
    let mut to_despawn = Vec::new();
    for (entity, streaming_coin) in existing_coins.iter() {
        let coin_pos = streaming_manager.positions[streaming_coin.position_id];
        let distance = player_pos.distance(coin_pos);
        
        if distance > streaming_manager.spawn_radius * 1.2 { // Add hysteresis
            to_despawn.push((entity, streaming_coin.position_id));
        }
    }

    // Despawn distant coins
    for (entity, position_id) in to_despawn {
        commands.entity(entity).despawn();
        streaming_manager.spawned_coins.remove(&position_id);
    }

    // Collect positions that need to be spawned
    let mut positions_to_spawn = Vec::new();
    let mut nearest_coin_distance = f32::INFINITY;
    let mut _total_in_range = 0;
    
    for (position_id, &position) in streaming_manager.positions.iter().enumerate() {
        let distance = player_pos.distance(position);
        

        if distance < nearest_coin_distance {
            nearest_coin_distance = distance;
        }
        
        if distance <= streaming_manager.spawn_radius {
            _total_in_range += 1;
            if !streaming_manager.spawned_coins.contains_key(&position_id) 
                && !streaming_manager.collected_positions.contains(&position_id) {
                positions_to_spawn.push((position_id, position));
            }
        }
    }



    // Spawn the collected positions
    for (position_id, position) in positions_to_spawn {
        
        let entity = spawn_streaming_coin(&mut commands, &assets, position, position_id);
        streaming_manager.spawned_coins.insert(position_id, entity);
    }

    let _active_coins = streaming_manager.spawned_coins.len();

}

/// Spawn a single streaming coin
fn spawn_streaming_coin(
    commands: &mut Commands,
    assets: &ModelAssets,
    position: Vec3,
    position_id: usize,
) -> Entity {
    // Adjust Y position based on current value
    let adjusted_position = Vec3::new(
        position.x,
        if position.y == -1.5 {
            position.y // No change for -1.5
        } else if position.y >= 10.0 {
            position.y + 2.5 // Add 2.5 if at least 10
        } else if position.y >= 5.0 {
            position.y + 2.0 // Add 2.0 if at least 5
        } else {
            position.y // No change for other values
        },
        position.z,
    );
    
    // Create a compound collider that better approximates a coin shape
    // This is more performant than mesh-fitted colliders while still being more accurate than a single sphere
    let coin_collider = Collider::compound(vec![
        // Main body - slightly flattened sphere
        (Vec3::ZERO, Quat::IDENTITY, Collider::sphere(0.4)),
        // Edge rings for better coin-like collision
        (Vec3::new(0.0, 0.0, 0.0), Quat::IDENTITY, Collider::cylinder(0.4, 0.1)),
    ]);
    
    commands.spawn((
        Name::new("Streaming Coin"),
        SceneRoot(assets.coin.clone()),
        Transform {
            translation: adjusted_position,
            scale: Vec3::splat(0.75),
            ..default()
        },
        coin_collider,
        RigidBody::Kinematic,
        Visibility::Visible,
        Collectible,
        CollectibleType::Coin,
        FloatingItem {
            base_height: adjusted_position.y, // Use adjusted position for floating base height
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        CollectibleRotation {
            enabled: true,
            clockwise: true,
            speed: 1.0,
        },
        Sensor, // This makes the coin non-solid but still detects collisions
        CollisionEventsEnabled, // Enable collision events for this coin
        StreamingCoin { position_id },
    )).id()
}

/// System that handles coin collection through collision events
fn handle_coin_collisions(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    player_query: Query<Entity, With<CharacterController>>,
    coin_query: Query<(Entity, &CollectibleType, Option<&StreamingCoin>), (With<Collectible>, Without<Collected>)>,
    mut progress_tracker: ResMut<CollectibleProgressTracker>,
    mut streaming_manager: ResMut<CoinStreamingManager>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // Get the player entity
    let Ok(player_entity) = player_query.single() else {
        return;
    };

    // Process collision events
    for CollisionStarted(collider1, collider2) in collision_events.read() {
        // Determine which entity is the player and which is the collectible
        let collectible_entity = if *collider1 == player_entity {
            *collider2
        } else if *collider2 == player_entity {
            *collider1
        } else {
            continue; // Neither entity is the player
        };

        // Check if the collectible is a coin
        if let Ok((entity, collectible_type, streaming_coin)) = coin_query.get(collectible_entity) {
            if *collectible_type == CollectibleType::Coin {
                // Remove from streaming manager if it's a streaming coin
                if let Some(streaming) = streaming_coin {
                    streaming_manager.spawned_coins.remove(&streaming.position_id);
                    streaming_manager.collected_positions.insert(streaming.position_id);
                }

                // Play coin collection sound effect
                sfx_events.write(PlaySfxEvent {
                    sfx_type: SfxType::CoinCollect,
                });

                // Mark as collected
                commands.entity(entity).insert(Collected);
                // Insert NextItemToAdd so inventory system will add it
                commands.insert_resource(NextItemToAdd(*collectible_type));
                // Despawn the entity immediately
                commands.entity(entity).despawn();
                // Update progress tracker
                match collectible_type {
                    CollectibleType::Coin => {
                        progress_tracker.coins_collected += 1;
                    },
                    CollectibleType::Book => {
                        progress_tracker.books_collected += 1;
                    },
                    CollectibleType::HealthPotion => {
                        progress_tracker.health_potions_collected += 1;
                    },
                    CollectibleType::SurvivalKit => {
                        progress_tracker.survival_kits_collected += 1;
                    },
                }
                



            }
        }
    }
}

fn update_floating_items(
    time: Res<Time>, 
    mut query: Query<(&FloatingItem, &mut Transform), With<Collectible>>
) {
    for (floating, mut transform) in query.iter_mut() {
        let time = time.elapsed_secs();
        let hover_offset = (time * floating.hover_speed).sin() * floating.hover_amplitude;
        transform.translation.y = floating.base_height + hover_offset;
    }
}

pub fn rotate_collectibles(
    mut collectible_query: Query<(&mut Transform, &CollectibleRotation), With<Collectible>>,
    time: Res<Time>,
) {
    for (mut transform, rotation) in collectible_query.iter_mut() {
        if rotation.enabled {
            let rotation_amount = if rotation.clockwise {
                rotation.speed * time.delta_secs()
            } else {
                -rotation.speed * time.delta_secs()
            };
            transform.rotate_y(rotation_amount);
        }
    }
}

// System to track player movement and update PlayerMovementTracker
fn track_player_movement(
    time: Res<Time>,
    player_query: Query<&Transform, With<CharacterController>>,
    mut tracker: ResMut<PlayerMovementTracker>,
) {
    let Ok(player_transform) = player_query.single() else { return; };
    let pos = player_transform.translation;
    let moved = if let Some(last) = tracker.last_position {
        pos.distance(last) > 0.05 // movement threshold
    } else {
        true
    };
    if moved {
        tracker.time_stationary = 0.0;
        tracker.paused = false;
        tracker.last_position = Some(pos);
    } else {
        tracker.time_stationary += time.delta_secs();
        if tracker.time_stationary >= 4.0 {
            tracker.paused = true;
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigationData {
    pub session_start: String,
    pub positions: Vec<NavigationPoint>,
    pub statistics: NavigationStats,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigationPoint {
    pub timestamp: f64,
    pub position: [f32; 3],
    pub session_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigationStats {
    pub total_points: usize,
    pub session_duration: f32,
    pub min_bounds: [f32; 3],
    pub max_bounds: [f32; 3],
    pub average_position: [f32; 3],
}

impl Default for NavigationData {
    fn default() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            session_start: format!("{}", timestamp),
            positions: Vec::new(),
            statistics: NavigationStats {
                total_points: 0,
                session_duration: 0.0,
                min_bounds: [f32::INFINITY; 3],
                max_bounds: [f32::NEG_INFINITY; 3],
                average_position: [0.0; 3],
            },
        }
    }
}



// Replace the surface-based spawning with navigation-based spawning
#[derive(Resource)]
pub struct NavigationBasedSpawner {
    pub nav_positions: Vec<Vec3>,
    pub spawn_probability: f32,
    pub loaded: bool,
}

impl Default for NavigationBasedSpawner {
    fn default() -> Self {
        Self {
            nav_positions: Vec::new(),
            spawn_probability: 0.15,     // 15% chance per nav position
            loaded: false,
        }
    }
}
