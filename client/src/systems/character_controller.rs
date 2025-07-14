use crate::constants::movement::CharacterMovementConfig;
use avian3d::{math::*, prelude::*};
use bevy::prelude::*;
use bevy_gltf_animation::prelude::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastInputDirection>()
            .init_resource::<JumpCooldown>()
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (movement, apply_movement_damping, update_animations).chain(),
            );
    }
}

/// An event sent for a movement input action.
#[derive(Event, Debug)]
pub enum MovementAction {
    Move(Vector2),
    Jump,
    FightMove1,
    FightMove2,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// Component to track stair climbing state for smoother transitions
#[derive(Component)]
pub struct StairClimbingState {
    // Reserved for future stair climbing state tracking
}

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(pub Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller with animation support.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    pub character_controller: CharacterController,
    pub body: RigidBody,
    pub collider: Collider,
    pub ground_caster: ShapeCaster,
    pub locked_axes: LockedAxes,
    pub movement: MovementBundle,
    pub animation_state: AnimationState,
    pub stair_climbing_state: StairClimbingState,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    jump_impulse: JumpImpulse,
}

impl MovementBundle {
    pub const fn new(_acceleration: Scalar, _damping: Scalar, jump_impulse: Scalar) -> Self {
        Self {
            jump_impulse: JumpImpulse(jump_impulse),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(CharacterMovementConfig::MOVEMENT_ACCELERATION, 0.9, 7.0)
    }
}

/// Add a resource to store the last movement input for camera rotation
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct LastInputDirection(pub Vec2);

/// Add a resource to track jump cooldown
#[derive(Resource, Debug)]
pub struct JumpCooldown {
    pub last_jump_time: f32,
    pub cooldown_duration: f32,
}

impl Default for JumpCooldown {
    fn default() -> Self {
        Self {
            last_jump_time: 0.0,
            cooldown_duration: 1.5, // 1.5 second cooldown
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &JumpImpulse,
        &mut LinearVelocity,
        &mut Transform,
        &mut AnimationState,
    )>,
    mut jump_cooldown: ResMut<JumpCooldown>,
) {
    let delta_time = time.delta_secs();
    jump_cooldown.last_jump_time += delta_time;

    for event in movement_event_reader.read() {
        for (jump_impulse, mut linear_velocity, mut transform, mut animation_state) in
            &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    // Smooth rotation
                    if direction.x != 0.0 {
                        let target_rotation =
                            -direction.x * CharacterMovementConfig::ROTATION_SPEED * delta_time;
                        transform.rotate_y(target_rotation);
                    }

                    // Get movement vectors
                    let forward = transform.forward();
                    let right = transform.right();
                    let movement_direction = (forward * -direction.y) + (right * direction.x);

                    // Calculate target velocity
                    let target_speed = if animation_state.forward_hold_time >= 3.0 {
                        CharacterMovementConfig::MAX_RUN_SPEED * direction.length()
                    } else {
                        CharacterMovementConfig::MAX_SPEED * direction.length()
                    };
                    let current_speed = Vec2::new(linear_velocity.x, linear_velocity.z).length();

                    // Smooth acceleration/deceleration
                    let acceleration = if target_speed > current_speed {
                        CharacterMovementConfig::MOVEMENT_ACCELERATION
                    } else {
                        CharacterMovementConfig::MOVEMENT_DECELERATION
                    };

                    // Apply movement with stability for running
                    let target_velocity = movement_direction * target_speed;

                    // Use more stable interpolation for running
                    let interpolation_factor = if animation_state.forward_hold_time >= 3.0 {
                        // More stable interpolation for running
                        (acceleration * delta_time).min(0.8)
                    } else {
                        // Normal interpolation for walking
                        acceleration * delta_time
                    };

                    // Smoothly interpolate current velocity to target velocity
                    linear_velocity.x = linear_velocity
                        .x
                        .lerp(target_velocity.x, interpolation_factor);
                    linear_velocity.z = linear_velocity
                        .z
                        .lerp(target_velocity.z, interpolation_factor);
                }
                MovementAction::Jump => {
                    if jump_cooldown.last_jump_time >= jump_cooldown.cooldown_duration {
                        linear_velocity.y = jump_impulse.0;
                        jump_cooldown.last_jump_time = 0.0;
                    }
                }
                MovementAction::FightMove1 => {
                    // Trigger fight move 1 animation
                    animation_state.fight_move_1 = true;
                }
                MovementAction::FightMove2 => {
                    // Trigger fight move 2 animation
                    animation_state.fight_move_2 = true;
                }
            }
        }
    }
}

