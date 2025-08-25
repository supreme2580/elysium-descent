use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;
use avian3d::prelude::*;
use crate::systems::character_controller::AnimationState;

/// Marker component for enemy entities
#[derive(Component)]
pub struct Enemy;

/// Simple path follower that traces player's exact steps
#[derive(Component)]
pub struct PathFollower {
    pub move_speed: f32,
    pub is_moving: bool,
    pub player_path: Vec<Vec3>, // Queue of player positions to follow
    pub last_record_time: f32,
    pub record_interval: f32, // How often to record player position
}

impl Default for PathFollower {
    fn default() -> Self {
        Self {
            move_speed: 4.0, // Slightly slower than player
            is_moving: false,
            player_path: Vec::new(),
            last_record_time: 0.0,
            record_interval: 0.2, // Record every 0.2 seconds
        }
    }
}

/// Bundle for enemy entities
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub path_follower: PathFollower,
    pub animation_state: AnimationState,
    pub body: RigidBody,
    pub collider: Collider,
    pub locked_axes: LockedAxes,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            path_follower: PathFollower::default(),
            animation_state: AnimationState {
                forward_hold_time: 0.0,
                current_animation: 1, // Start with idle animation
                fight_move_1: false,
                fight_move_2: false,
            },
            body: RigidBody::Kinematic,
            collider: Collider::capsule(0.5, 1.5),
            locked_axes: LockedAxes::ROTATION_LOCKED,
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
                follow_player_path,
                enemy_animations,
            ).chain(),
        );
    }
}

/// Records player positions for enemies to follow
fn record_player_path(
    time: Res<Time>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
    mut enemy_query: Query<&mut PathFollower, With<Enemy>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let current_time = time.elapsed_secs();
    
    for mut path_follower in &mut enemy_query {
        // Record player position at intervals
        if current_time - path_follower.last_record_time >= path_follower.record_interval {
            let player_pos = player_transform.translation;
            
            // Only add if position is different enough from last recorded position
            let should_record = path_follower.player_path.last()
                .map_or(true, |last_pos| last_pos.distance(player_pos) > 0.8);
            
            if should_record {
                path_follower.player_path.push(player_pos);
                
                // Keep memory efficient - limit to 50 positions max
                if path_follower.player_path.len() > 50 {
                    path_follower.player_path.remove(0);
                }
                
                // Log occasionally for debugging
                if path_follower.player_path.len() % 10 == 0 {
                    info!("Enemy path following: {} positions in queue", path_follower.player_path.len());
                }
            }
            
            path_follower.last_record_time = current_time;
        }
    }
}

/// Enemy follows the recorded player path
fn follow_player_path(
    time: Res<Time>,
    mut enemy_query: Query<(&mut Transform, &mut PathFollower, &mut AnimationState), With<Enemy>>,
    player_query: Query<&Transform, (With<crate::systems::character_controller::CharacterController>, Without<Enemy>)>,
) {
    let delta_time = time.delta_secs();
    
    // Get player position for proximity check
    let Ok(player_transform) = player_query.single() else {
        return;
    };
        let player_pos = player_transform.translation;
    
    for (mut transform, mut path_follower, mut animation_state) in &mut enemy_query {
        let current_pos = transform.translation;
        let distance_to_player = current_pos.distance(player_pos);
        
        // Stop moving if within 5 units of player
        if distance_to_player <= 5.0 {
            path_follower.is_moving = false;
            animation_state.forward_hold_time = 0.0;
            // Log occasionally when stopped due to proximity
            if (time.elapsed_secs() * 10.0) as i32 % 30 == 0 {
                info!("Enemy stopped - too close to player ({:.1} units)", distance_to_player);
            }
            continue;
        }
        
        // Check if we have a path to follow
        if path_follower.player_path.is_empty() {
            path_follower.is_moving = false;
            animation_state.forward_hold_time = 0.0;
            continue;
        }
        
        // Get the next position to move towards (first in queue)
        let target_pos = path_follower.player_path[0];
        let direction = target_pos - current_pos;
                let distance = direction.length();
                
        // If we're close enough to the target, remove it from queue and continue
        if distance < 1.2 {
            path_follower.player_path.remove(0); // Memory efficient - remove consumed position
            continue;
        }
        
        // Move towards the target
        let move_direction = direction / distance;
        let movement = move_direction * path_follower.move_speed * delta_time;
        
        // Apply movement
        transform.translation += movement;
        path_follower.is_moving = true;
                    animation_state.forward_hold_time += delta_time;
        
        // Face movement direction
        let look_direction = Vec2::new(movement.x, movement.z);
        if look_direction.length() > 0.01 {
            let look_direction = look_direction.normalize();
            let target_rotation = Quat::from_rotation_arc(Vec3::Z, Vec3::new(look_direction.x, 0.0, look_direction.y));
            transform.rotation = transform.rotation.slerp(target_rotation, 8.0 * delta_time);
            }
    }
}

/// System that handles enemy animations
fn enemy_animations(
    mut enemy_query: Query<(&mut GltfAnimations, &mut AnimationState, &PathFollower), (With<Enemy>, Without<crate::systems::character_controller::CharacterController>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (mut animations, mut animation_state, path_follower) in &mut enemy_query {
        // Determine target animation based on movement state
        let target_animation = if path_follower.is_moving {
            4 // Walking animation when moving
        } else {
            1 // Idle animation when not moving
        };

        // Only change animation if needed
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