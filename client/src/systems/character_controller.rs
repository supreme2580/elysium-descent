use avian3d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use bevy_gltf_animation::prelude::*;
use crate::{rendering::cameras::player_camera::FlyCam, game::Player};

// Constants for movement tuning
const MAX_SLOPE_ANGLE: f32 = 45.0;
const STAIR_HEIGHT: f32 = 0.6;
const GROUND_SNAP_DISTANCE: f32 = 0.2;
const MOVEMENT_ACCELERATION: f32 = 30.0;
const MOVEMENT_DECELERATION: f32 = 40.0;
const MAX_SPEED: f32 = 5.0;
const ROTATION_SPEED: f32 = 5.0;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastInputDirection>()
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    gamepad_input,
                    update_grounded,
                    movement,
                    apply_movement_damping,
                    camera_follow_player_system,
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

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor;

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
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
}

impl MovementBundle {
    pub const fn new(
        _acceleration: Scalar,
        _damping: Scalar,
        jump_impulse: Scalar,
    ) -> Self {
        Self {
            damping: MovementDampingFactor,
            jump_impulse: JumpImpulse(jump_impulse),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(MOVEMENT_ACCELERATION, 0.9, 7.0)
    }
}

/// Add a resource to store the last movement input for camera rotation
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct LastInputDirection(pub Vec2);

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut last_input: ResMut<LastInputDirection>,
    mut query: Query<&mut AnimationState>,
    time: Res<Time>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

    // Update forward hold time
    if let Ok(mut animation_state) = query.single_mut() {
        if up && !down {
            animation_state.forward_hold_time += time.delta_secs();
        } else {
            animation_state.forward_hold_time = 0.0;
        }
    }

    if direction != Vector2::ZERO {
        movement_event_writer.write(MovementAction::Move(direction));
        last_input.0 = direction.as_dvec2().as_vec2();
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.write(MovementAction::Jump);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if let (Some(x), Some(y)) = (
            gamepad.get(GamepadAxis::LeftStickX),
            gamepad.get(GamepadAxis::LeftStickY),
        ) {
            movement_event_writer.write(MovementAction::Move(
                Vector2::new(x as Scalar, y as Scalar).clamp_length_max(1.0),
            ));
        }

        if gamepad.just_pressed(GamepadButton::South) {
            movement_event_writer.write(MovementAction::Jump);
        }
    }
}

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
        if let Some((_, hit)) = ray_cast
            .cast_ray(ray, &settings)
            .first()
        {
            let ground_normal = hit.normal;
            let slope_angle = ground_normal.angle_between(Vec3::Y).to_degrees();
            
            // Check if we're on a valid slope
            if slope_angle <= MAX_SLOPE_ANGLE {
                // Ground snapping for small obstacles
                let distance_to_ground = hit.distance;
                if distance_to_ground <= GROUND_SNAP_DISTANCE {
                    // Snap to ground
                    velocity.y = 0.0;
                    commands.entity(entity).insert(Grounded);
                    
                    // Handle stair climbing
                    if distance_to_ground > STAIR_HEIGHT {
                        // Try to climb the stair
                        let forward = transform.forward();
                        let stair_origin = ray_origin + forward * 0.6;
                        let stair_ray = Ray3d::new(stair_origin, ray_direction);
                        
                        // Perform the stair check with same settings
                        if let Some((_, stair_hit)) = ray_cast
                            .cast_ray(stair_ray, &settings)
                            .first()
                        {
                            if stair_hit.distance <= STAIR_HEIGHT {
                                // Smoothly move up the stair
                                velocity.y = (STAIR_HEIGHT / time.delta_secs()) * 1.5;
                            }
                        }
                    } else {
                        commands.entity(entity).remove::<Grounded>();
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
                        let target_rotation = -direction.x * ROTATION_SPEED * delta_time;
                        transform.rotate_y(target_rotation);
                    }

                    // Get movement vectors
                    let forward = transform.forward();
                    let right = transform.right();
                    let movement_direction = (forward * -direction.y) + (right * direction.x);
                    
                    // Calculate target velocity
                    let target_speed = MAX_SPEED * direction.length();
                    let current_speed = Vec2::new(linear_velocity.x, linear_velocity.z).length();
                    
                    // Smooth acceleration/deceleration
                    let acceleration = if target_speed > current_speed {
                        MOVEMENT_ACCELERATION
                    } else {
                        MOVEMENT_DECELERATION
                    };
                    
                    // Apply movement
                    let speed_multiplier = if animation_state.current_animation == 3 { 1.5 } else { 1.0 };
                    let target_velocity = movement_direction * target_speed * speed_multiplier;
                    
                    // Smoothly interpolate current velocity to target velocity
                    linear_velocity.x = linear_velocity.x.lerp(target_velocity.x, acceleration * delta_time);
                    linear_velocity.z = linear_velocity.z.lerp(target_velocity.z, acceleration * delta_time);
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
    mut query: Query<(&MovementDampingFactor, &mut LinearVelocity, Option<&Grounded>)>
) {
    for (_damping_factor, mut linear_velocity, grounded) in &mut query {
        // Apply air resistance when not grounded
        if grounded.is_none() {
            linear_velocity.x *= 0.98;
            linear_velocity.z *= 0.98;
        }
        
        // Apply ground friction
        if grounded.is_some() {
            linear_velocity.x *= 0.92;
            linear_velocity.z *= 0.92;
        }
        
        // Prevent tiny residual movement
        if linear_velocity.x.abs() < 0.01 {
            linear_velocity.x = 0.0;
        }
        if linear_velocity.z.abs() < 0.01 {
            linear_velocity.z = 0.0;
        }
    }
}

/// Update camera_follow_player_system to strictly follow player rotation
fn camera_follow_player_system(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FlyCam>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            let player_pos = player_transform.translation;
            let camera_distance = 18.0;
            let camera_height = 4.0;
            
            // Get player's forward direction
            let player_forward = player_transform.forward();
            
            // Calculate camera position in front of player
            let offset = Vec3::new(
                player_forward.x * camera_distance,
                camera_height,
                player_forward.z * camera_distance,
            );
            
            let target_pos = player_pos + offset;
            
            // Smoothly move camera to new position
            camera_transform.translation = camera_transform.translation.lerp(
                target_pos,
                (5.0 * time.delta_secs()).min(1.0),
            );
            
            // Make camera look at player
            camera_transform.look_at(player_pos, Vec3::Y);
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
    let mut player = animation_players.get_mut(gltf_animations.animation_player).unwrap();
    let animation = gltf_animations.get_by_number(2).unwrap();
    player.stop_all();
    player.play(animation).repeat();
}

impl CharacterControllerBundle {
    pub fn new() -> Self {
        let length = 0.8;
        let radius = 0.3;
        let offset = Vec3::new(0.0, (length / 2.0) + radius, 0.0);
        let capsule = Collider::capsule(radius, length);
        let collider = Collider::compound(vec![(offset, Quat::IDENTITY, capsule)]);
        // Use a small sphere for the ground caster shape
        let caster_shape = Collider::sphere(0.5);
        
        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            ).with_max_distance(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::new(MOVEMENT_ACCELERATION, 0.9, 7.0),
            animation_state: AnimationState { 
                forward_hold_time: 0.0,
                current_animation: 2, // Start with idle animation
            },
        }
    }
}