/// Applies movement damping and prevents unwanted climbing
fn apply_movement_damping(
    mut query: Query<(&mut LinearVelocity, &AnimationState, &Transform), With<CharacterController>>,
) {
    for (mut linear_velocity, animation_state, _transform) in &mut query {
        // Check for unwanted climbing behavior
        let horizontal_speed = Vec2::new(linear_velocity.x, linear_velocity.z).length();
        let is_moving_horizontally = horizontal_speed > 0.1;
        let is_rising_gradually = linear_velocity.y > 0.1 && linear_velocity.y < 2.0;

        // If moving horizontally and rising gradually, this is likely unwanted climbing
        if is_moving_horizontally && is_rising_gradually {
            // Reduce the climbing effect
            linear_velocity.y *= 0.1;
            // Also reduce horizontal movement slightly to prevent getting stuck
            linear_velocity.x *= 0.8;
            linear_velocity.z *= 0.8;
        }

        // Apply different damping based on movement state
        let damping_factor = if animation_state.forward_hold_time >= 3.0 {
            // More stable damping for running
            CharacterMovementConfig::AIR_RESISTANCE * 0.95
        } else {
            // Normal damping for walking
            CharacterMovementConfig::AIR_RESISTANCE
        };

        // Apply air resistance
        linear_velocity.x *= damping_factor;
        linear_velocity.z *= damping_factor;

        // Prevent tiny residual movement
        if linear_velocity.x.abs() < CharacterMovementConfig::MIN_MOVEMENT_THRESHOLD {
            linear_velocity.x = 0.0;
        }
        if linear_velocity.z.abs() < CharacterMovementConfig::MIN_MOVEMENT_THRESHOLD {
            linear_velocity.z = 0.0;
        }
    }
}

#[derive(Component)]
pub struct AnimationState {
    pub forward_hold_time: f32,
    pub current_animation: usize,
    pub fight_move_1: bool,
    pub fight_move_2: bool,
}

/// Updates animations based on character movement
fn update_animations(
    mut query: Query<(&LinearVelocity, &mut GltfAnimations, &mut AnimationState)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (velocity, mut animations, mut animation_state) in &mut query {
        let horizontal_velocity = Vec2::new(velocity.x, velocity.z);
        let is_moving = horizontal_velocity.length() > 0.1;

        // Check if fight moves are active
        if animation_state.fight_move_1 {
            // Play fight move 1 animation (index 5)
            if animation_state.current_animation != 5 {
                if let Some(animation) = animations.get_by_number(5) {
                    if let Ok(mut player) = animation_players.get_mut(animations.animation_player) {
                        player.stop_all();
                        player.play(animation);
                        animation_state.current_animation = 5;
                    }
                }
            }
            // Check if animation has finished
            if let Ok(player) = animation_players.get(animations.animation_player) {
                if player.all_finished() {
                    animation_state.fight_move_1 = false;
                }
            }
        } else if animation_state.fight_move_2 {
            // Play fight move 2 animation (index 6)
            if animation_state.current_animation != 6 {
                if let Some(animation) = animations.get_by_number(6) {
                    if let Ok(mut player) = animation_players.get_mut(animations.animation_player) {
                        player.stop_all();
                        player.play(animation);
                        animation_state.current_animation = 6;
                    }
                }
            }
            // Check if animation has finished
            if let Ok(player) = animation_players.get(animations.animation_player) {
                if player.all_finished() {
                    animation_state.fight_move_2 = false;
                }
            }
        } else {
            // Normal movement animations
            let target_animation = if !is_moving {
                3 // Idle
            } else if animation_state.forward_hold_time >= 3.0 {
                4 // Running
            } else {
                7 // Regular walking
            };

            // Only change animation if we need to
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
}

/// Sets up initial idle animation when character is spawned
pub fn setup_idle_animation(
    trigger: Trigger<OnAdd, GltfAnimations>,
    mut _commands: Commands,
    mut players: Query<&mut GltfAnimations>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    let Ok(mut gltf_animations) = players.get_mut(trigger.target()) else {
        return;
    };
    let mut player = animation_players
        .get_mut(gltf_animations.animation_player)
        .unwrap();
    let animation = gltf_animations.get_by_number(2).unwrap();
    player.stop_all();
    player.play(animation).repeat();
}

impl CharacterControllerBundle {
    pub fn new() -> Self {
        // Improved collider for better collision handling
        let length = 0.5; // Reduced height to prevent climbing
        let radius = 0.2; // Smaller radius for more precise collision
        let offset = Vec3::new(0.0, (length / 2.0) + radius, 0.0);
        let capsule = Collider::capsule(radius, length);
        let collider = Collider::compound(vec![(offset, Quat::IDENTITY, capsule)]);

        // Smaller ground caster for more precise ground detection
        let caster_shape = Collider::sphere(0.2);

        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(CharacterMovementConfig::GROUND_SNAP_DISTANCE + 0.1),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::new(CharacterMovementConfig::MOVEMENT_ACCELERATION, 0.9, 7.0),
            animation_state: AnimationState {
                forward_hold_time: 0.0,
                current_animation: 2, // Start with idle animation
                fight_move_1: false,
                fight_move_2: false,
            },
            stair_climbing_state: StairClimbingState {},
        }
    }
}
