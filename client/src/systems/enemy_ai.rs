use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;
use avian3d::{math::*, prelude::*};
use crate::systems::character_controller::AnimationState;
use crate::systems::boundary::BoundaryConstraint;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Component to track enemy AI state
#[derive(Component)]
pub struct EnemyAI {
    pub attack_range: f32,
    pub move_speed: f32,
    pub is_moving: bool,
    pub detection_range: f32,
    pub last_attack_time: f32,
    pub attack_cooldown: f32,
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
            attack_range: 3.0,
            move_speed: 2.5, // Slightly slower than player for better gameplay
            is_moving: false,
            detection_range: 15.0, // Detect player from further away
            last_attack_time: 0.0,
            attack_cooldown: 2.0, // 2 seconds between attacks
            path_memory: Vec::new(),
            max_path_memory: 20, // Remember last 20 player positions
            current_target_index: 0,
            stuck_time: 0.0,
            max_stuck_time: 3.0, // 3 seconds before recalculating path
            last_position: Vec3::ZERO,
            stuck_threshold: 0.5, // Consider stuck if moved less than 0.5 units
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
        app.add_systems(
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
    mut enemy_query: Query<(&mut Transform, &mut LinearVelocity, &mut EnemyAI, &mut AnimationState), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    boundary_constraint: Option<Res<BoundaryConstraint>>,
    spatial_query: SpatialQuery,
) {
    let delta_time = time.delta_secs();
    let _current_time = time.elapsed_secs();
    
    // Find the player
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Get boundary constraint if available
    let boundary_constraint = if let Some(constraint) = boundary_constraint {
        constraint
    } else {
        return; // No boundary constraints, skip boundary checking
    };

    for (mut enemy_transform, mut enemy_velocity, mut enemy_ai, mut animation_state) in &mut enemy_query {
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
                
                // Determine target position - use path memory if available and not stuck
                let target_pos = if !enemy_ai.path_memory.is_empty() && enemy_ai.stuck_time < enemy_ai.max_stuck_time {
                    // Use path memory for intelligent following
                    let path_index = enemy_ai.current_target_index.min(enemy_ai.path_memory.len() - 1);
                    enemy_ai.path_memory[path_index]
                } else {
                    // Direct line to player if stuck or no path memory
                    player_pos
                };
                
                let direction_to_target = (target_pos - enemy_pos).normalize();
                let target_velocity = direction_to_target * enemy_ai.move_speed;
                
                // Check for wall collisions using spatial query
                let ray_start = enemy_pos;
                let ray_direction = Dir3::new(Vec3::new(target_velocity.x, 0.0, target_velocity.z).normalize()).unwrap_or(Dir3::Y);
                let ray_distance = enemy_ai.move_speed * delta_time;
                
                let hit = spatial_query.cast_ray(
                    ray_start,
                    ray_direction,
                    ray_distance,
                    true,
                    &SpatialQueryFilter::default(),
                );
                
                let mut final_velocity = target_velocity;
                
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
                        final_velocity = target_velocity * 0.3;
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