pub struct CharacterMovementConfig;

impl CharacterMovementConfig {
    pub const MAX_SLOPE_ANGLE: f32 = 45.0;

    // Improved stair climbing constants
    pub const MAX_STAIR_HEIGHT: f32 = 0.5; // Maximum step the character can climb
    pub const STAIR_DETECTION_DISTANCE: f32 = 0.4; // How far forward to check for stairs

    pub const GROUND_SNAP_DISTANCE: f32 = 0.2;
    pub const MOVEMENT_ACCELERATION: f32 = 30.0;
    pub const MOVEMENT_DECELERATION: f32 = 40.0;
    pub const MAX_SPEED: f32 = 5.0;
    pub const MAX_RUN_SPEED: f32 = 13.5; // Maximum speed when running/sprinting
    pub const ROTATION_SPEED: f32 = 5.0;

    // Air and ground friction constants
    pub const AIR_RESISTANCE: f32 = 0.98;
    pub const GROUND_FRICTION: f32 = 0.92;

    // Movement threshold for stopping tiny residual movement
    pub const MIN_MOVEMENT_THRESHOLD: f32 = 0.01;
}
