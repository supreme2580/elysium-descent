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
    pub move_speed: f32,
    pub is_moving: bool,
    pub detection_range: f32,
    pub player_positions: Vec<Vec3>, // Store unvisited player positions
    pub last_record_time: f32, // Time of last position recording
    pub record_interval: f32, // Interval between position recordings (in seconds)
    pub current_target_index: usize, // Current target position index
    pub stuck_time: f32, // Time enemy has been stuck
    pub last_position: Vec3, // Last position for stuck detection
    pub stuck_threshold: f32, // Distance threshold for stuck detection
    pub position_reached_threshold: f32, // Distance threshold to consider a position reached
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            move_speed: 4.5, // Slightly slower than player's base speed
            is_moving: false,
            detection_range: 30.0, // Increased range to track player better
            player_positions: Vec::new(),
            last_record_time: 0.0,
            record_interval: 0.5, // Record position more frequently
            current_target_index: 0,
            stuck_time: 0.0,
            last_position: Vec3::ZERO,
            stuck_threshold: 0.3,
            position_reached_threshold: 1.0, // Smaller threshold to be more precise
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
    time: Res<Time>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    mut enemy_query: Query<(&Transform, &mut EnemyAI), With<Enemy>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let player_pos = player_transform.translation;
    let current_time = time.elapsed_secs();
    
    for (enemy_transform, mut enemy_ai) in &mut enemy_query {
        // Only record if enough time has passed since last recording
        if current_time - enemy_ai.last_record_time >= enemy_ai.record_interval {
            let enemy_pos = enemy_transform.translation;
            let distance_to_player = enemy_pos.distance(player_pos);
            
            // Clear recorded positions when close enough to directly chase
            if distance_to_player <= enemy_ai.detection_range {
                enemy_ai.player_positions.clear();
                enemy_ai.current_target_index = 0;
            } else {
                // Record positions when far - check if this position is significantly different
                let is_new_position = enemy_ai.player_positions.last()
                    .map_or(true, |last_pos| {
                        // Check both horizontal distance and height difference
                        let horizontal_dist = Vec2::new(player_pos.x - last_pos.x, player_pos.z - last_pos.z).length();
                        let height_diff = (player_pos.y - last_pos.y).abs();
                        horizontal_dist > 1.5 || height_diff > 0.5
                    });
                
                if is_new_position {
                    // Add current player position
                    enemy_ai.player_positions.push(player_pos);
                    
                    // Limit stored positions to prevent memory growth
                    if enemy_ai.player_positions.len() > 20 {
                        enemy_ai.player_positions.remove(0);
                        if enemy_ai.current_target_index > 0 {
                            enemy_ai.current_target_index -= 1;
                        }
                    }
                }
            }
            
            enemy_ai.last_record_time = current_time;
        }
    }
}

/// System that handles enemy movement towards the player
fn enemy_ai_movement(
    time: Res<Time>,
    mut enemy_query: Query<(Entity, &mut Transform, &mut EnemyAI, &mut AnimationState), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    boundary_constraint: Option<Res<BoundaryConstraint>>,
) {
    let delta_time = time.delta_secs();
    
    // Find the player
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Get boundary constraint if available
    let Some(boundary_constraint) = boundary_constraint else {
        return; // No boundary constraints, skip boundary checking
    };

    for (_enemy_id, mut enemy_transform, mut enemy_ai, mut animation_state) in &mut enemy_query {
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

        // Determine target position
        let target_pos = if distance_to_player <= enemy_ai.detection_range {
            // Directly chase player when close
            player_pos
        } else if !enemy_ai.player_positions.is_empty() {
            // Follow recorded path when far
            let current_index = enemy_ai.current_target_index.min(enemy_ai.player_positions.len() - 1);
            enemy_ai.player_positions[current_index]
        } else {
            // No target if no path and too far
            enemy_pos
        };

        // Calculate movement
        let direction = target_pos - enemy_pos;
        let distance = direction.length();
        
        // Handle close-range behavior
        let min_distance = 4.0;
        let should_move = if distance_to_player <= enemy_ai.detection_range {
            // Direct chase mode - maintain minimum distance
            distance_to_player > min_distance
        } else {
            // Path following mode - always move towards target
            distance > 0.5
        };

        if should_move && distance > 0.1 {
            // Calculate movement direction
            let move_direction = direction / distance;
            let movement_speed = if distance_to_player < min_distance + 2.0 {
                // Slow down when getting close
                enemy_ai.move_speed * 0.3
            } else {
                enemy_ai.move_speed
            };
            
            // Calculate movement for this frame
            let movement = move_direction * movement_speed * delta_time;
            
            // Check boundary constraints
            let proposed_pos = enemy_pos + movement;
            let mut final_movement = movement;
            
            // Clamp to boundaries
            if proposed_pos.x < boundary_constraint.min_x || proposed_pos.x > boundary_constraint.max_x {
                final_movement.x = 0.0;
            }
            if proposed_pos.z < boundary_constraint.min_z || proposed_pos.z > boundary_constraint.max_z {
                final_movement.z = 0.0;
            }
            
            // Apply movement directly to transform (kinematic body)
            enemy_transform.translation += final_movement;
            enemy_ai.is_moving = true;
            
            // Face movement direction
            if final_movement.length() > 0.01 {
                let look_direction = Vec2::new(final_movement.x, final_movement.z).normalize();
                let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(look_direction.x, 0.0, look_direction.y));
                enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, 5.0 * delta_time);
            }
        } else {
            enemy_ai.is_moving = false;
        }

        // Handle path progression for recorded positions
        if !enemy_ai.player_positions.is_empty() && distance_to_player > enemy_ai.detection_range {
            let current_index = enemy_ai.current_target_index.min(enemy_ai.player_positions.len() - 1);
            let current_target = enemy_ai.player_positions[current_index];
            
            // Check if we've reached the current target
            let distance_to_target = enemy_pos.distance(current_target);
            if distance_to_target < enemy_ai.position_reached_threshold {
                // Move to next position
                if current_index < enemy_ai.player_positions.len() - 1 {
                    enemy_ai.current_target_index = current_index + 1;
                }
            }
        }

        // Update animation state
        if enemy_ai.is_moving {
            animation_state.forward_hold_time += delta_time;
        } else {
            animation_state.forward_hold_time = 0.0;
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