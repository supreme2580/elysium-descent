use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;
use avian3d::{math::*, prelude::*};
use crate::systems::character_controller::AnimationState;
use crate::systems::boundary::BoundaryConstraint;
use crate::resources::navigation::NavigationData;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Component to track enemy AI state
#[derive(Component)]
pub struct EnemyAI {
    pub move_speed: f32,
    pub is_moving: bool,
    pub detection_range: f32,
    pub path_memory: Vec<Vec3>, // Store player positions for pathfinding
    pub max_path_memory: usize, // Maximum number of positions to remember
    pub current_target_index: usize, // Current target in path memory
    pub stuck_time: f32, // Time enemy has been stuck
    pub max_stuck_time: f32, // Maximum time to be stuck before recalculating
    pub last_position: Vec3, // Last position for stuck detection
    pub stuck_threshold: f32, // Distance threshold for stuck detection
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            move_speed: 3.0, // Increased speed for better pursuit
            is_moving: false,
            detection_range: 25.0, // Increased detection range
            path_memory: Vec::new(),
            max_path_memory: 30, // Increased path memory for smoother paths
            current_target_index: 0,
            stuck_time: 0.0,
            max_stuck_time: 1.5, // Reduced time before recalculating path
            last_position: Vec3::ZERO,
            stuck_threshold: 0.3, // Reduced threshold for more responsive stuck detection
        }
    }
}

/// Bundle for enemy entities
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub ai: EnemyAI,
    pub animation_state: AnimationState,
    pub body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
    pub ground_caster: ShapeCaster,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            ai: EnemyAI::default(),
            animation_state: AnimationState {
                forward_hold_time: 0.0,
                current_animation: 0, // Start uninitialized to prevent twitching
                fight_move_1: false,
                fight_move_2: false,
            },
            body: RigidBody::Kinematic, // Use kinematic instead of dynamic
            collider: Collider::capsule(0.5, 1.5),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            ground_caster: ShapeCaster::new(
                Collider::sphere(0.2),
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(2.0), // Ground detection
        }
    }
}

/// Plugin for enemy AI systems
pub struct EnemyAIPlugin;

impl Plugin for EnemyAIPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NavigationData>()
            .add_systems(Startup, crate::systems::nav_loader::load_navigation_data)
            .add_systems(
                Update,
                (
                    record_player_path,
                    enemy_ai_movement,
                    enemy_ai_animations,
                ).chain(),
            );
    }
}

/// System that records player positions for enemy pathfinding
fn record_player_path(
    _time: Res<Time>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    mut enemy_query: Query<&mut EnemyAI, With<Enemy>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    
    for mut enemy_ai in &mut enemy_query {
        // Only record if player is within detection range
        if player_pos.distance(enemy_ai.last_position) <= enemy_ai.detection_range {
            // Add current player position to path memory
            enemy_ai.path_memory.push(player_pos);
            
            // Keep only the last max_path_memory positions
            if enemy_ai.path_memory.len() > enemy_ai.max_path_memory {
                enemy_ai.path_memory.remove(0);
            }
        }
    }
}

