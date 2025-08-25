use bevy::prelude::*;
use serde_json::Value;
use crate::resources::navigation::{NavigationData, NavPoint, NavStatistics};

pub fn load_navigation_data(mut commands: Commands) {
    // Load nav.json file
    let nav_file = include_str!("../../nav.json");
    let nav_json: Value = serde_json::from_str(nav_file).expect("Failed to parse nav.json");
    
    // Parse positions
    let positions = nav_json["positions"].as_array().unwrap();
    let nav_points: Vec<NavPoint> = positions.iter().map(|pos| {
        NavPoint {
            timestamp: pos["timestamp"].as_f64().unwrap() as f32,
            position: Vec3::new(
                pos["position"][0].as_f64().unwrap() as f32,
                pos["position"][1].as_f64().unwrap() as f32,
                pos["position"][2].as_f64().unwrap() as f32,
            ),
            session_time: pos["session_time"].as_f64().unwrap() as f32,
        }
    }).collect();
    
    // Parse statistics
    let stats = &nav_json["statistics"];
    let statistics = NavStatistics {
        total_points: stats["total_points"].as_u64().unwrap() as usize,
        session_duration: stats["session_duration"].as_f64().unwrap() as f32,
        min_bounds: Vec3::new(
            stats["min_bounds"][0].as_f64().unwrap() as f32,
            stats["min_bounds"][1].as_f64().unwrap() as f32,
            stats["min_bounds"][2].as_f64().unwrap() as f32,
        ),
        max_bounds: Vec3::new(
            stats["max_bounds"][0].as_f64().unwrap() as f32,
            stats["max_bounds"][1].as_f64().unwrap() as f32,
            stats["max_bounds"][2].as_f64().unwrap() as f32,
        ),
        average_position: Vec3::new(
            stats["average_position"][0].as_f64().unwrap() as f32,
            stats["average_position"][1].as_f64().unwrap() as f32,
            stats["average_position"][2].as_f64().unwrap() as f32,
        ),
    };
    
    // Create navigation data resource
    let nav_data = NavigationData {
        nav_points,
        statistics,
    };
    
    commands.insert_resource(nav_data);
}
