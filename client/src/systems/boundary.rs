//! Boundary System for Elysium Descent
//! 
//! This system provides invisible boundaries around the playable area to prevent
//! the player, enemies, and collectibles from going outside the intended game world.
//! 
//! ## Features
//! - Invisible collision walls around the playable area
//! - Player movement constraints to prevent walking off the environment
//! - Enemy AI boundary awareness
//! - Collectible spawning boundary constraints
//! 
//! ## Boundary Dimensions
//! Based on navigation data analysis, the playable area is defined as:
//! - X: -40.0 to 285.0 (325 units wide)
//! - Z: -130.0 to 200.0 (330 units deep)
//! - Y: No vertical constraints (player can jump/fall naturally)
//! 
//! ## Usage
//! The BoundaryPlugin is automatically added to gameplay and fight scenes.
//! The boundaries are completely invisible but provide solid collision detection.
//! 
//! ## Components
//! - `BoundaryWall`: Marker for boundary wall entities
//! - `BoundaryConstraint`: Resource defining boundary limits

use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::math::primitives::Cuboid;
// Mesh3d and MeshMaterial3d are re-exported in prelude in Bevy 0.16
use crate::constants::boundary::BoundaryConstants;

/// Marker component for boundary walls
#[derive(Component)]
pub struct BoundaryWall;

/// Component to define boundary constraints
#[derive(Component, Resource, Clone)]
pub struct BoundaryConstraint {
    pub min_x: f32,
    pub max_x: f32,
    pub min_z: f32,
    pub max_z: f32,
}

impl Default for BoundaryConstraint {
    fn default() -> Self {
        // Create a perfect square boundary centered around the actual world center
        // Apply offsets to minimize visual gaps; keep the right side (east) exact
        let half_length = BoundaryConstants::BOUNDARY_LENGTH / 2.0;
        let inset = BoundaryConstants::WORLD_INSET;
        
        Self {
            min_x: BoundaryConstants::WORLD_CENTER_X - half_length,  // West boundary (offset inwards)
            max_x: BoundaryConstants::WORLD_CENTER_X + half_length - (inset * 2.0),          // East boundary (kept exact)
            min_z: BoundaryConstants::WORLD_CENTER_Z - half_length + inset,  // South boundary (offset inwards)
            max_z: BoundaryConstants::WORLD_CENTER_Z + half_length - inset,  // North boundary (offset inwards)
        }
    }
}

/// Plugin for boundary systems
pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::screens::Screen::GamePlay),
            spawn_boundary_walls,
        )
        .add_systems(
            Update,
            (
                constrain_player_movement,
                log_player_position,
            ),
        );
    }
}

/// Logs the player's current position every 5 seconds for debugging boundary lengths
fn log_player_position(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::systems::character_controller::CharacterController>>,
    boundary_constraint: Option<Res<BoundaryConstraint>>,
    mut last_log_time: Local<f32>,
) {
    // Log every 5 seconds
    if time.elapsed_secs() - *last_log_time < 5.0 {
        return;
    }
    
    let Some(boundary_constraint) = boundary_constraint else {
        return; // No boundary constraints defined, skip this system
    };

    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;
        
        info!("=== PLAYER POSITION LOG ===");
        info!("Player X: {:.2} (bounds: {:.2} to {:.2})", player_pos.x, boundary_constraint.min_x, boundary_constraint.max_x);
        info!("Player Z: {:.2} (bounds: {:.2} to {:.2})", player_pos.z, boundary_constraint.min_z, boundary_constraint.max_z);
        info!("Player Y: {:.2}", player_pos.y);
        info!("Distance to boundaries:");
        info!("  North: {:.2} units", boundary_constraint.max_z - player_pos.z);
        info!("  South: {:.2} units", player_pos.z - boundary_constraint.min_z);
        info!("  East: {:.2} units", boundary_constraint.max_x - player_pos.x);
        info!("  West: {:.2} units", player_pos.x - boundary_constraint.min_x);
        info!("==========================");
        
        *last_log_time = time.elapsed_secs();
    }
}

