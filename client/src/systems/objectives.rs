use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::screens::Screen;
use crate::systems::collectibles::CollectibleType;
use crate::ui::styles::ElysiumDescentColorPalette;

// ===== COMPONENTS & RESOURCES =====

#[derive(Component)]
pub struct ObjectiveUI;

#[derive(Component)]
pub struct ObjectiveSlot {
    // Removed unused objective_id field
}

#[derive(Component)]
pub struct ObjectiveCheckmark;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObjectiveType {
    Collect(CollectibleType, u32), // Collectible type and required count
    ReachLocation(Vec3, f32),      // Target position and completion radius
    Defeat(String),                 // Target entity ID
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Objective {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub objective_type: ObjectiveType,
    pub completed: bool,
}

impl Objective {
    pub fn new_collect(id: usize, title: String, description: String, item_type: CollectibleType, required_count: u32) -> Self {
        Self {
            id,
            title,
            description,
            objective_type: ObjectiveType::Collect(item_type, required_count),
            completed: false,
        }
    }

    pub fn new_location(id: usize, title: String, description: String, position: Vec3, radius: f32) -> Self {
        Self {
            id,
            title,
            description,
            objective_type: ObjectiveType::ReachLocation(position, radius),
            completed: false,
        }
    }

    pub fn new_defeat(id: usize, title: String, description: String, target_id: String) -> Self {
        Self {
            id,
            title,
            description,
            objective_type: ObjectiveType::Defeat(target_id),
            completed: false,
        }
    }

    // Helper method to get required count for collectible objectives
    pub fn get_required_count(&self) -> u32 {
        match &self.objective_type {
            ObjectiveType::Collect(_, count) => *count,
            _ => 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct ObjectiveManager {
    pub objectives: Vec<Objective>,
    pub next_id: usize,
    pub version: u32, // Track changes to trigger UI updates
}



impl ObjectiveManager {
    pub fn add_objective(&mut self, objective: Objective) {
        self.objectives.push(objective);
        self.next_id += 1;
    }

    // Removed unused update_progress, get_objective, and are_all_completed methods
}

// ===== PLUGIN =====

pub struct ObjectivesPlugin;

impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectiveManager>()
            .add_systems(OnEnter(Screen::GamePlay), setup_initial_objectives)
            .add_systems(
                Update,
                (update_objective_ui,).run_if(in_state(Screen::GamePlay)),
            );
    }
}

// ===== SYSTEMS =====

fn setup_initial_objectives(_objective_manager: ResMut<ObjectiveManager>) {
    // This function is now handled by the level manager
    // Objectives are loaded from level JSON files
}

fn update_objective_ui(
    mut commands: Commands,
    objective_manager: Res<ObjectiveManager>,
    progress_tracker: Res<crate::systems::collectibles::CollectibleProgressTracker>,
    font_assets: Option<Res<crate::assets::FontAssets>>,
    ui_assets: Option<Res<crate::assets::UiAssets>>,
    _objectives_ui_query: Query<Entity, With<ObjectiveUI>>,
    objectives_list_query: Query<Entity, (With<Node>, With<Name>)>,
    existing_slots: Query<Entity, With<ObjectiveSlot>>,
    view_more_query: Query<Entity, With<Name>>,
    _children: Query<&Children>,
    names: Query<&Name>,
) {
    // Always update progress display, but only recreate UI when objectives change
    let needs_ui_rebuild = objective_manager.is_changed();
    let needs_progress_update = progress_tracker.is_changed();
    
    if !needs_ui_rebuild && !needs_progress_update {
        return; // Only update when objectives or progress changes
    }
    


    let Some(font_assets) = font_assets else { return; };
    let Some(ui_assets) = ui_assets else { return; };

    // Find the objectives list container
    let mut objectives_list_entity = None;
    for entity in objectives_list_query.iter() {
        if let Ok(name) = names.get(entity) {
            if name.as_str() == "ObjectivesList" {
                objectives_list_entity = Some(entity);
                break;
            }
        }
    }

    let Some(list_entity) = objectives_list_entity else {
        warn!("Could not find ObjectivesList container");
        return;
    };

    // Check if we need to create the initial UI or just update existing slots
    let existing_slots_count = existing_slots.iter().count();
    let objectives_count = objective_manager.objectives.len();
    
    if existing_slots_count == 0 {
        // Initial creation - create all objective slots and View More button
        let font = font_assets.rajdhani_bold.clone();
        let coin_image = ui_assets.coin.clone();

        for objective in &objective_manager.objectives {
            let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone(), &progress_tracker)).id();
            commands.entity(list_entity).add_child(slot_entity);
        }

        // Add "View More" button after objectives
        let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
        commands.entity(list_entity).add_child(view_more_entity);
    } else if existing_slots_count == objectives_count {
        // Check if we need to update the UI
        let needs_update = needs_ui_rebuild || needs_progress_update;
        
        if needs_update {
            // Clear existing objective slots
            for slot_entity in existing_slots.iter() {
                commands.entity(slot_entity).despawn();
            }
            
            // Remove any existing View More buttons to prevent duplicates
            for entity in view_more_query.iter() {
                if let Ok(name) = names.get(entity) {
                    if name.as_str() == "View More Button" {
                        commands.entity(entity).despawn();
                    }
                }
            }
            
            // Recreate all objective slots
            let font = font_assets.rajdhani_bold.clone();
            let coin_image = ui_assets.coin.clone();
            
            for objective in &objective_manager.objectives {
                let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone(), &progress_tracker)).id();
                commands.entity(list_entity).add_child(slot_entity);
            }
            
