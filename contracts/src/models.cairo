use starknet::ContractAddress;

// Game management models
#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum GameStatus {
    NotStarted,
    InProgress,
    Paused,
    Completed,
}

impl GameStatusIntoFelt252 of Into<GameStatus, felt252> {
    fn into(self: GameStatus) -> felt252 {
        match self {
            GameStatus::NotStarted => 0,
            GameStatus::InProgress => 1,
            GameStatus::Paused => 2,
            GameStatus::Completed => 3,
        }
    }
}

#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum PlayerType {
    Man,
    Woman,
    Beast,
    Spirit,
}

impl PlayerTypeIntoFelt252 of Into<PlayerType, felt252> {
    fn into(self: PlayerType) -> felt252 {
        match self {
            PlayerType::Man => 0,
            PlayerType::Woman => 1,
            PlayerType::Beast => 2,
            PlayerType::Spirit => 3,
        }
    }
}

#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum ObjectiveType {
    Collect,
    ReachLocation,
    Defeat,
    Survive,
    Explore,
}

impl ObjectiveTypeIntoFelt252 of Into<ObjectiveType, felt252> {
    fn into(self: ObjectiveType) -> felt252 {
        match self {
            ObjectiveType::Collect => 0,
            ObjectiveType::ReachLocation => 1,
            ObjectiveType::Defeat => 2,
            ObjectiveType::Survive => 3,
            ObjectiveType::Explore => 4,
        }
    }
}

#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum BeastType {
    Monster,
    Dragon,
    Goblin,
    Orc,
    Demon,
    Undead,
    Elemental,
}

impl BeastTypeIntoFelt252 of Into<BeastType, felt252> {
    fn into(self: BeastType) -> felt252 {
        match self {
            BeastType::Monster => 0,
            BeastType::Dragon => 1,
            BeastType::Goblin => 2,
            BeastType::Orc => 3,
            BeastType::Demon => 4,
            BeastType::Undead => 5,
            BeastType::Elemental => 6,
        }
    }
}

// Player Profile Model
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct PlayerProfile {
    #[key]
    pub player: ContractAddress,
    pub username: felt252,
    pub player_type: PlayerType,
    pub created_at: u64,
    pub last_active: u64,
    pub total_games_played: u32,
    pub total_score: u64,
    pub highest_level_reached: u32,
    pub is_active: bool,
}

// Game Model
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct Game {
    #[key]
    pub game_id: u32,
    pub player: ContractAddress,
    pub status: GameStatus,
    pub current_level: u32,
    pub created_at: u64,
    pub score: u32,
    pub player_type: PlayerType,
}

// Level Model - Core level data
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct Level {
    #[key]
    pub level_id: u32,
    pub level_name: felt252,
    pub player_type: PlayerType,
    pub is_active: bool,
    pub created_at: u64,
    pub modified_at: u64,
    pub created_by: ContractAddress,
    pub next_level: u32,
}

// Level Coins Configuration
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelCoins {
    #[key]
    pub level_id: u32,
    pub spawn_count: u32,
    pub total_collected: u32,
}

// Level Coin Positions
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelCoinPosition {
    #[key]
    pub level_id: u32,
    #[key]
    pub coin_index: u32,
    pub x: felt252,
    pub y: felt252,
    pub z: felt252,
}

// Level Beasts Configuration
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelBeast {
    #[key]
    pub level_id: u32,
    #[key]
    pub beast_id: felt252,
    pub beast_type: BeastType,
    pub x: felt252,
    pub y: felt252,
    pub z: felt252,
    pub health: u32,
    pub damage: u32,
    pub speed: felt252,
    pub is_defeated: bool,
}

// Level Objectives Configuration
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelObjective {
    #[key]
    pub level_id: u32,
    #[key]
    pub objective_id: felt252,
    pub title: felt252,
    pub description: felt252,
    pub objective_type: ObjectiveType,
    pub target: felt252,
    pub required_count: u32,
    pub current_count: u32,
    pub reward: felt252,
    pub is_completed: bool,
}

// Level Objective Positions (for location-based objectives)
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelObjectivePosition {
    #[key]
    pub level_id: u32,
    #[key]
    pub objective_id: felt252,
    pub x: felt252,
    pub y: felt252,
    pub z: felt252,
    pub completion_radius: felt252,
}

// Level Environment Configuration
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelEnvironment {
    #[key]
    pub level_id: u32,
    pub dungeon_scale: felt252,
    pub dungeon_x: felt252,
    pub dungeon_y: felt252,
    pub dungeon_z: felt252,
    pub dungeon_rotation: felt252,
}

// Player Stats Model
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct PlayerStats {
    #[key]
    pub player: ContractAddress,
    pub health: u32,
    pub max_health: u32,
    pub level: u32,
    pub experience: u32,
    pub items_collected: u32,
    pub beasts_defeated: u32,
    pub objectives_completed: u32,
}

// Global game counter for unique game IDs
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct GameCounter {
    #[key]
    pub counter_id: u32, // Use constant GAME_COUNTER_ID
    pub next_game_id: u32,
}

// Global level counter for unique level IDs
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelCounter {
    #[key]
    pub counter_id: u32, // Use constant LEVEL_COUNTER_ID
    pub next_level_id: u32,
}

// Constants for special identifiers
pub const GAME_COUNTER_ID: u32 = 999999999;
pub const LEVEL_COUNTER_ID: u32 = 999999998;

// Inventory models
#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum ItemType {
    HealthPotion,
    SurvivalKit,
    Book,
    Coin,
    BeastEssence,
    AncientKnowledge,
}

impl ItemTypeIntoFelt252 of Into<ItemType, felt252> {
    fn into(self: ItemType) -> felt252 {
        match self {
            ItemType::HealthPotion => 1,
            ItemType::SurvivalKit => 2,
            ItemType::Book => 3,
            ItemType::Coin => 4,
            ItemType::BeastEssence => 5,
            ItemType::AncientKnowledge => 6,
        }
    }
}

#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct PlayerInventory {
    #[key]
    pub player: ContractAddress,
    pub health_potions: u32,
    pub survival_kits: u32,
    pub books: u32,
    pub coins: u32,
    pub beast_essences: u32,
    pub ancient_knowledge: u32,
    pub capacity: u32,
}

// Level items spawned per level
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct LevelItems {
    #[key]
    pub game_id: u32,
    #[key]
    pub level: u32,
    pub total_health_potions: u32,
    pub total_survival_kits: u32,
    pub total_books: u32,
    pub collected_health_potions: u32,
    pub collected_survival_kits: u32,
    pub collected_books: u32,
}

// Individual item instances in the world
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct WorldItem {
    #[key]
    pub game_id: u32,
    #[key]
    pub item_id: u32,
    pub item_type: ItemType,
    pub x_position: u32,
    pub y_position: u32,
    pub is_collected: bool,
    pub level: u32,
}

// Game Progress Tracking
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct GameProgress {
    #[key]
    pub game_id: u32,
    #[key]
    pub level: u32,
    pub coins_collected: u32,
    pub beasts_defeated: u32,
    pub objectives_completed: u32,
    pub level_started_at: u64,
    pub level_completed_at: u64,
    pub is_level_completed: bool,
}

// Admin Management
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct Admin {
    #[key]
    pub admin_address: ContractAddress,
    pub role: felt252, // "owner", "moderator", etc.
    pub permissions: u32, // Bit flags for different permissions
    pub added_at: u64,
}

