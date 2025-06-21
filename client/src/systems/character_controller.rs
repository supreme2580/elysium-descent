use crate::constants::movement::CharacterMovementConfig;
use avian3d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use bevy_gltf_animation::prelude::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastInputDirection>()
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    update_grounded,
                    movement,
                    apply_movement_damping,
                    update_animations,
                )
                    .chain(),
            );
    }
}

/// An event sent for a movement input action.
#[derive(Event, Debug)]
pub enum MovementAction {
    Move(Vector2),
    Jump,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;

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


/// Updates the [`Grounded`] status and handles ground snapping
fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut LinearVelocity), With<CharacterController>>,
    time: Res<Time>,
    mut ray_cast: MeshRayCast,
) {
    for (entity, transform, mut velocity) in &mut query {
        // Cast ray downward to detect ground
        let ray_origin = transform.translation;
        let ray_direction = Dir3::NEG_Y;
        let ray = Ray3d::new(ray_origin, ray_direction);

        // Create settings for ground detection
        let settings = MeshRayCastSettings::default()
            .with_visibility(RayCastVisibility::Any) // Cast against all meshes
            .with_early_exit_test(&|_| true); // Stop at first hit

        // Perform the ground check
        if let Some((_, hit)) = ray_cast.cast_ray(ray, &settings).first() {
            let ground_normal = hit.normal;
            let slope_angle = ground_normal.angle_between(Vec3::Y).to_degrees();

            // Check if we're on a valid slope
            if slope_angle <= CharacterMovementConfig::MAX_SLOPE_ANGLE {
                // Ground snapping for small obstacles
                let distance_to_ground = hit.distance;
                if distance_to_ground <= CharacterMovementConfig::GROUND_SNAP_DISTANCE {
                    // Snap to ground
                    velocity.y = 0.0;
                    commands.entity(entity).insert(Grounded);

                    // Handle stair climbing with improved logic
                    if distance_to_ground > CharacterMovementConfig::GROUND_SNAP_DISTANCE 
                        && distance_to_ground <= CharacterMovementConfig::MAX_STAIR_HEIGHT {
                        
                        // Only try to climb stairs if character is moving forward
                        let horizontal_velocity = Vec2::new(velocity.x, velocity.z);
                        if horizontal_velocity.length() > 0.1 {
                            // Try to climb the stair with multiple detection points
                            let forward = transform.forward();
                            let stair_origin = ray_origin + forward * CharacterMovementConfig::STAIR_DETECTION_DISTANCE;
                            let stair_ray = Ray3d::new(stair_origin, ray_direction);

                            // Perform the stair check
                            if let Some((_, stair_hit)) = ray_cast.cast_ray(stair_ray, &settings).first() {
                                let step_height = distance_to_ground - stair_hit.distance;
                                
                                // Only climb if the step height is reasonable
                                if step_height > 0.05 && step_height <= CharacterMovementConfig::MAX_STAIR_HEIGHT {
                                    // Use physics-based calculation for smoother stair climbing
                                    velocity.y = (step_height / time.delta_secs()) * 1.2; // Reduced multiplier for less aggressive jumping
                                }
                            }
                        }
                    }
                } else {
                    commands.entity(entity).remove::<Grounded>();
                }
            } else {
                // Too steep, slide down
                let slide_direction = (ground_normal - Vec3::Y * ground_normal.y).normalize();
                velocity.x += slide_direction.x * 9.8 * time.delta_secs();
                velocity.z += slide_direction.z * 9.8 * time.delta_secs();
                commands.entity(entity).remove::<Grounded>();
            }
        } else {
            commands.entity(entity).remove::<Grounded>();
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
        Has<Grounded>,
        &AnimationState,
    )>,
) {
    let delta_time = time.delta_secs();

    for event in movement_event_reader.read() {
        for (jump_impulse, mut linear_velocity, mut transform, is_grounded, animation_state) in
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
                    let target_speed = CharacterMovementConfig::MAX_SPEED * direction.length();
                    let current_speed = Vec2::new(linear_velocity.x, linear_velocity.z).length();

                    // Smooth acceleration/deceleration
                    let acceleration = if target_speed > current_speed {
                        CharacterMovementConfig::MOVEMENT_ACCELERATION
                    } else {
                        CharacterMovementConfig::MOVEMENT_DECELERATION
                    };

                    // Apply movement
                    let speed_multiplier = if animation_state.current_animation == 3 {
                        1.5
                    } else {
                        1.0
                    };
                    let target_velocity = movement_direction * target_speed * speed_multiplier;

                    // Smoothly interpolate current velocity to target velocity
                    linear_velocity.x = linear_velocity
                        .x
                        .lerp(target_velocity.x, acceleration * delta_time);
                    linear_velocity.z = linear_velocity
                        .z
                        .lerp(target_velocity.z, acceleration * delta_time);
                }
                MovementAction::Jump => {
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

/// Applies movement damping and ground sticking
fn apply_movement_damping(
    mut query: Query<(
        &mut LinearVelocity,
        Option<&Grounded>,
    ), With<CharacterController>>,
) {
    for (mut linear_velocity, grounded) in &mut query {
        // Apply air resistance when not grounded
        if grounded.is_none() {
            linear_velocity.x *= CharacterMovementConfig::AIR_RESISTANCE;
            linear_velocity.z *= CharacterMovementConfig::AIR_RESISTANCE;
        }

        // Apply ground friction
        if grounded.is_some() {
            linear_velocity.x *= CharacterMovementConfig::GROUND_FRICTION;
            linear_velocity.z *= CharacterMovementConfig::GROUND_FRICTION;
        }

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
}

/// Updates animations based on character movement
fn update_animations(
    mut query: Query<(&LinearVelocity, &mut GltfAnimations, &mut AnimationState)>,
    mut animation_players: Query<&mut AnimationPlayer>,
) {
    for (velocity, mut animations, mut animation_state) in &mut query {
        let horizontal_velocity = Vec2::new(velocity.x, velocity.z);
        let is_moving = horizontal_velocity.length() > 0.1;

        // Determine which animation should be playing
        let target_animation = if !is_moving {
            2 // Idle
        } else if animation_state.forward_hold_time >= 3.0 {
            3 // Special animation after 3 seconds
        } else {
            4 // Regular running
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
        // Improved collider for better stair climbing
        let length = 0.6;  // Reduced height for better step clearance
        let radius = 0.25; // Slightly smaller radius
        let offset = Vec3::new(0.0, (length / 2.0) + radius, 0.0);
        let capsule = Collider::capsule(radius, length);
        let collider = Collider::compound(vec![(offset, Quat::IDENTITY, capsule)]);
        
        // Smaller ground caster for more precise ground detection
        let caster_shape = Collider::sphere(0.3);

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
            .with_max_distance(CharacterMovementConfig::GROUND_SNAP_DISTANCE + 0.1), // Adjusted detection distance
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::new(CharacterMovementConfig::MOVEMENT_ACCELERATION, 0.9, 7.0),
            animation_state: AnimationState {
                forward_hold_time: 0.0,
                current_animation: 2, // Start with idle animation
            },
            stair_climbing_state: StairClimbingState {},
        }
    }
}