            // Add "View More" button after objectives
            let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
            commands.entity(list_entity).add_child(view_more_entity);
        }
    } else {
        // Count mismatch - recreate everything
        // Clear existing objective slots
        for slot_entity in existing_slots.iter() {
            commands.entity(slot_entity).despawn();
        }

        // Remove any existing View More buttons to prevent duplicates
        for entity in view_more_query.iter() {
            if let Ok(name) = names.get(entity) {
                if name.as_str() == "View More Button" {
                    commands.entity(entity).despawn();
                }
            }
        }

        // Spawn new objective slots for each objective
        let font = font_assets.rajdhani_bold.clone();
        let coin_image = ui_assets.coin.clone();

        for objective in &objective_manager.objectives {
            let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone(), &progress_tracker)).id();
            commands.entity(list_entity).add_child(slot_entity);
        }

        // Add "View More" button after objectives
        let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
        commands.entity(list_entity).add_child(view_more_entity);
    }
}



fn create_objective_slot(
    objective: &Objective,
    font: Handle<Font>,
    item_image: Handle<Image>,
    check_icon: Handle<Image>,
    progress_tracker: &crate::systems::collectibles::CollectibleProgressTracker,
) -> impl Bundle {
    let (progress_text, progress_percent) = match &objective.objective_type {
        ObjectiveType::Collect(collectible_type, required_count) => {
            let current_count = match collectible_type {
                crate::systems::collectibles::CollectibleType::Coin => progress_tracker.coins_collected,
                crate::systems::collectibles::CollectibleType::Book => progress_tracker.books_collected,
                crate::systems::collectibles::CollectibleType::HealthPotion => progress_tracker.health_potions_collected,
                crate::systems::collectibles::CollectibleType::SurvivalKit => progress_tracker.survival_kits_collected,
            };
            let percent = if *required_count > 0 {
                current_count as f32 / *required_count as f32
            } else {
                1.0
            };
            (format!("{}/{}", current_count, required_count), percent)
        },
        ObjectiveType::ReachLocation(_, _) => {
            let percent = if objective.completed { 1.0 } else { 0.0 };
            (if objective.completed { "COMPLETED" } else { "GO TO LOCATION" }.to_string(), percent)
        },
        ObjectiveType::Defeat(_) => {
            let percent = if objective.completed { 1.0 } else { 0.0 };
            (if objective.completed { "COMPLETED" } else { "DEFEAT TARGET" }.to_string(), percent)
        },
    };

    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(160.0), // Increased from 136px to 160px for better progress bar spacing
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(12.0)),
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::DARKER_GLASS),
        BorderRadius::all(Val::Px(18.0)),
        ObjectiveSlot {
            // Removed unused objective_id field
        },
        children![
            // Item Icon Container
            (
                Node {
                    width: Val::Px(113.0), // Increased from 96px to 113px to maintain proportions (17.6% increase)
                    height: Val::Px(120.0), // Increased from 96px to 120px to maintain proportions
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::LIGHT_GLASS),
                BorderRadius::all(Val::Px(12.0)),
                                children![
                    // Item icon (always visible)
                    (
                        Node {
                            width: Val::Px(72.0),
                            height: Val::Px(72.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        BorderRadius::all(Val::Px(9.0)),
                        Name::new("ItemIcon"),
                        children![(
                            ImageNode {
                                image: item_image,
                                ..Default::default()
                            },
                            Node {
                                width: Val::Px(72.0),
                                height: Val::Px(72.0),
                                ..default()
                            },
                        )]
                    ),
                    // Small green checkmark at bottom right for completed objectives
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(-8.0),
                            bottom: Val::Px(-8.0),
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            display: if objective.completed { Display::Flex } else { Display::None },
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::SUCCESS_GREEN),
                        BorderColor(Color::SUCCESS_GREEN),
                        BorderRadius::MAX,
                        Name::new("CompletionCheckmark"),
                        children![(
                            ImageNode {
                                image: check_icon,
                                ..Default::default()
                            },
                            Node {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                ..default()
                            },
                        )]
                    )
                ]
            ),
            // Objective Info Container
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Px(353.0), // Increased from 300px to 353px to maintain proportions (17.6% increase)
                    height: Val::Px(120.0), // Increased from 96px to 120px to maintain proportions
                    ..default()
                },
                children![
                    // Objective Title
                    (
                        Text::new(&objective.title),
                        TextFont {
                            font: font.clone(),
                            font_size: 21.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                    ),
                    // Objective Description
                    (
                        Text::new(&objective.description),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                    ),
                    // Progress Text
                    (
                        Text::new(&progress_text),
                        TextFont {
                            font: font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::ELYSIUM_GOLD),
                        Node {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                    ),
                    // Progress Bar
                    (
                        Node {
                            width: Val::Px(318.0), // Increased from 270px to 318px to maintain proportions (17.6% increase)
                            height: Val::Px(16.0), // Increased from 12px to 16px for better visibility
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::DARKER_GLASS),
                        BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.4)),
                        BorderRadius::all(Val::Px(8.0)), // Adjusted border radius for new height
                        children![
                            (
                                Node {
                                    width: Val::Px(315.0 * progress_percent), // Increased from 267px to 315px to maintain proportions (17.6% increase)
                                    height: Val::Px(13.0), // Increased from 9px to 13px for better visibility
                                    margin: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::ELYSIUM_GOLD),
                                BorderRadius::all(Val::Px(6.5)), // Adjusted border radius for new height
                            )
                        ]
                    )
                ]
            )
        ],
    )
}

fn create_view_more_button(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(32.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(12.0)),
            ..default()
        },
        Name::new("View More Button"),
        Interaction::None,
        children![(
            Text::new("VIEW MORE"),
            TextFont {
                font,
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::ELYSIUM_GOLD),
        )]
    )
}

 