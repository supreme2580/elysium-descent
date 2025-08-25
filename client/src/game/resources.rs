use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/******************************************************************************
 *                               GAME RESOURCES                               *
 ******************************************************************************/

// Game timer resource to track countdown and game time
#[derive(Resource)]
pub struct GameTimer {
    pub time_left_seconds: f32,
    pub max_time_seconds: f32,
    pub is_running: bool,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            time_left_seconds: 300.0, // 5 minutes
            max_time_seconds: 300.0,   // 5 minutes
            is_running: true,
        }
    }
}

impl GameTimer {
    pub fn new(duration_seconds: f32) -> Self {
        Self {
            time_left_seconds: duration_seconds,
            max_time_seconds: duration_seconds,
            is_running: true,
        }
    }
    
    pub fn get_time_remaining_formatted(&self) -> String {
        let minutes = (self.time_left_seconds as u32) / 60;
        let seconds = (self.time_left_seconds as u32) % 60;
        format!("{}:{:02}", minutes, seconds)
    }
    
    pub fn get_time_percentage(&self) -> f32 {
        if self.max_time_seconds > 0.0 {
            self.time_left_seconds / self.max_time_seconds
        } else {
            0.0
        }
    }
}

// Player level and experience resource
#[derive(Resource)]
pub struct PlayerProgression {
    pub current_level: u32,
    pub max_level: u32,
}

impl Default for PlayerProgression {
    fn default() -> Self {
        Self {
            current_level: 1,
            max_level: 50,
        }
    }
}

impl PlayerProgression {
    pub fn get_level_display(&self) -> String {
        format!("{}/{}", self.current_level, self.max_level)
    }
    
    pub fn get_xp_for_display(&self) -> (u32, u32) {
        // For level progression, show current level progress out of max levels
        (self.current_level, self.max_level)
    }
}

/******************************************************************************
 *                            NAVIGATION RESOURCES                            *
 ******************************************************************************/

#[derive(Resource, Default)]
pub struct NavigationData {
    pub nav_points: Vec<NavPoint>,
    pub statistics: NavStatistics,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NavPoint {
    pub timestamp: f32,
    pub position: Vec3,
    pub session_time: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NavStatistics {
    pub total_points: usize,
    pub session_duration: f32,
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub average_position: Vec3,
}

impl NavigationData {
    pub fn find_nearest_point(&self, position: Vec3) -> Option<&NavPoint> {
        self.nav_points.iter().min_by(|a, b| {
            let dist_a = (a.position - position).length_squared();
            let dist_b = (b.position - position).length_squared();
            dist_a.partial_cmp(&dist_b).unwrap()
        })
    }

    pub fn find_path_to_target(&self, start: Vec3, target: Vec3, max_points: usize) -> Vec<Vec3> {
        let mut path = Vec::new();
        
        // Find nearest nav points to start and target
        let start_point = self.find_nearest_point(start);
        let target_point = self.find_nearest_point(target);
        
        if let (Some(start), Some(target)) = (start_point, target_point) {
            // Find nav points between start and target
            let start_idx = self.nav_points.iter().position(|p| p.position == start.position).unwrap_or(0);
            let target_idx = self.nav_points.iter().position(|p| p.position == target.position).unwrap_or(0);
            
            // Determine direction of path
            let (from_idx, to_idx) = if start_idx <= target_idx {
                (start_idx, target_idx)
            } else {
                (target_idx, start_idx)
            };
            
            // Get subset of points
            let points_between: Vec<_> = self.nav_points[from_idx..=to_idx]
                .iter()
                .map(|p| p.position)
                .collect();
            
            // Sample points to match max_points
            let step = (points_between.len() as f32 / max_points as f32).max(1.0) as usize;
            path = points_between.iter()
                .step_by(step)
                .cloned()
                .collect();
        }
        
        // Always include target position
        path.push(target);
        path
    }
}