/// Spawns invisible boundary walls around the playable area
/// 
/// Creates four static collision walls at the edges of the playable area:
/// - North wall (positive Z)
/// - South wall (negative Z) 
/// - East wall (positive X)
/// - West wall (negative X)
/// 
/// Each wall has a collision box for physics but no visual representation.
fn spawn_boundary_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let constraint = BoundaryConstraint::default();
    
    // Store boundary values before moving the constraint
    let min_x = constraint.min_x;
    let max_x = constraint.max_x;
    let min_z = constraint.min_z;
    let max_z = constraint.max_z;
    
    // Calculate wall dimensions and positions more accurately
    let world_width = max_x - min_x;
    let world_depth = max_z - min_z;
    let wall_thickness = 2.0;
    let wall_height = 24.0; // tall enough to cover above player
    let ground_y = -1.5; // environment ground offset
    let y_center = ground_y + (wall_height / 2.0);
    
    // Materials per side (semi-transparent)
    let mat_north = materials.add(StandardMaterial { base_color: Color::srgba(1.0, 0.0, 0.0, 0.35), alpha_mode: AlphaMode::Blend, ..default() });
    let mat_south = materials.add(StandardMaterial { base_color: Color::srgba(1.0, 1.0, 0.0, 0.35), alpha_mode: AlphaMode::Blend, ..default() });
    let mat_east  = materials.add(StandardMaterial { base_color: Color::srgba(0.0, 1.0, 0.0, 0.35), alpha_mode: AlphaMode::Blend, ..default() });
    let mat_west  = materials.add(StandardMaterial { base_color: Color::srgba(0.0, 0.5, 1.0, 0.35), alpha_mode: AlphaMode::Blend, ..default() });
    
    // North wall (positive Z) - spans the full width
    commands.spawn((
        Name::new("North Boundary Wall"),
        Transform::from_xyz(
            min_x + (world_width / 2.0), // Center of world width
            y_center, // Centered so it starts at ground level
            max_z + (wall_thickness / 2.0), // Just outside the max Z
        ),
        Collider::cuboid(
            // Span along X (world width)
            world_width / 2.0,
            wall_height / 2.0,
            wall_thickness / 2.0,
        ),
        RigidBody::Static,
        BoundaryWall,
        crate::screens::gameplay::PlayingScene,
    ))
    .with_children(|c| {
        c.spawn((
            Mesh3d(meshes.add(Cuboid::new(world_width, wall_height, wall_thickness))),
            MeshMaterial3d(mat_north.clone()),
            Transform::default(),
            GlobalTransform::default(),
        ));
    });

    // South wall (negative Z) - spans the full width
    commands.spawn((
        Name::new("South Boundary Wall"),
        Transform::from_xyz(
            min_x + (world_width / 2.0), // Center of world width
            y_center, // Centered so it starts at ground level
            min_z - (wall_thickness / 2.0), // Just outside the min Z
        ),
        Collider::cuboid(
            // Span along X (world width)
            world_width / 2.0,
            wall_height / 2.0,
            wall_thickness / 2.0,
        ),
        RigidBody::Static,
        BoundaryWall,
        crate::screens::gameplay::PlayingScene,
    ))
    .with_children(|c| {
        c.spawn((
            Mesh3d(meshes.add(Cuboid::new(world_width, wall_height, wall_thickness))),
            MeshMaterial3d(mat_south.clone()),
            Transform::default(),
            GlobalTransform::default(),
        ));
    });

    // East wall (positive X) - spans the full depth
    commands.spawn((
        Name::new("East Boundary Wall"),
        Transform::from_xyz(
            max_x + (wall_thickness / 2.0), // Just outside the max X
            y_center, // Centered so it starts at ground level
            min_z + (world_depth / 2.0), // Center of world depth
        ),
        Collider::cuboid(
            wall_thickness / 2.0, // Half the wall thickness
            wall_height / 2.0, // Half the wall height
            world_depth / 2.0, // Half the world depth
        ),
        RigidBody::Static,
        BoundaryWall,
        crate::screens::gameplay::PlayingScene,
    ))
    .with_children(|c| {
        c.spawn((
            Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, world_depth))),
            MeshMaterial3d(mat_east.clone()),
            Transform::default(),
            GlobalTransform::default(),
        ));
    });

    // West wall (negative X) - spans the full depth
    commands.spawn((
        Name::new("West Boundary Wall"),
        Transform::from_xyz(
            min_x - (wall_thickness / 2.0), // Just outside the min X
            y_center, // Centered so it starts at ground level
            min_z + (world_depth / 2.0), // Center of world depth
        ),
        Collider::cuboid(
            wall_thickness / 2.0, // Half the wall thickness
            wall_height / 2.0, // Half the wall height
            world_depth / 2.0, // Half the world depth
        ),
        RigidBody::Static,
        BoundaryWall,
        crate::screens::gameplay::PlayingScene,
    ))
    .with_children(|c| {
        c.spawn((
            Mesh3d(meshes.add(Cuboid::new(wall_thickness, wall_height, world_depth))),
            MeshMaterial3d(mat_west.clone()),
            Transform::default(),
            GlobalTransform::default(),
        ));
    });

    // Safety floor covering the whole playable area to prevent falling through gaps
    let floor_thickness = 1.0;
    commands.spawn((
        Name::new("Boundary Safety Floor"),
        Transform::from_xyz(
            min_x + (world_width / 2.0),
            ground_y - (floor_thickness / 2.0), // Top of the floor aligns with ground_y
            min_z + (world_depth / 2.0),
        ),
        Collider::cuboid(
            world_width / 2.0,
            floor_thickness / 2.0,
            world_depth / 2.0,
        ),
        RigidBody::Static,
        BoundaryWall,
        crate::screens::gameplay::PlayingScene,
    ));

    // Add the boundary constraint as a resource
    commands.insert_resource(constraint);
    
    // Log boundary information for debugging
    info!("Boundary walls spawned:");
    info!("  World dimensions: {} x {} units", world_width, world_depth);
    info!("  X range: {} to {}", min_x, max_x);
    info!("  Z range: {} to {}", min_z, max_z);
    info!("  North wall at Z = {}", max_z + (wall_thickness / 2.0));
    info!("  South wall at Z = {}", min_z - (wall_thickness / 2.0));
    info!("  East wall at X = {}", max_x + (wall_thickness / 2.0));
    info!("  West wall at X = {}", min_x - (wall_thickness / 2.0));
}

