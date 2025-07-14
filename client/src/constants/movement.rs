pub struct CharacterMovementConfig;

impl CharacterMovementConfig {
    pub const GROUND_SNAP_DISTANCE: f32 = 0.2;
    pub const MOVEMENT_ACCELERATION: f32 = 30.0;
    pub const MOVEMENT_DECELERATION: f32 = 40.0;
    pub const MAX_SPEED: f32 = 5.0;
    pub const MAX_RUN_SPEED: f32 = 13.5; // Maximum speed when running/sprinting
    pub const ROTATION_SPEED: f32 = 5.0;

    // Air resistance constant
    pub const AIR_RESISTANCE: f32 = 0.98;

    // Movement threshold for stopping tiny residual movement
    pub const MIN_MOVEMENT_THRESHOLD: f32 = 0.01;
}
