use crate::constants::dojo::CREATE_GAME_SELECTOR;
use crate::screens::Screen;
use bevy::prelude::*;
use dojo_bevy_plugin::{DojoEntityUpdated, DojoResource, TokioRuntime};
use starknet::core::types::Call;

/// Event to trigger game creation on the blockchain
#[derive(Event, Debug)]
pub struct CreateGameEvent;

/// Event emitted when a game is successfully created
#[derive(Event, Debug)]
pub struct GameCreatedEvent {
    pub game_id: u32,
    pub player_address: String,
}

/// Event emitted when game creation fails
#[derive(Event, Debug)]
pub struct GameCreationFailedEvent {
    pub error: String,
}

/// Resource to track the current game state
#[derive(Resource, Debug, Default)]
pub struct GameState {
    pub current_game_id: Option<u32>,
    pub is_creating_game: bool,
    pub player_address: Option<String>,
    pub subscribed_to_entities: bool,
}

/// Represents a Game entity from the blockchain
#[derive(Debug, Clone)]
pub struct GameEntity {
    pub game_id: u32,
    pub player: String,
    pub status: u32, // GameStatus enum value
    pub current_level: u32,
}

/// Event emitted when game entity data is received from Torii
#[derive(Event, Debug)]
pub struct GameDataReceivedEvent {
    pub game: GameEntity,
}

pub(super) fn plugin(app: &mut App) {
    app.add_event::<CreateGameEvent>()
        .add_event::<GameCreatedEvent>()
        .add_event::<GameCreationFailedEvent>()
        .add_event::<GameDataReceivedEvent>()
        .init_resource::<GameState>()
        .add_systems(OnEnter(Screen::GamePlay), auto_create_game_system)
        .add_systems(
            Update,
            (
                handle_create_game_events,
                handle_game_created_events,
                handle_game_creation_failed_events,
                subscribe_to_game_entities,
                handle_dojo_entity_updates,
                fetch_game_data_after_creation,
            )
                .run_if(in_state(Screen::GamePlay)),
        );
}

/// System to automatically create a game when entering gameplay (if not already created)
fn auto_create_game_system(
    mut create_game_events: EventWriter<CreateGameEvent>,
    game_state: Res<GameState>,
) {
    if game_state.current_game_id.is_none() && !game_state.is_creating_game {
        info!("Auto-creating game for new gameplay session");
        create_game_events.write(CreateGameEvent);
    }
}

/// System to handle CreateGameEvent and call the blockchain
fn handle_create_game_events(
    mut events: EventReader<CreateGameEvent>,
    mut dojo: ResMut<DojoResource>,
    tokio: Res<TokioRuntime>,
    dojo_config: Res<super::DojoSystemState>,
    mut game_state: ResMut<GameState>,
) {
    for _event in events.read() {
        if game_state.is_creating_game {
            warn!("Game creation already in progress, ignoring duplicate request");
            continue;
        }

        if game_state.current_game_id.is_some() {
            warn!(
                "Game already exists with ID {:?}",
                game_state.current_game_id
            );
            continue;
        }

        info!("Creating new game on blockchain...");
        game_state.is_creating_game = true;

        // Create the contract call for create_game function
        let call = Call {
            to: dojo_config.config.action_address,
            selector: CREATE_GAME_SELECTOR,
            calldata: vec![], // create_game takes no parameters
        };

        // Queue the call to the blockchain
        dojo.queue_tx(&tokio, vec![call]);
        info!("Game creation call queued successfully");

        // Subscribe to entity updates to listen for the created game
        if !game_state.subscribed_to_entities {
            // Subscribe to Game model updates - using empty string as entity ID for all entities
            dojo.subscribe_entities(&tokio, "".to_string(), None);
            game_state.subscribed_to_entities = true;
            info!("Subscribed to entity updates from Torii");
        }
    }
}

