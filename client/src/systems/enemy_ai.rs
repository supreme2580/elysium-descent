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
                enemy_ai_movement,
                enemy_ai_animations,
            ).chain(),
        );
    }
}

/// System that handles enemy movement towards the player
fn enemy_ai_movement(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut LinearVelocity, &mut EnemyAI, &mut AnimationState), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    boundary_constraint: Option<Res<BoundaryConstraint>>,
) {
    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs();
    
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

        // Only act if player is within detection range
        if distance_to_player <= enemy_ai.detection_range {
            // Check if we should attack (close enough and cooldown expired)
            let can_attack = distance_to_player <= enemy_ai.attack_range 
                && (current_time - enemy_ai.last_attack_time) >= enemy_ai.attack_cooldown;
            
            if can_attack {
                // Attack behavior - stop moving and prepare attack
                enemy_ai.is_moving = false;
                enemy_ai.last_attack_time = current_time;
                
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
            } else if distance_to_player > enemy_ai.attack_range {
                // Move towards player if not in attack range
                enemy_ai.is_moving = true;
                
                let direction_to_player = (player_pos - enemy_pos).normalize();
                let target_velocity = direction_to_player * enemy_ai.move_speed;
                
                // Check boundary constraints before applying movement
                let proposed_pos = enemy_pos + Vec3::new(target_velocity.x, 0.0, target_velocity.z) * delta_time;
                
                // Check if proposed position would be outside boundaries
                let would_be_outside = proposed_pos.x < boundary_constraint.min_x 
                    || proposed_pos.x > boundary_constraint.max_x 
                    || proposed_pos.z < boundary_constraint.min_z 
                    || proposed_pos.z > boundary_constraint.max_z;
                
                // If movement would take us outside boundaries, reduce or stop movement
                if would_be_outside {
                    // Calculate how much we can move without going outside boundaries
                    let mut clamped_velocity = target_velocity;
                    
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
                    enemy_velocity.x = enemy_velocity.x.lerp(target_velocity.x, 5.0 * delta_time);
                    enemy_velocity.z = enemy_velocity.z.lerp(target_velocity.z, 5.0 * delta_time);
                }
                
                enemy_velocity.y = 0.0;
                
                // Rotate to face player
                let direction_2d = Vec2::new(direction_to_player.x, direction_to_player.z).normalize();
                let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(direction_2d.x, 0.0, direction_2d.y));
                enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, 3.0 * delta_time);
                
                // Keep on ground
                enemy_transform.translation.y = -1.65;
                
                // Update animation state
                let horizontal_speed = Vec2::new(enemy_velocity.x, enemy_velocity.z).length();
                if horizontal_speed > 0.1 {
                    animation_state.forward_hold_time += delta_time;
                } else {
                    animation_state.forward_hold_time = 0.0;
                }
            } else {
                // In attack range but on cooldown - stay still and face player
                enemy_ai.is_moving = false;
                
                // Stop moving
                enemy_velocity.x = 0.0;
                enemy_velocity.z = 0.0;
                enemy_velocity.y = 0.0;
                animation_state.forward_hold_time = 0.0;
                
                // Face the player
                let direction_to_player = (player_pos - enemy_pos).normalize();
                let direction_2d = Vec2::new(direction_to_player.x, direction_to_player.z).normalize();
                let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(direction_2d.x, 0.0, direction_2d.y));
                enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, 5.0 * delta_time);
            }
        } else {
            // Player out of detection range - idle behavior
            enemy_ai.is_moving = false;
            
            // Stop moving
            enemy_velocity.x = 0.0;
            enemy_velocity.z = 0.0;
            enemy_velocity.y = 0.0;
            animation_state.forward_hold_time = 0.0;
            
            // Keep on ground
            enemy_transform.translation.y = -1.65;
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