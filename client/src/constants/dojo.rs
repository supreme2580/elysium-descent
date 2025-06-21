use starknet::core::types::Felt;
use starknet::macros::selector;
use std::env;

/// Configuration for Dojo blockchain integration
#[derive(Debug, Clone)]
pub struct DojoConfig {
    pub torii_url: String,
    pub katana_url: String,
    pub world_address: Felt,
    pub action_address: Felt,
    pub use_dev_account: bool,
    pub dev_account_index: u32,
}

impl Default for DojoConfig {
    fn default() -> Self {
        Self {
            torii_url: env::var("TORII_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            katana_url: env::var("KATANA_URL")
                .unwrap_or_else(|_| "http://0.0.0.0:5050".to_string()),
            world_address: env::var("WORLD_ADDRESS")
                .ok()
                .and_then(|addr| Felt::from_hex(&addr).ok())
                .unwrap_or_else(|| {
                    // Real deployed world address from manifest_dev.json
                    Felt::from_hex_unchecked(
                        "0x01d3be0144b9a1d96f8ea55ad581c5a1ab2281837821c6e9c1aa6c37b35b7d5f",
                    )
                }),
            action_address: env::var("ACTION_ADDRESS")
                .ok()
                .and_then(|addr| Felt::from_hex(&addr).ok())
                .unwrap_or_else(|| {
                    // Real deployed action address from manifest_dev.json
                    Felt::from_hex_unchecked(
                        "0x03e808dd3f605913362dd9acf16a9853eb824d1619c182b84199d7424464f670",
                    )
                }),
            use_dev_account: env::var("USE_DEV_ACCOUNT").unwrap_or_else(|_| "true".to_string())
                == "true",
            dev_account_index: env::var("DEV_ACCOUNT_INDEX")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .unwrap_or(0),
        }
    }
}

// Updated selectors for Elysium Descent contract functions
pub const CREATE_GAME_SELECTOR: Felt = selector!("create_game");
pub const PICKUP_ITEM_SELECTOR: Felt = selector!("pickup_item");