/// System to handle successful game creation
fn handle_game_created_events(
    mut events: EventReader<GameCreatedEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in events.read() {
        info!(
            "Game created successfully! Game ID: {}, Player: {}",
            event.game_id, event.player_address
        );

        game_state.current_game_id = Some(event.game_id);
        game_state.player_address = Some(event.player_address.clone());
        game_state.is_creating_game = false;

        // TODO: You could trigger UI updates here, or initialize level 1
        info!("Game state updated - ready to start playing!");
    }
}

/// System to handle failed game creation
fn handle_game_creation_failed_events(
    mut events: EventReader<GameCreationFailedEvent>,
    mut game_state: ResMut<GameState>,
) {
    for event in events.read() {
        error!("Game creation failed: {}", event.error);
        game_state.is_creating_game = false;

        // TODO: Show error message to user
        // TODO: Optionally retry after a delay
    }
}

/// System to subscribe to game entities when needed
fn subscribe_to_game_entities(
    mut dojo: ResMut<DojoResource>,
    tokio: Res<TokioRuntime>,
    mut game_state: ResMut<GameState>,
) {
    // Subscribe to entity updates when we have a player address but haven't subscribed yet
    if game_state.player_address.is_some() && !game_state.subscribed_to_entities {
        // Subscribe to all entity updates - using empty string as entity ID
        dojo.subscribe_entities(&tokio, "".to_string(), None);
        game_state.subscribed_to_entities = true;
        info!("Subscribed to Game, PlayerStats, and PlayerInventory entity updates");
    }
}

/// System to handle entity updates from Dojo/Torii
fn handle_dojo_entity_updates(
    mut dojo_events: EventReader<DojoEntityUpdated>,
    mut game_created_events: EventWriter<GameCreatedEvent>,
    mut game_data_events: EventWriter<GameDataReceivedEvent>,
    game_state: Res<GameState>,
) {
    for event in dojo_events.read() {
        info!(
            "Received Dojo entity update for entity_id: {:?}",
            event.entity_id
        );

        // Process each model in the entity update
        for model in &event.models {
            info!("Processing model: {:?}", model.name);

            match model.name.as_str() {
                "Game" => {
                    info!("Game model updated - creating placeholder game entity");

                    // For now, create a placeholder game entity since detailed parsing requires more setup
                    let game_entity = GameEntity {
                        game_id: 1,                  // TODO: Extract from model data
                        player: "0x123".to_string(), // TODO: Extract from model data
                        status: 0,
                        current_level: 1,
                    };

                    // If we're creating a game, emit GameCreatedEvent
                    if game_state.is_creating_game {
                        game_created_events.write(GameCreatedEvent {
                            game_id: game_entity.game_id,
                            player_address: game_entity.player.clone(),
                        });
                    }

                    // Always emit game data received event for UI updates
                    game_data_events.write(GameDataReceivedEvent { game: game_entity });
                }
                "PlayerStats" => {
                    info!("Received PlayerStats update");
                    // TODO: Handle player stats updates
                }
                "PlayerInventory" => {
                    info!("Received PlayerInventory update");
                    // TODO: Handle inventory updates
                }
                _ => {
                    info!("Received update for unknown model: {}", model.name);
                }
            }
        }
    }
}

/// System to fetch game data after successful creation
fn fetch_game_data_after_creation(
    mut game_data_events: EventReader<GameDataReceivedEvent>,
    game_state: Res<GameState>,
) {
    for event in game_data_events.read() {
        info!(
            "Processing game data: Game ID {}, Status {}, Level {}",
            event.game.game_id, event.game.status, event.game.current_level
        );

        // Update our local game state with the blockchain data
        if game_state.current_game_id == Some(event.game.game_id) {
            info!("Game data synchronized with blockchain state");
            // TODO: Update UI elements, initialize level if needed

            // If game status is 0 (NotStarted), we might want to automatically start level 1
            if event.game.status == 0 && event.game.current_level == 0 {
                info!("Game is ready to start - could trigger level initialization");
            }
        }
    }
}
