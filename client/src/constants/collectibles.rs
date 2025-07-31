/// Maximum number of coins that can be spawned in the game world
pub const MAX_COINS: usize = 200;

/// Maximum number of attempts to place coins before giving up
pub const MAX_COIN_PLACEMENT_ATTEMPTS: usize = 10000;

/// Minimum distance between coins to avoid clustering
pub const MIN_DISTANCE_BETWEEN_COINS: f32 = 4.0;

/// Coin streaming radius around the player
pub const COIN_STREAMING_RADIUS: f32 = 100.0; 