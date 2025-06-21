use crate::assets::ModelAssets;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;
use bevy_yarnspinner::events::ExecuteCommandEvent;
use std::sync::Arc;

use crate::systems::character_controller::CharacterController;
use crate::systems::dojo::PickupItemEvent;
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

#[derive(Component, Clone, Copy, Debug)]
pub enum CollectibleType {
    Book,
    FirstAidKit,
}

#[derive(Component)]
pub struct Sensor;

/// Component marking objects that can be interacted with
#[derive(Component)]
pub struct Interactable {
    pub interaction_radius: f32,
    pub prompt_text: String,
}

/// Event triggered when player presses interaction key
#[derive(Event, Debug)]
pub struct InteractionEvent;

/// Event triggered when player starts being near an interactable object
#[derive(Event, Debug)]
pub struct InteractionPromptEvent {
    pub show: bool,
    pub text: String,
}

/// Event to trigger book dialogue
#[derive(Event, Debug)]
pub struct StartBookDialogueEvent {
    pub book_entity: Entity,
}

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
        .add_event::<StartBookDialogueEvent>()
        .init_resource::<NearbyInteractable>()
        .init_resource::<CurrentBookEntity>()
        .add_systems(
            Update,
            (
                collect_items, 
                update_floating_items, 
                rotate_collectibles,
                detect_nearby_interactables,
                handle_interactions,
                handle_book_dialogue_events,
                handle_dialogue_commands,
                debug_dialogue_system,
                update_interaction_prompts,
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
        Some((entity, distance, interactable)) => {
            if nearby_interactable.entity != Some(entity) {
                // New interactable entered range
                // warn!("🔍 PROXIMITY: Player entered range of interactable entity {:?} - '{}'", entity, interactable.prompt_text);
                nearby_interactable.entity = Some(entity);
                nearby_interactable.distance = distance;
                prompt_events.write(InteractionPromptEvent {
                    show: true,
                    text: interactable.prompt_text.clone(),
                });
            } else {
                // Update distance for existing interactable
                nearby_interactable.distance = distance;
            }
        }
        None => {
            if nearby_interactable.entity.is_some() {
                // Left interaction range
                // warn!("🚶 PROXIMITY: Player left interaction range");
                nearby_interactable.entity = None;
                nearby_interactable.distance = 0.0;
                prompt_events.write(InteractionPromptEvent {
                    show: false,
                    text: String::new(),
                });
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
    mut book_dialogue_events: EventWriter<StartBookDialogueEvent>,
    mut pickup_events: EventWriter<PickupItemEvent>,
) {
    for _event in interaction_events.read() {
        // warn!("🎯 INTERACTION EVENT RECEIVED! Checking for nearby interactable...");
        
        if let Some(entity) = nearby_interactable.entity {
            // warn!("✅ Found nearby interactable entity: {:?}", entity);
            
            if let Ok((collectible_type, _collectible)) = interactable_query.get(entity) {
                // warn!("✅ Entity is valid with type: {:?}", collectible_type);
                
                // Trigger dialogue for books, blockchain transaction for FirstAidKit, direct collection for others
                match collectible_type {
                    CollectibleType::Book => {
                        // warn!("📚 BOOK DETECTED! Triggering StartBookDialogueEvent...");
                        book_dialogue_events.write(StartBookDialogueEvent {
                            book_entity: entity,
                        });
                        // warn!("📚 StartBookDialogueEvent SENT!");
                    }
                    CollectibleType::FirstAidKit => {
                        // Trigger blockchain transaction for FirstAidKit
                        info!("🏥 FirstAidKit interacted with - triggering blockchain transaction");
                        pickup_events.write(PickupItemEvent {
                            item_type: *collectible_type,
                            item_entity: entity,
                        });
                        collectible_counter.collectibles_collected += 1;
                        info!(
                            "Total collectibles collected: {}",
                            collectible_counter.collectibles_collected
                        );
                    }
                }

                // Hide the interaction prompt
                prompt_events.write(InteractionPromptEvent {
                    show: false,
                    text: String::new(),
                });
            } else {
                // warn!("❌ Nearby entity is not a valid interactable!");
            }
        } else {
            // warn!("❌ No nearby interactable entity when E was pressed!");
        }
    }
}

/// Resource to track current book being interacted with
#[derive(Resource, Default)]
pub struct CurrentBookEntity {
    pub entity: Option<Entity>,
}

/// System to handle book dialogue events
fn handle_book_dialogue_events(
    mut book_dialogue_events: EventReader<StartBookDialogueEvent>,
    mut dialogue_runner_query: Query<&mut DialogueRunner>,
    mut commands: Commands,
    mut collectible_counter: ResMut<CollectibleCounter>,
    mut current_book: ResMut<CurrentBookEntity>,
    book_query: Query<&Collectible, With<CollectibleType>>,
) {
    for event in book_dialogue_events.read() {
        // warn!("🎯 BOOK INTERACTION EVENT TRIGGERED! Starting dialogue for book entity: {:?}", event.book_entity);
        
        // Store the current book entity so we can collect it later
        current_book.entity = Some(event.book_entity);
        
        // Try different approaches to start dialogue
        match dialogue_runner_query.single_mut() {
            Ok(mut dialogue_runner) => {
                // warn!("✅ Found DialogueRunner, attempting to start Ancient_Tome dialogue");
                
                // Check if dialogue is already running
                if dialogue_runner.is_running() {
                    // warn!("⚠️  DialogueRunner is already running dialogue - stopping first");
                    dialogue_runner.stop();
                }
                
                // Detailed logging before starting dialogue
                // warn!("🎬 STARTING DIALOGUE:");
                // warn!("  📍 Node: 'Ancient_Tome'");
                // warn!("  🏃 Runner state before: running={}", dialogue_runner.is_running());
                
                // Start the dialogue - this method doesn't return Result, just starts the node
                dialogue_runner.start_node("Ancient_Tome");
                
                // Immediately check state after starting
                // warn!("🎉 SUCCESS: DialogueRunner.start_node('Ancient_Tome') called!");
                // warn!("🔄 Runner state after: running={}", dialogue_runner.is_running());
                
                // Force an immediate continue to ensure first line appears
                // warn!("🔄 Calling continue_in_next_update() to trigger first event...");
                dialogue_runner.continue_in_next_update();
            }
            Err(_e) => {
                // No DialogueRunner found - try to create one for this interaction
                // warn!("❌ No DialogueRunner found: {:?}. Available runners: {}", e, dialogue_runner_query.iter().count());
                
                // Fallback to simple book collection
                info!("📖 Fallback: You found an ancient tome! It contains mystical knowledge about Elysium's depths.");
                info!("The book's wisdom becomes part of your understanding.");
                
                // Collect the book
                if let Ok(collectible) = book_query.get(event.book_entity) {
                    (collectible.on_collect)(&mut commands, event.book_entity);
                    collectible_counter.collectibles_collected += 1;
                    info!("Book collected! Total collectibles: {}", collectible_counter.collectibles_collected);
                }
            }
        }
    }
}

/// System to handle dialogue commands like collect_book
fn handle_dialogue_commands(
    mut command_events: EventReader<ExecuteCommandEvent>,
    mut commands: Commands,
    mut collectible_counter: ResMut<CollectibleCounter>,
    mut current_book: ResMut<CurrentBookEntity>,
    book_query: Query<&Collectible, With<CollectibleType>>,
) {
    for command_event in command_events.read() {
        info!("Received dialogue command: {:?}", command_event.command);
        
        match command_event.command.name.as_str() {
            "collect_book" => {
                if let Some(book_entity) = current_book.entity {
                    info!("Collecting book from dialogue command");
                    if let Ok(collectible) = book_query.get(book_entity) {
                        (collectible.on_collect)(&mut commands, book_entity);
                        collectible_counter.collectibles_collected += 1;
                        info!(
                            "Total collectibles collected: {}",
                            collectible_counter.collectibles_collected
                        );
                    }
                    current_book.entity = None; // Clear the current book
                } else {
                    warn!("collect_book command received but no current book entity");
                }
            }
            _ => {
                info!("Unknown dialogue command: {:?}", command_event.command);
            }
        }
    }
}

/// System to debug dialogue system state
fn debug_dialogue_system(
    dialogue_runners: Query<&DialogueRunner>,
    yarn_project: Option<Res<YarnProject>>,
    mut debug_timer: Local<f32>,
    time: Res<Time>,
) {
    *debug_timer += time.delta_secs();
    
    // Only log every 5 seconds to avoid spam
    if *debug_timer > 5.0 {
        *debug_timer = 0.0;
        
        let runner_count = dialogue_runners.iter().count();
        let _project_exists = yarn_project.is_some();
        
        if runner_count == 0 {
            // warn!("🔍 YARN DEBUG: DialogueRunners: {}, YarnProject exists: {}", 
            //       runner_count, project_exists);
            // warn!("❌ No DialogueRunner entities found! This is why dialogue isn't working.");
        } else {
            // info!("✅ YARN DEBUG: Found {} DialogueRunner(s), YarnProject exists: {}", 
            //       runner_count, project_exists);
            
            // Check if any runners are actually running dialogue
            let mut _active_runners = 0;
            for runner in dialogue_runners.iter() {
                if runner.is_running() {
                    _active_runners += 1;
                }
            }
            
            // if active_runners > 0 {
            //     info!("🎯 ACTIVE DIALOGUE: {} runner(s) currently running dialogue", active_runners);
            // } else {
            //     info!("⚠️  IDLE RUNNERS: All {} runner(s) are idle (no dialogue running)", runner_count);
            // }
            
            // YarnProject exists, dialogue should work
            // if yarn_project.is_some() {
            //     info!("✅ YarnProject resource exists - dialogue system should be ready");
            // }
        }
    }
}

/// System to update interaction prompt UI (placeholder for now)
fn update_interaction_prompts(
    mut prompt_events: EventReader<InteractionPromptEvent>,
) {
    for event in prompt_events.read() {
        if event.show {
            info!("SHOW PROMPT: {}", event.text);
            // TODO: Show UI prompt with event.text
        } else {
            info!("HIDE PROMPT");
            // TODO: Hide UI prompt
        }
    }
}

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
            interaction_radius: 3.0,
            prompt_text: "Press E to read".to_string(),
        },
        CollectibleRotation {
            enabled: true,
            clockwise: true,
            speed: 1.0,
        },
    ));
}
