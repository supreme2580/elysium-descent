use crate::assets::ModelAssets;
use avian3d::prelude::*;
use bevy::prelude::*;

use crate::screens::Screen;
use crate::systems::character_controller::CharacterController;
use crate::systems::dojo::PickupItemEvent;

// ===== COMPONENTS & RESOURCES =====

#[derive(Component)]
pub struct Collectible;

#[derive(Component)]
pub struct Collected;

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
    Coin,
}

#[derive(Resource)]
pub struct NextItemToAdd(pub CollectibleType);

#[derive(Component)]
pub struct Sensor;

/// Component marking objects that can be interacted with
#[derive(Component, Clone, Copy)]
pub struct Interactable {
    pub interaction_radius: f32,
}

/// Event triggered when player presses interaction key
#[derive(Event, Debug)]
pub struct InteractionEvent;

// Configuration for spawning collectibles
#[derive(Clone)]
pub struct CollectibleConfig {
    pub position: Vec3,
    pub collectible_type: CollectibleType,
    pub scale: f32,
    pub rotation: Option<CollectibleRotation>,
}

// ===== PLUGIN =====

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InteractionEvent>()
            .insert_resource(crate::ui::inventory::InventoryVisibilityState::default())
            .add_systems(
                Update,
                (
                    auto_collect_nearby_interactables,
                    handle_interactions,
                    update_floating_items,
                    rotate_collectibles,
                    crate::ui::inventory::add_item_to_inventory,
                    crate::ui::inventory::toggle_inventory_visibility,
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
        CollectibleType::Coin => assets.coin.clone(),
    };

    let mut entity = commands.spawn((
        Name::new(format!("{:?}", config.collectible_type)),
        SceneRoot(model_handle),
        Transform {
            translation: config.position,
            scale: Vec3::splat(config.scale),
            ..default()
        },
        Collider::sphere(0.5),
        RigidBody::Kinematic,
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Collectible,
        config.collectible_type,
        FloatingItem {
            base_height: config.position.y,
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        Sensor,
        scene_marker.clone(),
        Interactable {
            interaction_radius: 4.0,
        },
    ));

    if let Some(rotation) = config.rotation {
        entity.insert(rotation);
    }
}

/// System to automatically collect any collectible when the player is within the Interactable's radius
fn auto_collect_nearby_interactables(
    mut commands: Commands,
    player_query: Query<&Transform, With<CharacterController>>,
    interactable_query: Query<
        (Entity, &Transform, &Interactable, &CollectibleType),
        Without<Collected>,
    >,
    mut pickup_events: EventWriter<PickupItemEvent>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (entity, transform, interactable, collectible_type) in interactable_query.iter() {
        let distance = player_transform.translation.distance(transform.translation);
        if distance <= interactable.interaction_radius {
            if *collectible_type == CollectibleType::Coin {
                // Mark as collected
                commands.entity(entity).insert(Collected);
                // Insert NextItemToAdd so inventory system will add it
                commands.insert_resource(NextItemToAdd(*collectible_type));
                // Despawn the entity immediately
                commands.entity(entity).despawn();
                // Trigger blockchain event
                pickup_events.write(PickupItemEvent {
                    item_type: *collectible_type,
                    item_entity: entity,
                });
            }
        }
    }
}

/// System to handle pressing E near a Book
fn handle_interactions(
    mut commands: Commands,
    player_query: Query<&Transform, With<CharacterController>>,
    interactable_query: Query<
        (Entity, &Transform, &Interactable, &CollectibleType),
        Without<Collected>,
    >,
    mut pickup_events: EventWriter<PickupItemEvent>,
    mut interaction_events: EventReader<InteractionEvent>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let mut interacted = false;
    for _ in interaction_events.read() {
        for (entity, transform, interactable, collectible_type) in interactable_query.iter() {
            let distance = player_transform.translation.distance(transform.translation);
            if distance <= interactable.interaction_radius
                && *collectible_type == CollectibleType::Book
            {
                // Mark as collected
                commands.entity(entity).insert(Collected);
                // Insert NextItemToAdd so inventory system will add it
                commands.insert_resource(NextItemToAdd(*collectible_type));
                // Despawn the entity immediately
                commands.entity(entity).despawn();
                // Trigger blockchain event
                pickup_events.write(PickupItemEvent {
                    item_type: *collectible_type,
                    item_entity: entity,
                });
                // Transition to fight scene
                next_state.set(Screen::FightScene);
                interacted = true;
                break;
            }
        }
        if interacted {
            break;
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

/// Resource to track current book being interacted with
#[derive(Resource, Default)]
pub struct CurrentBookEntity;

/// Helper function to spawn an interactable book
pub fn spawn_interactable_book(
    commands: &mut Commands,
    assets: &Res<ModelAssets>,
    position: Vec3,
    scale: f32,
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
    entity.insert((Collider::sphere(0.5), RigidBody::Kinematic));

    // Add visibility components
    entity.insert((
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // Add collectible components
    entity.insert((
        Collectible,
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
