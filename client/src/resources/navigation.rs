use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
        
        // Always start with current position
        path.push(start);
        
        // Find nearest nav points to start and target
        let start_point = self.find_nearest_point(start);
        let target_point = self.find_nearest_point(target);
        
        // Get path points
        if let (Some(start), Some(target)) = (start_point, target_point) {
            let start_idx = self.nav_points.iter().position(|p| p.position == start.position).unwrap_or(0);
            let target_idx = self.nav_points.iter().position(|p| p.position == target.position).unwrap_or(0);
            
            // Get all points between start and target
            let points: Vec<_> = if start_idx <= target_idx {
                self.nav_points[start_idx..=target_idx].iter().map(|p| p.position).collect()
            } else {
                // Reverse path if target is before start
                self.nav_points[target_idx..=start_idx].iter().rev().map(|p| p.position).collect()
            };
            
            // Sample points to create a smoother path
            if points.len() > max_points {
                let step = points.len() / max_points;
                path.extend(points.iter().step_by(step).cloned());
            } else {
                path.extend(points);
            }
        } else {
            // If no nav points found, create a direct path with interpolated points
            let distance = start.distance(target);
            let point_count = (distance / 5.0).ceil() as usize; // One point every 5 units
            
            for i in 1..point_count {
                let t = i as f32 / point_count as f32;
                let point = start.lerp(target, t);
                path.push(point);
            }
        }
        
        // Always include target position if not already included
        if path.last() != Some(&target) {
            path.push(target);
        }
        
        path
    }
}
