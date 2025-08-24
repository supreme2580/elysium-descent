use bevy::prelude::*;
use log::{info, error, warn};

use crate::screens::pregame_loading::Wallet;

// Starknet configuration
const SEPOLIA_RPC_URL: &str = "https://starknet-sepolia.public.blastapi.io/rpc/v0_8";
const CONTRACT_ADDRESS: &str = "0x1d3be0144b9a1d96f8ea55ad581c5a1ab2281837821c6e9c1aa6c37b35b7d5f";
const UUID_SELECTOR: &str = "0x2ee0e84a99e5b3cb20adcdbe548dd1ab3bb535bdd690595e43594707778be82"; // actual uuid selector from Voyager

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
        
        self.wallet = Some(wallet.clone());
        self.initialized = true;
        
        info!("✅ Starknet HTTP client initialized successfully");
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
        
        // Retry logic with exponential backoff
        let max_retries = 3;
        let mut attempt = 0;
        let mut last_error = None;
        
        while attempt < max_retries {
            attempt += 1;
            info!("Attempt {} of {} for contract call...", attempt, max_retries);
            
            let result = {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.call_uuid_native().await
                }
                
                #[cfg(target_arch = "wasm32")]
                {
                    self.call_uuid_wasm().await
                }
            };
            
            match result {
                Ok(value) => {
                    info!("✅ Contract call successful on attempt {}", attempt);
                    return Ok(value);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    warn!("Attempt {} failed: {}", attempt, e);
                    
                    if attempt < max_retries {
                        let delay_ms = (2_u64.pow(attempt as u32)) * 1000; // 2, 4, 8 seconds
                        info!("Retrying in {}ms...", delay_ms);
                        
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            // Use std::thread::sleep since we're not in a Tokio context
                            let delay_duration = std::time::Duration::from_millis(delay_ms);
                            let (tx, rx) = std::sync::mpsc::channel();
                            std::thread::spawn(move || {
                                std::thread::sleep(delay_duration);
                                let _ = tx.send(());
                            });
                            let _ = rx.recv();
                        }
                        
                        #[cfg(target_arch = "wasm32")]
                        {
                            // WASM sleep using timeout
                            use wasm_bindgen_futures::JsFuture;
                            use wasm_bindgen::closure::Closure;
                            use wasm_bindgen::JsCast;
                            use web_sys::{window};
                            
                            if let Some(window) = window() {
                                let promise = js_sys::Promise::new(&mut |resolve, _| {
                                    let closure = Closure::once_into_js(move || {
                                        resolve.call0(&wasm_bindgen::JsValue::undefined()).unwrap();
                                    });
                                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                        closure.as_ref().unchecked_ref(), delay_ms as i32
                                    ).unwrap();
                                });
                                let _ = JsFuture::from(promise).await;
                            }
                        }
                    }
                }
            }
        }
        
        // All retries failed
        let error_msg = format!(
            "All {} attempts failed. Last error: {}",
            max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        );
        error!("❌ {}", error_msg);
        Err(error_msg)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    async fn call_uuid_native(&self) -> Result<String, String> {
        info!("🚀 Executing actual blockchain call (native)...");
        
        // Run in thread pool without Tokio async context
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            let result = Self::blocking_contract_call();
            let _ = tx.send(result);
        });
        
        // Wait for result with timeout
        match rx.recv_timeout(std::time::Duration::from_secs(30)) {
            Ok(result) => result,
            Err(_) => Err("Contract call timeout".to_string()),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn blocking_contract_call() -> Result<String, String> {
        // Prepare JSON-RPC call using blocking reqwest
        let call_data = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "starknet_call",
            "params": {
                "request": {
                    "contract_address": CONTRACT_ADDRESS,
                    "entry_point_selector": UUID_SELECTOR,
                    "calldata": []
                },
                "block_id": "latest"
            },
            "id": 1
        });
        
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(SEPOLIA_RPC_URL)
            .header("Content-Type", "application/json")
            .json(&call_data)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        
        let response_obj: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        if let Some(error) = response_obj.get("error") {
            error!("❌ Contract call failed: {:?}", error);
            return Err(format!("RPC error: {:?}", error));
        }
        
        if let Some(result) = response_obj.get("result") {
            info!("✅ Contract call successful!");
            
            // Extract UUID value from result array
            let uuid_value = if let Some(values) = result.as_array() {
                if !values.is_empty() {
                    values[0].as_str().unwrap_or("Unknown").to_string()
                } else {
                    "No return value".to_string()
                }
            } else {
                "Invalid result format".to_string()
            };
            
            info!("   📝 Transaction Hash: {}", uuid_value);
            
            Ok(uuid_value)
        } else {
            Err("No result in response".to_string())
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_latest_block_info_blocking() {
        // Get latest block information for transaction context
        std::thread::spawn(|| {
            let block_data = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "starknet_getBlockWithTxHashes",
                "params": ["latest"],
                "id": 2
            });
            
            let client = reqwest::blocking::Client::new();
            if let Ok(response) = client
                .post(SEPOLIA_RPC_URL)
                .header("Content-Type", "application/json")
                .json(&block_data)
                .send()
            {
                if let Ok(block_obj) = response.json::<serde_json::Value>() {
                    if let Some(result) = block_obj.get("result") {
                        if let Some(block_number) = result.get("block_number") {
                            info!("   📦 Latest Block Number: {}", block_number);
                        }
                        if let Some(block_hash) = result.get("block_hash") {
                            info!("   🔒 Block Hash: {}", block_hash.as_str().unwrap_or("Unknown"));
                        }
                        if let Some(timestamp) = result.get("timestamp") {
                            info!("   ⏰ Block Timestamp: {}", timestamp);
                        }
                        if let Some(transactions) = result.get("transactions") {
                            if let Some(tx_array) = transactions.as_array() {
                                info!("   🔄 Transactions in Block: {}", tx_array.len());
                                if !tx_array.is_empty() {
                                    info!("   📝 Recent Transaction Hashes:");
                                    for (i, tx) in tx_array.iter().take(3).enumerate() {
                                        if let Some(tx_hash) = tx.as_str() {
                                            info!("      {}. {}", i + 1, tx_hash);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }
    
    #[cfg(target_arch = "wasm32")]
    async fn call_uuid_wasm(&self) -> Result<String, String> {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{window, RequestInit, Request, Response};
        
        info!("🚀 Executing actual blockchain call (WASM)...");
        
        let window = window().ok_or("No window object")?;
        
        // Prepare JSON-RPC call
        let call_data = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "starknet_call",
            "params": {
                "request": {
                    "contract_address": CONTRACT_ADDRESS,
                    "entry_point_selector": UUID_SELECTOR,
                    "calldata": []
                },
                "block_id": "latest"
            },
            "id": 1
        });
        
        let mut opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&JsValue::from_str(&call_data.to_string()));
        
        let request = Request::new_with_str_and_init(SEPOLIA_RPC_URL, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;
        
        request.headers().set("Content-Type", "application/json")
            .map_err(|e| format!("Failed to set headers: {:?}", e))?;
        
        let response_promise = window.fetch_with_request(&request);
        let response_js = JsFuture::from(response_promise).await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;
        
        let response: Response = response_js.dyn_into()
            .map_err(|_| "Failed to cast response")?;
        
        if !response.ok() {
            return Err(format!("HTTP error: {}", response.status()));
        }
        
        let json_promise = response.json()
            .map_err(|e| format!("Failed to get JSON: {:?}", e))?;
        let json_js = JsFuture::from(json_promise).await
            .map_err(|e| format!("JSON parsing failed: {:?}", e))?;
        
        // Parse the JSON response
        let response_obj: serde_json::Value = serde_wasm_bindgen::from_value(json_js)
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        if let Some(error) = response_obj.get("error") {
            error!("❌ Contract call failed: {:?}", error);
            return Err(format!("RPC error: {:?}", error));
        }
        
        if let Some(result) = response_obj.get("result") {
            info!("✅ Contract call successful!");
            
            // Extract UUID value from result array
            let uuid_value = if let Some(values) = result.as_array() {
                if !values.is_empty() {
                    values[0].as_str().unwrap_or("Unknown").to_string()
                } else {
                    "No return value".to_string()
                }
            } else {
                "Invalid result format".to_string()
            };
            
            info!("   📝 Transaction Hash: {}", uuid_value);
            
            Ok(uuid_value)
        } else {
            Err("No result in response".to_string())
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    fn get_latest_block_info_wasm() {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{window, RequestInit, Request, Response};
        
        wasm_bindgen_futures::spawn_local(async {
            if let Ok(window) = window().ok_or("No window") {
                let block_data = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "starknet_getBlockWithTxHashes",
                    "params": ["latest"],
                    "id": 2
                });
                
                let mut opts = RequestInit::new();
                opts.set_method("POST");
                opts.set_body(&JsValue::from_str(&block_data.to_string()));
                
                if let Ok(request) = Request::new_with_str_and_init(SEPOLIA_RPC_URL, &opts) {
                    let _ = request.headers().set("Content-Type", "application/json");
                    
                    if let Ok(response_js) = JsFuture::from(window.fetch_with_request(&request)).await {
                        if let Ok(response) = response_js.dyn_into::<Response>() {
                            if let Ok(json_js) = JsFuture::from(response.json().unwrap()).await {
                                if let Ok(block_obj) = serde_wasm_bindgen::from_value::<serde_json::Value>(json_js) {
                                    if let Some(result) = block_obj.get("result") {
                                        if let Some(block_number) = result.get("block_number") {
                                            info!("   📦 Latest Block Number: {}", block_number);
                                        }
                                        if let Some(block_hash) = result.get("block_hash") {
                                            info!("   🔒 Block Hash: {}", block_hash.as_str().unwrap_or("Unknown"));
                                        }
                                        if let Some(timestamp) = result.get("timestamp") {
                                            info!("   ⏰ Block Timestamp: {}", timestamp);
                                        }
                                        if let Some(transactions) = result.get("transactions") {
                                            if let Some(tx_array) = transactions.as_array() {
                                                info!("   🔄 Transactions in Block: {}", tx_array.len());
                                                if !tx_array.is_empty() {
                                                    info!("   📝 Recent Transaction Hashes:");
                                                    for (i, tx) in tx_array.iter().take(3).enumerate() {
                                                        if let Some(tx_hash) = tx.as_str() {
                                                            info!("      {}. {}", i + 1, tx_hash);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
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
    use web_sys::window;
    
    let window = window().ok_or("No window object available")?;
    let local_storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;
    
    let wallet_json = local_storage
        .get_item("ed-wallet")
        .map_err(|_| "Failed to read from localStorage")?;
    
    match wallet_json {
        Some(json) => {
            info!("Reading wallet from localStorage");
            serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse wallet from localStorage: {}", e))
        }
        None => {
            info!("No wallet found in localStorage");
            Err("Wallet not found in localStorage".to_string())
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn save_wallet(wallet: &Wallet) -> Result<(), String> {
    use web_sys::window;
    
    let window = window().ok_or("No window object available")?;
    let local_storage = window
        .local_storage()
        .map_err(|_| "Failed to access localStorage")?
        .ok_or("localStorage not available")?;
    
    let wallet_json = serde_json::to_string(wallet)
        .map_err(|e| format!("Failed to serialize wallet: {}", e))?;
    
    local_storage
        .set_item("ed-wallet", &wallet_json)
        .map_err(|_| "Failed to save wallet to localStorage")?;
    
    info!("Wallet saved to localStorage successfully");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn save_wallet(wallet: &Wallet) -> Result<(), String> {
    use std::fs;
    
    let wallet_json = serde_json::to_string(wallet)
        .map_err(|e| format!("Failed to serialize wallet: {}", e))?;
    
    fs::write("ed-wallet.json", wallet_json)
        .map_err(|e| format!("Failed to write wallet file: {}", e))?;
    
    info!("Wallet saved to ed-wallet.json successfully");
    Ok(())
}

pub fn plugin(app: &mut App) {
    app.init_resource::<StarknetClient>()
        .add_systems(Update, initialize_starknet_client);
}
