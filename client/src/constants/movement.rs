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

    pub const RUN_TRIGGER_HOLD_TIME: f32 = 3.0;
}

pub struct CharacterAnimationConfig;

impl CharacterAnimationConfig {
    pub const IDLE: usize = 1;
    pub const RUNNING: usize = 3;
    pub const WALKING: usize = 4;
    pub const FIGHT_MOVE_1: usize = 5;
    pub const FIGHT_MOVE_2: usize = 6;
}