/// System that handles enemy movement towards the player
fn enemy_ai_movement(
    time: Res<Time>,
    mut enemy_query: Query<(Entity, &mut Transform, &mut LinearVelocity, &mut EnemyAI, &mut AnimationState), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    player_query: Query<(Entity, &Transform), (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    boundary_constraint: Option<Res<BoundaryConstraint>>,
    spatial_query: SpatialQuery,
    nav_data: Res<NavigationData>,
) {
    let delta_time = time.delta_secs();
    let _current_time = time.elapsed_secs();
    
    // Find the player
    let Ok((player_id, player_transform)) = player_query.single() else {
        return;
    };

    // Get boundary constraint if available
    let boundary_constraint = if let Some(constraint) = boundary_constraint {
        constraint
    } else {
        return; // No boundary constraints, skip boundary checking
    };

    for (enemy_id, mut enemy_transform, mut enemy_velocity, mut enemy_ai, mut animation_state) in &mut enemy_query {
        let player_pos = player_transform.translation;
        let enemy_pos = enemy_transform.translation;
        let distance_to_player = enemy_pos.distance(player_pos);

        // Update stuck detection
        let distance_moved = enemy_pos.distance(enemy_ai.last_position);
        if distance_moved < enemy_ai.stuck_threshold {
            enemy_ai.stuck_time += delta_time;
        } else {
            enemy_ai.stuck_time = 0.0;
        }
        enemy_ai.last_position = enemy_pos;

        // Always act on player - no distance limitation
        // Check if player is too close (within 5 units) - be idle
        if distance_to_player <= 5.0 {
            // Too close - be idle and face player
            enemy_ai.is_moving = false;
            
            // Stop movement
            enemy_velocity.x = 0.0;
            enemy_velocity.z = 0.0;
            enemy_velocity.y = 0.0;
            animation_state.forward_hold_time = 0.0;
            
            // Face the player
            let direction_to_player = (player_pos - enemy_pos).normalize();
            let direction_2d = Vec2::new(direction_to_player.x, direction_to_player.z).normalize();
            let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(direction_2d.x, 0.0, direction_2d.y));
            enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, 5.0 * delta_time);
            
            // Match player elevation
            let target_y = player_pos.y;
            enemy_transform.translation.y = enemy_transform.translation.y.lerp(target_y, 2.0 * delta_time);
        } else {
            // Player is not too close - always follow regardless of distance
            enemy_ai.is_moving = true;
                
                // Check if player has moved significantly
                let player_moved = if let Some(last_pos) = enemy_ai.path_memory.last() {
                    player_pos.distance(*last_pos) > 1.0 // Only consider significant movement
                } else {
                    true // If no previous position, consider it as moved
                };

                // Check if we need to recalculate path - only check every 3 seconds
                let should_recalculate = time.elapsed_secs() % 3.0 < delta_time && (
                    enemy_ai.path_memory.is_empty() 
                    || enemy_ai.stuck_time >= enemy_ai.max_stuck_time
                    || (enemy_ai.current_target_index >= enemy_ai.path_memory.len() - 1 && player_moved) // Only recalculate at end of path if player moved
                    || (player_pos.distance(enemy_pos) > enemy_ai.detection_range * 1.5 && player_moved) // Only recalculate if player is far AND moved
                );

                if should_recalculate {
                    // Get new path from current position to player
                    let path = nav_data.find_path_to_target(enemy_pos, player_pos, enemy_ai.max_path_memory);
                    
                    info!("AI Movement - Enemy[{:?}] recalculating path:", enemy_id);
                    info!("  Player[{:?}] pos: ({:.1}, {:.1}, {:.1})", player_id, player_pos.x, player_pos.y, player_pos.z);
                    info!("  Enemy pos: ({:.1}, {:.1}, {:.1})", enemy_pos.x, enemy_pos.y, enemy_pos.z);
                    info!("  Distance to player: {:.1}", distance_to_player);
                    info!("  Path points: {}", path.len());
                    for (i, point) in path.iter().enumerate() {
                        info!("    Point {}: ({:.1}, {:.1}, {:.1})", i, point.x, point.y, point.z);
                    }
                    
                    enemy_ai.path_memory = path;
                    enemy_ai.current_target_index = 0;
                    enemy_ai.stuck_time = 0.0;
                }
                
                // Get current target position from path
                let target_pos = if !enemy_ai.path_memory.is_empty() {
                    let path_index = enemy_ai.current_target_index.min(enemy_ai.path_memory.len() - 1);
                    let current_target = enemy_ai.path_memory[path_index];
                    
                    // Check if we're at the last point in the path
                    let is_last_point = path_index == enemy_ai.path_memory.len() - 1;
                    let distance_to_player = enemy_pos.distance(player_pos);
                    
                    // If at last point, check player distance
                    if is_last_point {
                        if distance_to_player < enemy_ai.detection_range {
                            // Close enough to target player directly
                            player_pos
                        } else {
                            // Too far, stay at current point until player moves
                            current_target
                        }
                    } else {
                                                // Move to next point if close enough to current target
                    let dist_to_target = enemy_pos.distance(current_target);
                    let dist_to_next = if path_index + 1 < enemy_ai.path_memory.len() {
                        enemy_pos.distance(enemy_ai.path_memory[path_index + 1])
                    } else {
                        f32::MAX
                    };
                    
                    // Transition to next point if:
                    // 1. Close enough to current target OR
                    // 2. Closer to next target than current target OR
                    // 3. Been at current target too long OR
                    // 4. Moving away from target
                    let moving_away = {
                        let current_dist = enemy_pos.distance(current_target);
                        let prev_dist = enemy_ai.last_position.distance(current_target);
                        current_dist > prev_dist
                    };
                    
                    if dist_to_target < 1.0 || // Distance threshold
                       (dist_to_next < dist_to_target && dist_to_next < enemy_ai.detection_range) ||
                       enemy_ai.stuck_time > 0.5 || // Stuck time threshold
                       moving_away { // Moving away from target
                        let next_index = (path_index + 1).min(enemy_ai.path_memory.len() - 1);
                        
                        info!("AI Movement - Enemy[{:?}] reached waypoint:", enemy_id);
                        info!("  Current pos: ({:.1}, {:.1}, {:.1})", enemy_pos.x, enemy_pos.y, enemy_pos.z);
                        info!("  Moving to next point: {} -> {}", path_index, next_index);
                        info!("  Distance to current: {:.1}, Distance to next: {:.1}", dist_to_target, dist_to_next);
                        info!("  Stuck time: {:.1}", enemy_ai.stuck_time);
                        info!("  Moving away: {}", moving_away);
                        enemy_ai.current_target_index = next_index;
                        enemy_ai.stuck_time = 0.0; // Reset stuck timer when reaching waypoint
                    }
                    
                    // Log movement progress
                    if time.elapsed_secs() % 1.0 < delta_time { // Log every second
                        info!("AI Movement - Enemy[{:?}] status:", enemy_id);
                        info!("  Current pos: ({:.1}, {:.1}, {:.1})", enemy_pos.x, enemy_pos.y, enemy_pos.z);
                        info!("  Target pos: ({:.1}, {:.1}, {:.1})", current_target.x, current_target.y, current_target.z);
                        info!("  Distance to target: {:.1}", dist_to_target);
                        info!("  Current waypoint: {}/{}", path_index + 1, enemy_ai.path_memory.len());
                        info!("  Stuck time: {:.1}", enemy_ai.stuck_time);
                    }
                    
                    current_target
                    }
                } else {
                    // This should never happen with improved pathfinding
                    player_pos
                };
                
                // Calculate direction and velocity
                let direction = target_pos - enemy_pos;
                let distance = direction.length();
                
                // Calculate normalized direction and target velocity
                let (direction_normalized, target_velocity) = if distance > 0.1 {
                    let dir_norm = direction / distance; // Manual normalization
                    
                    // Always move at full speed to follow path
                    let target_vel = dir_norm * enemy_ai.move_speed;
                    (dir_norm, target_vel)
                } else {
                    (Vec3::ZERO, Vec3::ZERO)
                };
                
                // Apply velocity
                if distance > 0.1 {
                    // Set linear velocity directly
                    enemy_velocity.x = target_velocity.x;
                    enemy_velocity.y = 0.0;
                    enemy_velocity.z = target_velocity.z;
                    enemy_ai.is_moving = true;
                    
                    // Log movement for debugging - only every 3 seconds
                    if time.elapsed_secs() % 3.0 < delta_time {
                        info!("AI Movement - Enemy[{:?}] movement:", enemy_id);
                        info!("  Current pos: ({:.1}, {:.1}, {:.1})", enemy_pos.x, enemy_pos.y, enemy_pos.z);
                        info!("  Target pos: ({:.1}, {:.1}, {:.1})", target_pos.x, target_pos.y, target_pos.z);
                        info!("  Distance: {:.1}", distance);
                        info!("  Velocity: ({:.1}, {:.1}, {:.1})", enemy_velocity.x, enemy_velocity.y, enemy_velocity.z);
                    }
                } else {
                    // Stop if we're at the target
                    enemy_velocity.x = 0.0;
                    enemy_velocity.y = 0.0;
                    enemy_velocity.z = 0.0;
                    enemy_ai.is_moving = false;
                }
                
                // Log velocity for debugging - only every 3 seconds
                if time.elapsed_secs() % 3.0 < delta_time {
                    info!("AI Movement - Enemy[{:?}] velocity:", enemy_id);
                    info!("  Direction: ({:.1}, {:.1}, {:.1})", direction_normalized.x, direction_normalized.y, direction_normalized.z);
                    info!("  Target velocity: ({:.1}, {:.1}, {:.1})", target_velocity.x, target_velocity.y, target_velocity.z);
                    info!("  Applied velocity: ({:.1}, {:.1}, {:.1})", enemy_velocity.x, enemy_velocity.y, enemy_velocity.z);
                    info!("  Distance to target: {:.1}", distance);
                    
                    info!("AI Movement - Enemy[{:?}] status:", enemy_id);
                    info!("  Current pos: ({:.1}, {:.1}, {:.1})", enemy_pos.x, enemy_pos.y, enemy_pos.z);
                    info!("  Current velocity: ({:.1}, {:.1}, {:.1})", enemy_velocity.x, enemy_velocity.y, enemy_velocity.z);
                }
                
                // Check for wall collisions using spatial query
                let ray_start = enemy_pos;
                let ray_direction = Dir3::new(Vec3::new(enemy_velocity.x, 0.0, enemy_velocity.z).normalize()).unwrap_or(Dir3::Y);
                let ray_distance = enemy_ai.move_speed * delta_time;
                
                let hit = spatial_query.cast_ray(
                    ray_start,
                    ray_direction,
                    ray_distance,
                    true,
                    &SpatialQueryFilter::default(),
                );
                
                let mut final_velocity = Vec3::new(enemy_velocity.x, enemy_velocity.y, enemy_velocity.z);
                
                // If we hit a wall, try to find an alternative path
                if let Some(_hit) = hit {
                    // Try to move around the obstacle by checking perpendicular directions
                    let perpendicular1 = Dir3::new(Vec3::new(-ray_direction.as_vec3().z, 0.0, ray_direction.as_vec3().x)).unwrap_or(Dir3::X);
                    let perpendicular2 = Dir3::new(Vec3::new(ray_direction.as_vec3().z, 0.0, -ray_direction.as_vec3().x)).unwrap_or(Dir3::X);
                    
                    let hit1 = spatial_query.cast_ray(
                        ray_start,
                        perpendicular1,
                        ray_distance,
                        true,
                        &SpatialQueryFilter::default(),
                    );
                    
                    let hit2 = spatial_query.cast_ray(
                        ray_start,
                        perpendicular2,
                        ray_distance,
                        true,
                        &SpatialQueryFilter::default(),
                    );
                    
                    // Choose the direction with more space
                    if hit1.is_none() && hit2.is_none() {
                        // Both directions are free, choose the one closer to target
                        let dir1_towards_target = perpendicular1.as_vec3().dot((target_pos - enemy_pos).normalize());
                        let dir2_towards_target = perpendicular2.as_vec3().dot((target_pos - enemy_pos).normalize());
                        
                        if dir1_towards_target.abs() > dir2_towards_target.abs() {
                            final_velocity = perpendicular1.as_vec3() * enemy_ai.move_speed;
                        } else {
                            final_velocity = perpendicular2.as_vec3() * enemy_ai.move_speed;
                        }
                    } else if hit1.is_none() {
                        final_velocity = perpendicular1.as_vec3() * enemy_ai.move_speed;
                    } else if hit2.is_none() {
                        final_velocity = perpendicular2.as_vec3() * enemy_ai.move_speed;
                    } else {
                        // Both directions blocked, reduce speed and try to push through
                        final_velocity = final_velocity * 0.3;
                    }
                }
                
                // Check boundary constraints before applying movement
                let proposed_pos = enemy_pos + Vec3::new(final_velocity.x, 0.0, final_velocity.z) * delta_time;
                
                // Check if proposed position would be outside boundaries
                let would_be_outside = proposed_pos.x < boundary_constraint.min_x 
                    || proposed_pos.x > boundary_constraint.max_x 
                    || proposed_pos.z < boundary_constraint.min_z 
                    || proposed_pos.z > boundary_constraint.max_z;
                
                // If movement would take us outside boundaries, reduce or stop movement
                if would_be_outside {
                    // Calculate how much we can move without going outside boundaries
                    let mut clamped_velocity = final_velocity;
                    
                    if proposed_pos.x < boundary_constraint.min_x || proposed_pos.x > boundary_constraint.max_x {
                        clamped_velocity.x = 0.0;
                    }
                    if proposed_pos.z < boundary_constraint.min_z || proposed_pos.z > boundary_constraint.max_z {
                        clamped_velocity.z = 0.0;
                    }
                    
                    // Apply clamped velocity
                    enemy_velocity.x = enemy_velocity.x.lerp(clamped_velocity.x, 5.0 * delta_time);
                    enemy_velocity.z = enemy_velocity.z.lerp(clamped_velocity.z, 5.0 * delta_time);
                } else {
                    // Normal movement within boundaries
                    enemy_velocity.x = enemy_velocity.x.lerp(final_velocity.x, 5.0 * delta_time);
                    enemy_velocity.z = enemy_velocity.z.lerp(final_velocity.z, 5.0 * delta_time);
                }
                
                enemy_velocity.y = 0.0;
                
                // Rotate to face movement direction
                let movement_direction = Vec2::new(enemy_velocity.x, enemy_velocity.z).normalize();
                if movement_direction.length() > 0.1 {
                    let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(movement_direction.x, 0.0, movement_direction.y));
                    enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, 3.0 * delta_time);
                }
                
                // Match player elevation smoothly
                let target_y = player_pos.y;
                enemy_transform.translation.y = enemy_transform.translation.y.lerp(target_y, 2.0 * delta_time);
                
                // Update animation state
                let horizontal_speed = Vec2::new(enemy_velocity.x, enemy_velocity.z).length();
                if horizontal_speed > 0.1 {
                    animation_state.forward_hold_time += delta_time;
                } else {
                    animation_state.forward_hold_time = 0.0;
                }
                
                // Update path target if we're close to current target
                if !enemy_ai.path_memory.is_empty() && enemy_pos.distance(target_pos) < 2.0 {
                    enemy_ai.current_target_index = (enemy_ai.current_target_index + 1) % enemy_ai.path_memory.len();
                }
            }
    }
}

/// System that handles enemy animations
fn enemy_ai_animations(
    mut enemy_query: Query<(&mut GltfAnimations, &mut AnimationState, &EnemyAI), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (mut animations, mut animation_state, enemy_ai) in &mut enemy_query {
        // Use AI state directly - much simpler and more reliable
        let is_moving = enemy_ai.is_moving;
        
        // Determine target animation based on state - match player logic exactly
        let target_animation = if !is_moving {
            1 // Idle animation when not moving (same as player's gameplay idle)
        } else {
            4 // Walking animation when moving (try animation 4 for enemy)
        };

        // Only change animation if we need to - no timer, immediate switching like player
        if target_animation != animation_state.current_animation {
            if let Some(animation) = animations.get_by_number(target_animation) {
                if let Ok(mut player) = animation_players.get_mut(animations.animation_player) {
                    player.stop_all();
                    player.play(animation).repeat();
                    animation_state.current_animation = target_animation;
                }
            }
        }
    }
} 