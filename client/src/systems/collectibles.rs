use crate::assets::ModelAssets;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::sync::Arc;

use crate::systems::dojo::PickupItemEvent;
use crate::{systems::character_controller::CharacterController, ui::inventory};
use crate::screens::Screen;

// ===== COMPONENTS & RESOURCES =====

#[derive(Resource)]
pub struct CollectibleCounter {
    pub collectibles_collected: u32,
}

#[derive(Component)]
pub struct Collectible {
    pub on_collect: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

#[derive(Component, Clone)]
pub struct CollectibleRotation {
    pub enabled: bool,
    pub clockwise: bool,
    pub speed: f32,
}

#[derive(Component)]
pub struct FloatingItem {
    pub base_height: f32,
    pub hover_amplitude: f32,
    pub hover_speed: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum CollectibleType {
    Book,
    FirstAidKit,
}


#[derive(Resource)]
pub struct NextItemToAdd(pub CollectibleType);


#[derive(Component)]
pub struct Sensor;

/// Component marking objects that can be interacted with
#[derive(Component)]
pub struct Interactable {
    pub interaction_radius: f32,
}

/// Event triggered when player presses interaction key
#[derive(Event, Debug)]
pub struct InteractionEvent;

/// Event triggered when player starts being near an interactable object
#[derive(Event, Debug)]
pub struct InteractionPromptEvent;

/// Resource to track current interactable object
#[derive(Resource, Default)]
pub struct NearbyInteractable {
    pub entity: Option<Entity>,
    pub distance: f32,
}

// Configuration for spawning collectibles
#[derive(Clone)]
pub struct CollectibleConfig {
    pub position: Vec3,
    pub collectible_type: CollectibleType,
    pub scale: f32,
    pub rotation: Option<CollectibleRotation>,
    pub on_collect: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

// ===== PLUGIN =====

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CollectibleCounter {
            collectibles_collected: 0,
        })
        .add_event::<InteractionEvent>()
        .add_event::<InteractionPromptEvent>()
        .init_resource::<NearbyInteractable>()
        .insert_resource(inventory::InventoryVisibilityState::default())
        .add_systems(
            Update,
            (
                collect_items, 
                update_floating_items, 
                rotate_collectibles,
                detect_nearby_interactables,
                handle_interactions,
                inventory::add_item_to_inventory.run_if(in_state(Screen::GamePlay)),
                inventory::toggle_inventory_visibility.run_if(in_state(Screen::GamePlay)),
            )
                .run_if(in_state(Screen::GamePlay)),
        );
    }
}

// ===== SYSTEMS =====


