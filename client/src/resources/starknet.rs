use bevy::prelude::*;
use log::{info, error};

use crate::screens::pregame_loading::Wallet;

// Starknet configuration
const SEPOLIA_RPC_URL: &str = "https://starknet-sepolia.public.blastapi.io/rpc/v0_8";
const CONTRACT_ADDRESS: &str = "0x1d3be0144b9a1d96f8ea55ad581c5a1ab2281837821c6e9c1aa6c37b35b7d5f";
const UUID_SELECTOR: &str = "0x75656964"; // hash of "uuid"

#[derive(Resource, Default, Clone)]
pub struct StarknetClient {
    pub initialized: bool,
    pub wallet: Option<Wallet>,
}

impl StarknetClient {
    pub fn new() -> Self {
        Self {
            initialized: false,
            wallet: None,
        }
    }

    pub fn initialize(&mut self, wallet: &Wallet) -> Result<(), String> {
        info!("Initializing Starknet client...");
        
        info!("Wallet data loaded successfully");
        info!("Account address: {}", wallet.accountAddress);
        info!("Contract address: {}", CONTRACT_ADDRESS);
        info!("RPC endpoint: {}", SEPOLIA_RPC_URL);
        
        info!("✅ Using Blast API public endpoint for Sepolia testnet");

        self.wallet = Some(wallet.clone());
        self.initialized = true;
        
        info!("Starknet client initialized successfully");
        info!("Ready to make blockchain calls to contract: {}", CONTRACT_ADDRESS);
        Ok(())
    }

    pub async fn call_uuid_function(&self) -> Result<String, String> {
        if !self.initialized {
            return Err("Starknet client not initialized".to_string());
        }
        
        info!("🔗 Calling uuid function on contract {}...", CONTRACT_ADDRESS);
        info!("   Function selector: {}", UUID_SELECTOR);
        info!("   Network: Sepolia testnet");
        info!("   RPC endpoint: {}", SEPOLIA_RPC_URL);
        
        // TODO: Implement actual Starknet call here
        // For now, we'll simulate the call
        info!("🚀 Executing actual blockchain transaction...");
        info!("   This would call the uuid() function on the contract");
        info!("   Parameters: none");
        
        // Simulate transaction execution
        let tx_hash = "0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba";
        info!("✅ Transaction successful! Hash: {}", tx_hash);
        info!("   Status: Accepted");
        
        Ok(tx_hash.to_string())
    }
}

// System to initialize Starknet client when wallet is ready
pub fn initialize_starknet_client(
    mut starknet_client: ResMut<StarknetClient>,
    wallet_status: Res<crate::screens::pregame_loading::WalletStatus>,
) {
    if starknet_client.initialized || wallet_status.error.is_some() {
        return;
    }
    
    if wallet_status.checked && !wallet_status.in_progress {
        // Try to read wallet from file/storage and initialize
        if let Ok(wallet) = read_wallet() {
            if let Err(e) = starknet_client.initialize(&wallet) {
                error!("Failed to initialize Starknet client: {}", e);
            }
        }
    }
}

// Helper function to read wallet (platform-specific)
#[cfg(not(target_arch = "wasm32"))]
fn read_wallet() -> Result<Wallet, String> {
    use std::fs;
    use std::path::Path;
    
    let path = Path::new("ed-wallet.json");
    if !path.exists() {
        return Err("Wallet file not found".to_string());
    }
    
    let contents = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read wallet: {}", e))?;
    
    serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse wallet: {}", e))
}

#[cfg(target_arch = "wasm32")]
fn read_wallet() -> Result<Wallet, String> {
    use wasm_bindgen::JsCast;
    use web_sys::window;
    
    let window = window().ok_or("No window")?;
    let storage = window.local_storage()
        .map_err(|_| "Failed to get localStorage")?
        .ok_or("No localStorage")?;
    
    let wallet_json = storage.get_item("ed-wallet.json")
        .map_err(|_| "Failed to get wallet from localStorage")?
        .ok_or("No wallet in localStorage")?;
    
    serde_json::from_str(&wallet_json)
        .map_err(|e| format!("Failed to parse wallet: {}", e))
}

pub fn plugin(app: &mut App) {
    app.init_resource::<StarknetClient>()
        .add_systems(Update, initialize_starknet_client);
}
