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
}

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
}

// Global game counter for unique game IDs
#[derive(Copy, Drop, Serde)]
#[dojo::model]
pub struct GameCounter {
    #[key]
    pub counter_id: u32, // Use constant GAME_COUNTER_ID
    pub next_game_id: u32,
}

// Constants for special identifiers
pub const GAME_COUNTER_ID: u32 = 999999999;

// Inventory models
#[derive(Serde, Copy, Drop, Introspect, PartialEq)]
pub enum ItemType {
    HealthPotion,
    SurvivalKit,
    Book,
}

impl ItemTypeIntoFelt252 of Into<ItemType, felt252> {
    fn into(self: ItemType) -> felt252 {
        match self {
            ItemType::HealthPotion => 1,
            ItemType::SurvivalKit => 2,
            ItemType::Book => 3,
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

