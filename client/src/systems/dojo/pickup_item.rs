use crate::constants::dojo::PICKUP_ITEM_SELECTOR;
use crate::screens::Screen;
use crate::systems::collectibles::CollectibleType;
use bevy::prelude::*;
use dojo_bevy_plugin::TokioRuntime;
use dojo_bevy_plugin::{DojoEntityUpdated, DojoResource};
use futures::FutureExt;
use starknet::accounts::Account;
use starknet::core::types::Call;
use tokio::task::JoinHandle;

/// Event to trigger item pickup on the blockchain
#[derive(Event, Debug)]
#[allow(dead_code)]
pub struct PickupItemEvent {
    #[allow(dead_code)]
    pub item_type: CollectibleType,
    #[allow(dead_code)]
    pub item_entity: Entity,
}

/// Event emitted when an item pickup is successfully processed on blockchain
#[derive(Event, Debug)]
#[allow(dead_code)]
pub struct ItemPickedUpEvent {
    #[allow(dead_code)]
    pub item_type: CollectibleType,
    #[allow(dead_code)]
    pub transaction_hash: String,
}

/// Event emitted when item pickup fails
#[derive(Event, Debug)]
#[allow(dead_code)]
pub struct ItemPickupFailedEvent {
    #[allow(dead_code)]
    pub item_type: CollectibleType,
    #[allow(dead_code)]
    pub error: String,
}

/// Resource to track pending pickup transactions
#[derive(Resource, Debug, Default)]
pub struct PickupTransactionState {
    #[allow(dead_code)]
    pub pending_pickups: Vec<(Entity, CollectibleType)>,
}

#[derive(Resource, Default)]
pub struct PendingPickupTasks(
    #[allow(dead_code)]
    pub  Vec<
        JoinHandle<Result<(Entity, CollectibleType, String), (Entity, CollectibleType, String)>>,
    >,
);

#[allow(dead_code)]
pub(super) fn plugin(app: &mut App) {
    app.add_event::<PickupItemEvent>()
        .add_event::<ItemPickedUpEvent>()
        .add_event::<ItemPickupFailedEvent>()
        .init_resource::<PickupTransactionState>()
        .init_resource::<PendingPickupTasks>()
        .add_systems(
            Update,
            (
                handle_pickup_item_events,
                poll_pickup_tasks,
                handle_item_picked_up_events,
                handle_item_pickup_failed_events,
                handle_pickup_entity_updates,
            )
                .run_if(in_state(Screen::GamePlay)),
        );
}

/// System to handle PickupItemEvent and call the blockchain
#[allow(dead_code)]
fn handle_pickup_item_events(
    mut events: EventReader<PickupItemEvent>,
    dojo: Res<DojoResource>,
    dojo_config: Res<super::DojoSystemState>,
    tokio: Res<TokioRuntime>,
    mut pending_tasks: ResMut<PendingPickupTasks>,
) {
    let account = dojo.sn.account.clone();
    for event in events.read() {
        let call = Call {
            to: dojo_config.config.action_address,
            selector: PICKUP_ITEM_SELECTOR,
            calldata: vec![],
        };
        let entity = event.item_entity;
        let item_type = event.item_type;
        let account = account.clone();
        let handle = tokio.runtime.spawn(async move {
            if let Some(account) = account {
                let tx = account.execute_v3(vec![call]);
                match tx.send().await {
                    Ok(result) => {
                        Ok((entity, item_type, format!("{:#x}", result.transaction_hash)))
                    }
                    Err(e) => Err((entity, item_type, format!("{:?}", e))),
                }
            } else {
                Err((entity, item_type, "No account available".to_string()))
            }
        });
        pending_tasks.0.push(handle);
    }
}

/// System to handle successful item pickup
#[allow(dead_code)]
fn handle_item_picked_up_events(
    mut events: EventReader<ItemPickedUpEvent>,
    // mut commands: Commands, // No longer needed for despawn
    _world: &World,
) {
    for event in events.read() {
        info!(
            "Item pickup confirmed on blockchain! {:?} (TX: {})",
            event.item_type, event.transaction_hash
        );
        // Entity is already despawned immediately on pickup
    }
}

/// System to handle failed item pickup
#[allow(dead_code)]
fn handle_item_pickup_failed_events(mut events: EventReader<ItemPickupFailedEvent>) {
    for event in events.read() {
        error!(
            "Item pickup failed for {:?}: {}",
            event.item_type, event.error
        );

        // TODO: Show error message to user
        // TODO: Optionally retry the pickup
        warn!(
            "Item {:?} remains in game world due to pickup failure",
            event.item_type
        );
    }
}

/// System to handle entity updates from Dojo/Torii related to pickups
#[allow(dead_code)]
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
                        let _entity = entity;
                        item_picked_up_events.write(ItemPickedUpEvent {
                            item_type,
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

// Poll background tasks and emit events when done
#[allow(dead_code)]
fn poll_pickup_tasks(
    mut pending_tasks: ResMut<PendingPickupTasks>,
    mut item_picked_up_events: EventWriter<ItemPickedUpEvent>,
    mut item_pickup_failed_events: EventWriter<ItemPickupFailedEvent>,
) {
    pending_tasks.0.retain_mut(|handle| {
        if let Some(result) = handle.now_or_never() {
            match result {
                Ok(Ok((entity, item_type, tx_hash))) => {
                    let _entity = entity;
                    info!(
                        "Blockchain pickup tx completed: {} for {:?}",
                        tx_hash, item_type
                    );
                    item_picked_up_events.write(ItemPickedUpEvent {
                        item_type,
                        transaction_hash: tx_hash,
                    });
                }
                Ok(Err((entity, item_type, err))) => {
                    let _entity = entity;
                    item_pickup_failed_events.write(ItemPickupFailedEvent {
                        item_type,
                        error: err,
                    });
                }
                Err(join_err) => {
                    error!("JoinHandle error in pickup task: {:?}", join_err);
                }
            }
            false // Remove finished handle
        } else {
            true // Keep unfinished handle
        }
    });
}
