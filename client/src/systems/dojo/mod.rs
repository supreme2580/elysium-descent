use crate::constants::dojo::DojoConfig;
use bevy::prelude::*;
use dojo_bevy_plugin::{DojoResource, TokioRuntime};

pub mod create_game;
pub mod pickup_item;

pub use create_game::CreateGameEvent;

/// Resource to track overall Dojo system state
#[derive(Resource, Debug, Default)]
pub struct DojoSystemState {
    #[allow(dead_code)]
    pub torii_connected: bool,
    #[allow(dead_code)]
    pub account_connected: bool,
    #[allow(dead_code)]
    pub last_error: Option<String>,
    #[allow(dead_code)]
    pub config: DojoConfig,
}

#[allow(dead_code)]
pub fn plugin(app: &mut App) {
    app.init_resource::<DojoSystemState>()
        .add_systems(Startup, (setup_dojo_config, handle_dojo_setup).chain())
        .add_systems(
            Update,
            log_dojo_status.run_if(resource_changed::<DojoSystemState>),
        )
        .add_plugins((create_game::plugin, pickup_item::plugin));
}

#[allow(dead_code)]
fn setup_dojo_config(mut dojo_state: ResMut<DojoSystemState>) {
    dojo_state.config = DojoConfig::default();
    info!("Dojo configuration loaded: {:?}", dojo_state.config);

    // Warn about development account usage
    if dojo_state.config.use_dev_account {
        warn!("Using development account - NOT SUITABLE FOR PRODUCTION");
        warn!("Set USE_DEV_ACCOUNT=false for production deployment");
    }
}

#[allow(dead_code)]
fn handle_dojo_setup(
    tokio: Res<TokioRuntime>,
    mut dojo: ResMut<DojoResource>,
    mut dojo_state: ResMut<DojoSystemState>,
) {
    let config = dojo_state.config.clone();

    info!("Attempting to connect to Dojo services...");

    // Attempt Torii connection
    info!("Connecting to Torii at {}...", config.torii_url);
    dojo.connect_torii(&tokio, config.torii_url.clone(), config.world_address);
    // Note: Connection success is assumed for now as the dojo plugin doesn't provide error feedback
    info!("Torii connection initiated successfully");
    dojo_state.torii_connected = true;

    // Attempt account connection
    if config.use_dev_account {
        info!(
            "Connecting to Katana account #{} at {}...",
            config.dev_account_index, config.katana_url
        );
        dojo.connect_predeployed_account(
            &tokio,
            config.katana_url.clone(),
            config.dev_account_index as usize,
        );
        // Note: Connection success is assumed for now as the dojo plugin doesn't provide error feedback
        info!("Katana account connection initiated successfully");
        dojo_state.account_connected = true;
    } else {
        info!("Development account disabled - manual account connection required");
    }

    // Log overall connection status
    if dojo_state.torii_connected && (dojo_state.account_connected || !config.use_dev_account) {
        info!("Dojo blockchain integration initialized successfully");
        info!("🎮 Press 'G' to create a new game on the blockchain!");
    } else {
        warn!(
            "Dojo blockchain integration has connection issues - game may have limited functionality"
        );
    }
}

/// System to log Dojo status changes for user feedback
#[allow(dead_code)]
fn log_dojo_status(dojo_state: Res<DojoSystemState>) {
    if let Some(error) = &dojo_state.last_error {
        error!("❌ Dojo Error: {}", error);
    }

    let connection_status = match (dojo_state.torii_connected, dojo_state.account_connected) {
        (true, true) => "✅ Fully Connected - Ready for blockchain interactions",
        (true, false) => "⚠️ Partially Connected - Torii only (manual account required)",
        (false, true) => "⚠️ Partially Connected - Account only (Torii connection failed)",
        (false, false) => "❌ Disconnected - No blockchain functionality available",
    };

    info!("🔗 Dojo Status: {}", connection_status);
}
