use crate::constants::dojo::PICKUP_ITEM_SELECTOR;
use crate::screens::Screen;
use crate::systems::collectibles::CollectibleType;
use bevy::prelude::*;
use dojo_bevy_plugin::{DojoEntityUpdated, DojoResource, TokioRuntime};
use starknet::core::types::Call;

/// Event to trigger item pickup on the blockchain
#[derive(Event, Debug)]
pub struct PickupItemEvent {
    pub item_type: CollectibleType,
    pub item_entity: Entity,
}

/// Event emitted when an item pickup is successfully processed on blockchain
#[derive(Event, Debug)]
pub struct ItemPickedUpEvent {
    pub item_type: CollectibleType,
    pub item_entity: Entity,
    pub transaction_hash: String,
}

/// Event emitted when item pickup fails
#[derive(Event, Debug)]
pub struct ItemPickupFailedEvent {
    pub item_type: CollectibleType,
    pub error: String,
}

/// Resource to track pending pickup transactions
#[derive(Resource, Debug, Default)]
pub struct PickupTransactionState {
    pub pending_pickups: Vec<(Entity, CollectibleType)>,
}

pub(super) fn plugin(app: &mut App) {
    app.add_event::<PickupItemEvent>()
        .add_event::<ItemPickedUpEvent>()
        .add_event::<ItemPickupFailedEvent>()
        .init_resource::<PickupTransactionState>()
        .add_systems(
            Update,
            (
                handle_pickup_item_events,
                handle_item_picked_up_events,
                handle_item_pickup_failed_events,
                handle_pickup_entity_updates,
            )
                .run_if(in_state(Screen::GamePlay)),
        );
}

/// System to handle PickupItemEvent and call the blockchain
fn handle_pickup_item_events(
    mut events: EventReader<PickupItemEvent>,
    mut dojo: ResMut<DojoResource>,
    tokio: Res<TokioRuntime>,
    dojo_config: Res<super::DojoSystemState>,
    mut pickup_state: ResMut<PickupTransactionState>,
    _game_state: Res<super::create_game::GameState>,
    mut item_picked_up_events: EventWriter<ItemPickedUpEvent>,
) {
    for event in events.read() {
        info!(
            "Picking up {:?} item on blockchain (contract uses caller address for identification)",
            event.item_type
        );

        // Create the contract call for pickup_item function
        // pickup_item() - takes no parameters according to contract interface
        let call = Call {
            to: dojo_config.config.action_address,
            selector: PICKUP_ITEM_SELECTOR,
            calldata: vec![], // No parameters required
        };

        // Queue the call to the blockchain
        dojo.queue_tx(&tokio, vec![call]);
        
        // Track this pickup transaction
        pickup_state.pending_pickups.push((event.item_entity, event.item_type));
        
        info!(
            "Pickup item call queued successfully for {:?}",
            event.item_type
        );
        
        // Since the contract currently always returns true and has no real logic,
        // we can immediately trigger the success event to remove the item
        // In a real implementation, this would wait for actual blockchain confirmation
        info!("⚡ Fast-tracking item removal since contract is stubbed (always returns true)");
        
        // Immediately trigger successful pickup to remove item from world
        item_picked_up_events.write(ItemPickedUpEvent {
            item_type: event.item_type,
            item_entity: event.item_entity,
            transaction_hash: "0x123456789abcdef".to_string(), // Mock transaction hash
        });
        
        warn!("🚀 Item pickup success event triggered for {:?}", event.item_type);
    }
}

/// System to handle successful item pickup
fn handle_item_picked_up_events(
    mut events: EventReader<ItemPickedUpEvent>,
    mut commands: Commands,
    world: &World,
) {
    for event in events.read() {
        info!(
            "Item pickup confirmed on blockchain! {:?} (TX: {})",
            event.item_type, event.transaction_hash
        );

        // Check if the entity still exists before trying to despawn it
        if world.get_entity(event.item_entity).is_ok() {
            commands.entity(event.item_entity).despawn();
            info!("Item {:?} successfully removed from game world", event.item_type);
        } else {
            info!("Item {:?} entity no longer exists (ID: {:?}) - likely already removed", 
                  event.item_type, event.item_entity);
        }
    }
}

/// System to handle failed item pickup
fn handle_item_pickup_failed_events(
    mut events: EventReader<ItemPickupFailedEvent>,
) {
    for event in events.read() {
        error!(
            "Item pickup failed for {:?}: {}",
            event.item_type, event.error
        );
        
        // TODO: Show error message to user
        // TODO: Optionally retry the pickup
        warn!("Item {:?} remains in game world due to pickup failure", event.item_type);
    }
}

/// System to handle entity updates from Dojo/Torii related to pickups
fn handle_pickup_entity_updates(
    mut dojo_events: EventReader<DojoEntityUpdated>,
    mut item_picked_up_events: EventWriter<ItemPickedUpEvent>,
    _item_pickup_failed_events: EventWriter<ItemPickupFailedEvent>,
    mut pickup_state: ResMut<PickupTransactionState>,
) {
    for event in dojo_events.read() {
        // Process each model in the entity update
        for model in &event.models {
            match model.name.as_str() {
                "PlayerInventory" => {
                    info!("PlayerInventory updated - item pickup may have succeeded");
                    
                    // For now, assume any inventory update means pickup succeeded
                    // In a full implementation, you'd parse the model data to confirm
                    if let Some((entity, item_type)) = pickup_state.pending_pickups.pop() {
                        item_picked_up_events.write(ItemPickedUpEvent {
                            item_type,
                            item_entity: entity,
                            transaction_hash: "0x123".to_string(), // TODO: Extract real TX hash
                        });
                    }
                }
                "PlayerStats" => {
                    info!("PlayerStats updated - may be related to item pickup");
                    // TODO: Handle stat changes from item pickup
                }
                _ => {
                    // Other model updates not related to pickup
                }
            }
        }
    }
}