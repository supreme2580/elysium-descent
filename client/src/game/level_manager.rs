use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== LEVEL DATA STRUCTURES =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelData {
    pub level_id: u32,
    pub level_name: String,
    pub player_type: String,
    pub coins: CoinData,
    pub beasts: Vec<BeastData>,
    pub objectives: Vec<ObjectiveData>,
    pub environment: EnvironmentData,
    pub next_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinData {
    pub spawn_count: u32,
    pub spawn_positions: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeastData {
    pub id: String,
    pub beast_type: String,
    pub spawn_position: Position,
    pub health: u32,
    pub damage: u32,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveData {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective_type: String,
    pub target: String,
    pub required_count: Option<u32>,
    pub position: Option<Position>,
    pub completion_radius: Option<f32>,
    pub reward: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentData {
    pub dungeon_scale: f32,
    pub dungeon_position: Position,
    pub dungeon_rotation: f32,
}

// ===== LEVEL MANAGER RESOURCE =====

#[derive(Resource)]
pub struct LevelManager {
    pub current_level: u32,
    pub levels: HashMap<u32, LevelData>,
    pub default_level: LevelData,
    pub level_completed: bool,
}

impl Default for LevelManager {
    fn default() -> Self {
        Self {
            current_level: 1,
            levels: HashMap::new(),
            default_level: Self::create_default_level(),
            level_completed: false,
        }
    }
}

impl LevelManager {
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.load_levels();
        manager
    }

    pub fn load_levels(&mut self) {
        // Load level 1
        if let Ok(level_1_data) = std::fs::read_to_string("client/src/levels/level_1.json") {
            if let Ok(level_data) = serde_json::from_str::<LevelData>(&level_1_data) {
                self.levels.insert(1, level_data);
            }
        }

        // Load level 2
        if let Ok(level_2_data) = std::fs::read_to_string("client/src/levels/level_2.json") {
            if let Ok(level_data) = serde_json::from_str::<LevelData>(&level_2_data) {
                self.levels.insert(2, level_data);
            }
        }
    }

    pub fn get_current_level(&self) -> &LevelData {
        self.levels.get(&self.current_level).unwrap_or(&self.default_level)
    }

    pub fn advance_level(&mut self) -> bool {
        if let Some(current_level_data) = self.levels.get(&self.current_level) {
            if let Some(next_level_id) = current_level_data.next_level {
                if self.levels.contains_key(&next_level_id) {
                    self.current_level = next_level_id;
                    self.level_completed = false;
                    return true;
                }
            }
        }
        
        // No more levels, go to default
        self.current_level = 0;
        self.level_completed = false;
        true
    }

    pub fn mark_level_completed(&mut self) {
        self.level_completed = true;
    }

    pub fn reset_to_level(&mut self, level_id: u32) {
        if self.levels.contains_key(&level_id) {
            self.current_level = level_id;
            self.level_completed = false;
        }
    }

    fn create_default_level() -> LevelData {
        LevelData {
            level_id: 0,
            level_name: "Endless Adventure".to_string(),
            player_type: "Man".to_string(),
            coins: CoinData {
                spawn_count: 25,
                spawn_positions: vec![
                    Position { x: 0.0, y: 1.0, z: 0.0 },
                    Position { x: 10.0, y: 1.0, z: 10.0 },
                    Position { x: -10.0, y: 1.0, z: -10.0 },
                    Position { x: 20.0, y: 1.0, z: 20.0 },
                    Position { x: -20.0, y: 1.0, z: -20.0 },
                ],
            },
            beasts: vec![
                BeastData {
                    id: "endless_monster".to_string(),
                    beast_type: "Monster".to_string(),
                    spawn_position: Position { x: 0.0, y: 1.0, z: 0.0 },
                    health: 200,
                    damage: 50,
                    speed: 5.0,
                },
            ],
            objectives: vec![
                ObjectiveData {
                    id: "endless_collect".to_string(),
                    title: "Endless Collection".to_string(),
                    description: "Collect coins endlessly in this infinite adventure".to_string(),
                    objective_type: "collect".to_string(),
                    target: "coins".to_string(),
                    required_count: Some(10),
                    position: None,
                    completion_radius: None,
                    reward: "endless_glory".to_string(),
                },
            ],
            environment: EnvironmentData {
                dungeon_scale: 10.0,
                dungeon_position: Position { x: 0.0, y: -1.5, z: 0.0 },
                dungeon_rotation: -1.5708,
            },
            next_level: None,
        }
    }
}

// ===== LEVEL EVENTS =====

#[derive(Event)]
pub struct LevelCompletedEvent {
    pub level_id: u32,
}

#[derive(Event)]
pub struct LevelStartedEvent {
    pub level_id: u32,
    pub level_data: LevelData,
}

// ===== PLUGIN =====

pub struct LevelManagerPlugin;

impl Plugin for LevelManagerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelManager>()
            .add_event::<LevelCompletedEvent>()
            .add_event::<LevelStartedEvent>()
            .add_systems(Startup, setup_level_manager)
            .add_systems(Update, handle_level_progression);
    }
}

// ===== SYSTEMS =====

fn setup_level_manager(mut commands: Commands) {
    let level_manager = LevelManager::new();
    commands.insert_resource(level_manager);
}

fn handle_level_progression(
    mut level_manager: ResMut<LevelManager>,
    mut level_completed_events: EventReader<LevelCompletedEvent>,
    mut level_started_events: EventWriter<LevelStartedEvent>,
) {
    for _event in level_completed_events.read() {
        level_manager.mark_level_completed();
        
        // Advance to next level
        if level_manager.advance_level() {
            let current_level_data = level_manager.get_current_level().clone();
            level_started_events.write(LevelStartedEvent {
                level_id: level_manager.current_level,
                level_data: current_level_data,
            });
        }
    }
}
