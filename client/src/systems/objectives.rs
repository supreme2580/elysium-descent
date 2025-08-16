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
pub struct Objective {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub item_type: CollectibleType,
    pub required_count: u32,
    pub current_count: u32,
    pub completed: bool,
}

impl Objective {
    pub fn new(id: usize, title: String, description: String, item_type: CollectibleType, required_count: u32) -> Self {
        Self {
            id,
            title,
            description,
            item_type,
            required_count,
            current_count: 0,
            completed: false,
        }
    }

    // Removed unused is_completed and add_progress methods
}

#[derive(Resource, Default)]
pub struct ObjectiveManager {
    pub objectives: Vec<Objective>,
    pub next_id: usize,
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
    font_assets: Option<Res<crate::assets::FontAssets>>,
    ui_assets: Option<Res<crate::assets::UiAssets>>,
    _objectives_ui_query: Query<Entity, With<ObjectiveUI>>,
    objectives_list_query: Query<Entity, (With<Node>, With<Name>)>,
    existing_slots: Query<Entity, With<ObjectiveSlot>>,
    view_more_query: Query<Entity, With<Name>>,
    _children: Query<&Children>,
    names: Query<&Name>,
) {
    if !objective_manager.is_changed() {
        return; // Only update when objectives change
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
            let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone())).id();
            commands.entity(list_entity).add_child(slot_entity);
        }

        // Add "View More" button after objectives
        let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
        commands.entity(list_entity).add_child(view_more_entity);
    } else if existing_slots_count == objectives_count {
        // Check if any objectives have changed progress
        let mut needs_update = false;
        for (slot_index, _slot_entity) in existing_slots.iter().enumerate() {
            if let Some(_objective) = objective_manager.objectives.get(slot_index) {
                // For now, always recreate to ensure UI updates
                needs_update = true;
                break;
            }
        }
        
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
                let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone())).id();
                commands.entity(list_entity).add_child(slot_entity);
            }
            
            // Add "View More" button after objectives
            let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
            commands.entity(list_entity).add_child(view_more_entity);
        } else {
            // Update existing slots - just update the progress values
            update_existing_objective_slots(
                &mut commands,
                &objective_manager,
                &existing_slots,
                &names,
            );
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
            let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone())).id();
            commands.entity(list_entity).add_child(slot_entity);
        }

        // Add "View More" button after objectives
        let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
        commands.entity(list_entity).add_child(view_more_entity);
    }
}

/// Function to update existing objective slots without recreating the entire UI
fn update_existing_objective_slots(
    _commands: &mut Commands,
    objective_manager: &ObjectiveManager,
    existing_slots: &Query<Entity, With<ObjectiveSlot>>,
    _names: &Query<&Name>,
) {
    info!("Updating {} existing objective slots with new progress values", existing_slots.iter().count());
    
    // For now, we'll use a simpler approach: just log the updates
    // The UI will be updated on the next frame when the objectives system runs
    for (slot_index, _slot_entity) in existing_slots.iter().enumerate() {
        if let Some(objective) = objective_manager.objectives.get(slot_index) {
            info!("📊 Objective {}: '{}' - Progress: {}/{} ({}%)", 
                slot_index + 1,
                objective.title,
                objective.current_count,
                objective.required_count,
                if objective.required_count > 0 {
                    (objective.current_count as f32 / objective.required_count as f32 * 100.0) as u32
                } else {
                    100
                }
            );
        }
    }
    
    // TODO: Implement proper UI updates using Bevy's component system
    // This would require adding specific components to track progress bars and text
    // and then updating them directly through queries
}

fn create_objective_slot(
    objective: &Objective,
    font: Handle<Font>,
    item_image: Handle<Image>,
    check_icon: Handle<Image>,
) -> impl Bundle {
    let progress_percent = if objective.required_count > 0 {
        objective.current_count as f32 / objective.required_count as f32
    } else {
        1.0
    };

    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(136.0),
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
                    width: Val::Px(96.0),
                    height: Val::Px(96.0),
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
                    width: Val::Px(300.0),
                    height: Val::Px(96.0),
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
                        Text::new(format!("{}/{}", objective.current_count, objective.required_count)),
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
                            width: Val::Px(270.0),
                            height: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::DARKER_GLASS),
                        BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.4)),
                        BorderRadius::all(Val::Px(6.0)),
                        children![
                            (
                                Node {
                                    width: Val::Px(267.0 * progress_percent),
                                    height: Val::Px(9.0),
                                    margin: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::ELYSIUM_GOLD),
                                BorderRadius::all(Val::Px(4.5)),
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

 