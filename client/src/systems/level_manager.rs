use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::screens::Screen;
use crate::systems::collectibles::{CollectibleType, CoinStreamingManager};
use crate::systems::objectives::{Objective, ObjectiveManager};
use crate::systems::character_controller::CharacterController;

// ===== LEVEL DATA STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub level_id: u32,
    pub level_name: String,
    pub player_type: String,
    pub coins: CoinSpawnData,
    pub beasts: Vec<BeastData>,
    pub objectives: Vec<LevelObjectiveData>,
    pub environment: EnvironmentData,
    pub next_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinSpawnData {
    pub spawn_count: usize,
    pub spawn_positions: Vec<Position3D>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastData {
    pub id: String,
    pub beast_type: String,
    pub spawn_position: Position3D,
    pub health: u32,
    pub damage: u32,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelObjectiveData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective_type: String,
    pub target: String,
    pub required_count: Option<u32>,
    pub position: Option<Position3D>,
    pub completion_radius: Option<f32>,
    pub reward: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentData {
    pub dungeon_scale: f32,
    pub dungeon_position: Position3D,
    pub dungeon_rotation: f32,
}

// ===== LEVEL MANAGER RESOURCE =====

#[derive(Resource)]
pub struct LevelManager {
    pub current_level: Option<u32>,
    pub levels: HashMap<u32, LevelData>,
    pub player_position: Option<Vec3>,
    pub level_completed: bool,
}

impl Default for LevelManager {
    fn default() -> Self {
        Self {
            current_level: None,
            levels: HashMap::new(),
            player_position: None,
            level_completed: false,
        }
    }
}

impl LevelManager {
    pub fn load_level(&mut self, level_id: u32) -> Option<&LevelData> {
        if let Some(level_data) = self.levels.get(&level_id) {
            self.current_level = Some(level_id);
            self.level_completed = false;
            Some(level_data)
        } else {
            None
        }
    }

    pub fn get_current_level(&self) -> Option<&LevelData> {
        self.current_level.and_then(|id| self.levels.get(&id))
    }

    pub fn is_level_completed(&self) -> bool {
        self.level_completed
    }

    pub fn mark_level_completed(&mut self) {
        self.level_completed = true;
    }

    pub fn get_next_level_id(&self) -> Option<u32> {
        self.get_current_level()?.next_level
    }

    pub fn advance_to_next_level(&mut self) -> bool {
        if let Some(next_level_id) = self.get_next_level_id() {
            if self.levels.contains_key(&next_level_id) {
                self.load_level(next_level_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

// ===== LEVEL LOADING SYSTEM =====

pub fn load_level_data() -> HashMap<u32, LevelData> {
    let mut levels = HashMap::new();
    
    // Load level 1
    if let Ok(level_1_data) = serde_json::from_str::<LevelData>(include_str!("../levels/level_1.json")) {
        levels.insert(1, level_1_data);
    }
    
    // Load level 2
    if let Ok(level_2_data) = serde_json::from_str::<LevelData>(include_str!("../levels/level_2.json")) {
        levels.insert(2, level_2_data);
    }
    
    levels
}

// ===== SYSTEMS =====

/// System to initialize level manager with loaded level data
fn initialize_level_manager(
    mut level_manager: ResMut<LevelManager>,
) {
    if level_manager.levels.is_empty() {
        level_manager.levels = load_level_data();
        // Start with level 1
        level_manager.load_level(1);
    }
}

/// System to track player position for level management
fn track_player_position_for_levels(
    player_query: Query<&Transform, With<CharacterController>>,
    mut level_manager: ResMut<LevelManager>,
) {
    if let Ok(player_transform) = player_query.single() {
        level_manager.player_position = Some(player_transform.translation);
    }
}

/// System to load level objectives into the objective manager
fn load_level_objectives(
    level_manager: Res<LevelManager>,
    mut objective_manager: ResMut<ObjectiveManager>,
    mut coin_streaming_manager: ResMut<CoinStreamingManager>,
) {
    if let Some(level_data) = level_manager.get_current_level() {
        // Clear existing objectives
        objective_manager.objectives.clear();
        objective_manager.next_id = 0;
        
        // Clear existing coin positions
        coin_streaming_manager.positions.clear();
        coin_streaming_manager.spawned_coins.clear();
        coin_streaming_manager.collected_positions.clear();
        
        // Load coin positions into streaming manager
        for position in &level_data.coins.spawn_positions {
            coin_streaming_manager.add_position(Vec3::new(
                position.x,
                position.y,
                position.z,
            ));
        }
        
        // Convert level objectives to game objectives
        for level_obj in level_data.objectives.iter() {
            let objective_id = objective_manager.next_id;
            
            // Determine collectible type based on objective target
            let collectible_type = match level_obj.target.as_str() {
                "coins" => CollectibleType::Coin,
                "ancient_book" => CollectibleType::Book,
                "power_crystal" => CollectibleType::Book, // Using Book as placeholder
                _ => CollectibleType::Coin, // Default to coin
            };
            
            // Determine required count
            let required_count = level_obj.required_count.unwrap_or(1);
            
            let objective = Objective::new(
                objective_id,
                level_obj.title.clone(),
                level_obj.description.clone(),
                collectible_type,
                required_count,
            );
            
            objective_manager.add_objective(objective);
        }
    }
}

/// System to check level completion based on objectives
fn check_level_completion(
    level_manager: Res<LevelManager>,
    objective_manager: Res<ObjectiveManager>,
    _next_state: ResMut<NextState<Screen>>,
) {
    if level_manager.is_level_completed() {
        return;
    }
    
    // Check if all objectives are completed
    let all_completed = objective_manager.objectives.iter().all(|obj| obj.completed);
    
    if all_completed {
        // Mark level as completed
        // Note: We can't mutate level_manager here due to borrow checker
        // This will be handled in the next frame
        
        // Check if there's a next level
        if level_manager.get_next_level_id().is_some() {
            // Could transition to next level or show completion screen
            // For now, just mark as completed
        }
    }
}

/// System to mark level as completed when all objectives are done
fn mark_level_completed(
    mut level_manager: ResMut<LevelManager>,
    objective_manager: Res<ObjectiveManager>,
) {
    if !level_manager.is_level_completed() {
        let all_completed = objective_manager.objectives.iter().all(|obj| obj.completed);
        if all_completed {
            level_manager.mark_level_completed();
            info!("Level {} completed!", level_manager.current_level.unwrap_or(0));
        }
    }
}

/// System to handle level transitions
fn handle_level_transitions(
    mut level_manager: ResMut<LevelManager>,
    mut objective_manager: ResMut<ObjectiveManager>,
    mut coin_streaming_manager: ResMut<CoinStreamingManager>,
) {
    if level_manager.is_level_completed() {
        if let Some(next_level_id) = level_manager.get_next_level_id() {
            // Advance to next level
            if level_manager.advance_to_next_level() {
                info!("Advancing to level {}", next_level_id);
                
                // Reload objectives for the new level
                if let Some(level_data) = level_manager.get_current_level() {
                    // Clear existing objectives
                    objective_manager.objectives.clear();
                    objective_manager.next_id = 0;
                    
                    // Clear existing coin positions
                    coin_streaming_manager.positions.clear();
                    coin_streaming_manager.spawned_coins.clear();
                    coin_streaming_manager.collected_positions.clear();
                    
                    // Load coin positions into streaming manager
                    for position in &level_data.coins.spawn_positions {
                        coin_streaming_manager.add_position(Vec3::new(
                            position.x,
                            position.y,
                            position.z,
                        ));
                    }
                    
                    // Convert level objectives to game objectives
                    for level_obj in &level_data.objectives {
                        let objective_id = objective_manager.next_id;
                        
                        // Determine collectible type based on objective target
                        let collectible_type = match level_obj.target.as_str() {
                            "coins" => CollectibleType::Coin,
                            "ancient_book" => CollectibleType::Book,
                            "power_crystal" => CollectibleType::Book, // Using Book as placeholder
                            _ => CollectibleType::Coin, // Default to coin
                        };
                        
                        // Determine required count
                        let required_count = level_obj.required_count.unwrap_or(1);
                        
                        let objective = Objective::new(
                            objective_id,
                            level_obj.title.clone(),
                            level_obj.description.clone(),
                            collectible_type,
                            required_count,
                        );
                        
                        objective_manager.add_objective(objective);
                    }
                }
            }
        }
    }
}

/// System to update objective progress based on collectible collection
fn update_objective_progress(
    mut objective_manager: ResMut<ObjectiveManager>,
    mut collectible_events: EventReader<crate::systems::dojo::pickup_item::PickupItemEvent>,
) {
    for event in collectible_events.read() {
        // Find objectives that match the collected item type
        for objective in &mut objective_manager.objectives {
            if objective.item_type == event.item_type && !objective.completed {
                objective.current_count += 1;
                
                // Check if objective is completed
                if objective.current_count >= objective.required_count {
                    objective.completed = true;
                }
            }
        }
    }
}

/// System to check location-based objectives
fn check_location_objectives(
    level_manager: Res<LevelManager>,
    mut objective_manager: ResMut<ObjectiveManager>,
    player_query: Query<&Transform, With<CharacterController>>,
) {
    if let (Some(level_data), Ok(player_transform)) = (level_manager.get_current_level(), player_query.single()) {
        let player_pos = player_transform.translation;
        
        for level_obj in &level_data.objectives {
            if level_obj.objective_type == "reach_location" {
                if let Some(target_position) = &level_obj.position {
                    let target_vec3 = Vec3::new(target_position.x, target_position.y, target_position.z);
                    let distance = player_pos.distance(target_vec3);
                    let completion_radius = level_obj.completion_radius.unwrap_or(5.0);
                    
                    if distance <= completion_radius {
                        // Find and complete the corresponding objective
                        for objective in &mut objective_manager.objectives {
                            if objective.title == level_obj.title && !objective.completed {
                                objective.completed = true;
                                info!("Location objective completed: {}", objective.title);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// System to handle defeat objectives (monster defeats)
/// This would need to be connected to the combat system
fn check_defeat_objectives(
    level_manager: Res<LevelManager>,
    _objective_manager: ResMut<ObjectiveManager>,
) {
    if let Some(level_data) = level_manager.get_current_level() {
        for level_obj in &level_data.objectives {
            if level_obj.objective_type == "defeat" {
                // This would need to be connected to the combat system
                // For now, we'll just log that we found a defeat objective
                debug!("Found defeat objective: {} - {}", level_obj.title, level_obj.target);
            }
        }
    }
}

/// Debug system to show level information
fn debug_level_info(
    level_manager: Res<LevelManager>,
    objective_manager: Res<ObjectiveManager>,
) {
    if level_manager.is_changed() || objective_manager.is_changed() {
        if let Some(level_data) = level_manager.get_current_level() {
            info!("=== Level {}: {} ===", level_data.level_id, level_data.level_name);
            info!("Player Type: {}", level_data.player_type);
            info!("Coins to spawn: {}", level_data.coins.spawn_count);
            info!("Beasts: {}", level_data.beasts.len());
            info!("Objectives: {}", level_data.objectives.len());
            
            for (i, objective) in objective_manager.objectives.iter().enumerate() {
                info!("  Objective {}: {} ({}/{}) - {}", 
                    i + 1, 
                    objective.title, 
                    objective.current_count, 
                    objective.required_count,
                    if objective.completed { "COMPLETED" } else { "IN PROGRESS" }
                );
            }
            
            if level_manager.is_level_completed() {
                info!("Level {} COMPLETED!", level_data.level_id);
                if let Some(next_level) = level_data.next_level {
                    info!("Next level: {}", next_level);
                } else {
                    info!("This is the final level!");
                }
            }
        }
    }
}

/// System to check if player has reached the next level area
fn check_next_level_transition(
    level_manager: Res<LevelManager>,
    player_query: Query<&Transform, With<CharacterController>>,
) {
    if let (Some(level_data), Ok(player_transform)) = (level_manager.get_current_level(), player_query.single()) {
        let player_pos = player_transform.translation;
        
        // Check if player has reached the next level area (beyond the current level's boundaries)
        // For level 1, check if player has moved significantly forward
        if level_data.level_id == 1 && player_pos.x > 60.0 {
            info!("Player has reached the next level area! Current position: {:?}", player_pos);
        }
        
        // For level 2, check if player has moved even further
        if level_data.level_id == 2 && player_pos.x > 120.0 {
            info!("Player has reached the next level area! Current position: {:?}", player_pos);
        }
    }
}

// ===== PLUGIN =====

pub struct LevelManagerPlugin;

impl Plugin for LevelManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelManager>()
            .add_systems(Startup, initialize_level_manager)
            .add_systems(
                Update,
                (
                    track_player_position_for_levels,
                    check_level_completion,
                    mark_level_completed,
                    handle_level_transitions,
                    update_objective_progress,
                    check_location_objectives,
                    check_defeat_objectives,
                    debug_level_info,
                    check_next_level_transition,
                ).run_if(in_state(Screen::GamePlay)),
            )
            .add_systems(
                OnEnter(Screen::GamePlay),
                load_level_objectives,
            );
    }
}