pub fn spawn_collectible(
    commands: &mut Commands,
    assets: &Res<ModelAssets>,
    config: CollectibleConfig,
    scene_marker: impl Component + Clone,
) {
    let model_handle = match config.collectible_type {
        CollectibleType::Book => assets.book.clone(),
        CollectibleType::FirstAidKit => assets.first_aid_kit.clone(),
    };

    let mut entity = commands.spawn((
        Name::new(format!("{:?}", config.collectible_type)),
        SceneRoot(model_handle),
        Transform {
            translation: config.position,
            scale: Vec3::splat(config.scale),
            ..default()
        },
        Collider::sphere(0.5), // Simple sphere collider - won't interfere with character movement
        RigidBody::Kinematic,
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Collectible {
            on_collect: config.on_collect,
        },
        config.collectible_type,
        FloatingItem {
            base_height: config.position.y,
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        Sensor,
        scene_marker.clone(),
    ));

    if let Some(rotation) = config.rotation {
        entity.insert(rotation);
    }
}

fn collect_items(
    mut commands: Commands,
    mut collectible_counter: ResMut<CollectibleCounter>,
    player_query: Query<&Transform, With<CharacterController>>,
    collectible_query: Query<(Entity, &Transform, &CollectibleType, &Collectible), (With<Sensor>, Without<Interactable>)>,
    mut pickup_events: EventWriter<PickupItemEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (collectible_entity, collectible_transform, collectible_type, collectible) in
        collectible_query.iter()
    {
        let distance = player_transform
            .translation
            .distance(collectible_transform.translation);
        if distance < 5.0 {
            // Collection radius - only for non-interactable items (like FirstAidKit)
            info!("Collected a {:?}!", collectible_type);
            commands.insert_resource(NextItemToAdd(*collectible_type));

            match collectible_type {
                CollectibleType::FirstAidKit => {
                    // Trigger blockchain transaction for FirstAidKit
                    info!("🏥 FirstAidKit collected - triggering blockchain transaction");
                    pickup_events.write(PickupItemEvent {
                        item_type: *collectible_type,
                        item_entity: collectible_entity,
                    });
                    
                    // Note: The item will be removed from the world when the blockchain transaction is confirmed
                    // in the pickup_item system's handle_item_picked_up_events
                }
                _ => {
                    // For other items (not FirstAidKit), use the old local collection method
                    (collectible.on_collect)(&mut commands, collectible_entity);
                }
            }

            collectible_counter.collectibles_collected += 1;
            info!(
                "Total collectibles collected: {}",
                collectible_counter.collectibles_collected
            );
        }
    }
}


fn update_floating_items(time: Res<Time>, mut query: Query<(&FloatingItem, &mut Transform)>) {
    for (floating, mut transform) in query.iter_mut() {
        let time = time.elapsed_secs();
        let hover_offset = (time * floating.hover_speed).sin() * floating.hover_amplitude;
        transform.translation.y = floating.base_height + hover_offset;
    }
}

pub fn rotate_collectibles(
    mut collectible_query: Query<(&mut Transform, &CollectibleRotation)>,
    time: Res<Time>,
) {
    for (mut transform, rotation) in collectible_query.iter_mut() {
        if rotation.enabled {
            let rotation_amount = if rotation.clockwise {
                rotation.speed * time.delta_secs()
            } else {
                -rotation.speed * time.delta_secs()
            };
            transform.rotate_y(rotation_amount);
        }
    }
}

/// System to detect when player is near interactable objects
fn detect_nearby_interactables(
    player_query: Query<&Transform, With<CharacterController>>,
    interactable_query: Query<(Entity, &Transform, &Interactable)>,
    mut nearby_interactable: ResMut<NearbyInteractable>,
    mut prompt_events: EventWriter<InteractionPromptEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    let mut closest_interactable: Option<(Entity, f32, &Interactable)> = None;

    // Find the closest interactable within range
    for (entity, transform, interactable) in interactable_query.iter() {
        let distance = player_transform.translation.distance(transform.translation);
        
        if distance <= interactable.interaction_radius {
            if let Some((_, closest_distance, _)) = closest_interactable {
                if distance < closest_distance {
                    closest_interactable = Some((entity, distance, interactable));
                }
            } else {
                closest_interactable = Some((entity, distance, interactable));
            }
        }
    }

    // Update nearby interactable state
    match closest_interactable {
        Some((entity, distance, _interactable)) => {
            if nearby_interactable.entity != Some(entity) {
                // New interactable entered range
                nearby_interactable.entity = Some(entity);
                nearby_interactable.distance = distance;
                prompt_events.write(InteractionPromptEvent);
            } else {
                // Update distance for existing interactable
                nearby_interactable.distance = distance;
            }
        }
        None => {
            if nearby_interactable.entity.is_some() {
                // Left interaction range
                nearby_interactable.entity = None;
                nearby_interactable.distance = 0.0;
                prompt_events.write(InteractionPromptEvent);
            }
        }
    }
}

/// System to handle interaction events
fn handle_interactions(
    mut interaction_events: EventReader<InteractionEvent>,
    _commands: Commands,
    mut collectible_counter: ResMut<CollectibleCounter>,
    nearby_interactable: Res<NearbyInteractable>,
    interactable_query: Query<(&CollectibleType, &Collectible), With<Interactable>>,
    mut prompt_events: EventWriter<InteractionPromptEvent>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    for _event in interaction_events.read() {
        if let Some(entity) = nearby_interactable.entity {
            if let Ok((collectible_type, _collectible)) = interactable_query.get(entity) {
                // Trigger dialogue for books, blockchain transaction for FirstAidKit, direct collection for others
                match collectible_type {
                    CollectibleType::Book => {
                        // Transition to fight scene when book is interacted with
                        info!("📚 Book interacted with - transitioning to fight scene");
                        next_state.set(Screen::FightScene);
                    }
                    CollectibleType::FirstAidKit => {
                        // Trigger blockchain transaction for FirstAidKit
                        info!("🏥 FirstAidKit interacted with - triggering blockchain transaction");
                        collectible_counter.collectibles_collected += 1;
                        info!(
                            "Total collectibles collected: {}",
                            collectible_counter.collectibles_collected
                        );
                    }
                }

                // Hide the interaction prompt
                prompt_events.write(InteractionPromptEvent);
            }
        }
    }
}

/// Resource to track current book being interacted with
#[derive(Resource, Default)]
pub struct CurrentBookEntity;

/// Helper function to spawn an interactable book
pub fn spawn_interactable_book(
    commands: &mut Commands,
    assets: &Res<ModelAssets>,
    position: Vec3,
    scale: f32,
    on_collect: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
    scene_marker: impl Component + Clone,
) {
    let mut entity = commands.spawn((
        Name::new("Interactable Book"),
        SceneRoot(assets.book.clone()),
        Transform {
            translation: position,
            scale: Vec3::splat(scale),
            ..default()
        },
        scene_marker.clone(),
    ));

    // Add physics components - simple sphere collider to avoid character movement interference
    entity.insert((
        Collider::sphere(0.5),
        RigidBody::Kinematic,
    ));

    // Add visibility components
    entity.insert((
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // Add collectible components
    entity.insert((
        Collectible { on_collect },
        CollectibleType::Book,
        FloatingItem {
            base_height: position.y,
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        Sensor,
    ));

    // Add interaction components
    entity.insert((
        Interactable {
            interaction_radius: 5.0,
        },
        CollectibleRotation {
            enabled: true,
            clockwise: true,
            speed: 1.0,
        },
    ));
}