/// Constrains player movement to stay within boundaries
/// 
/// This system acts as a backup to prevent the player from somehow
/// getting outside the boundary walls. It directly constrains the
/// player's transform position to stay within the defined bounds.
fn constrain_player_movement(
    boundary_constraint: Option<Res<BoundaryConstraint>>,
    mut player_query: Query<&mut Transform, With<crate::systems::character_controller::CharacterController>>,
) {
    let Some(boundary_constraint) = boundary_constraint else {
        return; // No boundary constraints defined, skip this system
    };
    
    for mut player_transform in player_query.iter_mut() {
        let mut pos = player_transform.translation;
        let mut constrained = false;

        // Constrain X position
        if pos.x < boundary_constraint.min_x {
            pos.x = boundary_constraint.min_x;
            constrained = true;
        } else if pos.x > boundary_constraint.max_x {
            pos.x = boundary_constraint.max_x;
            constrained = true;
        }

        // Constrain Z position
        if pos.z < boundary_constraint.min_z {
            pos.z = boundary_constraint.min_z;
            constrained = true;
        } else if pos.z > boundary_constraint.max_z {
            pos.z = boundary_constraint.max_z;
            constrained = true;
        }

        // Apply constraints if needed
        if constrained {
            player_transform.translation = pos;
        }
    }
}
