use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::screens::Screen;
use crate::ui::styles::ElysiumDescentColorPalette;
use crate::game::level_manager::{LevelManager, LevelCompletedEvent, ObjectiveData as LevelObjectiveData};

// ===== COMPONENTS & RESOURCES =====

#[derive(Component)]
pub struct ObjectiveUI;

#[derive(Component)]
pub struct ObjectiveSlot {
    pub objective_id: String,
}

#[derive(Component)]
pub struct ObjectiveCheckmark;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Objective {
    pub id: String,
    pub title: String,
    pub description: String,
    pub objective_type: String,
    pub target: String,
    pub required_count: Option<u32>,
    pub current_count: u32,
    pub completed: bool,
    pub position: Option<Vec3>,
    pub completion_radius: Option<f32>,
    pub reward: String,
}

impl Objective {
    pub fn new(
        id: String,
        title: String,
        description: String,
        objective_type: String,
        target: String,
        required_count: Option<u32>,
        position: Option<Vec3>,
        completion_radius: Option<f32>,
        reward: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            objective_type,
            target,
            required_count,
            current_count: 0,
            completed: false,
            position,
            completion_radius,
            reward,
        }
    }

    pub fn from_level_data(level_obj: &LevelObjectiveData) -> Self {
        let position = level_obj.position.as_ref().map(|pos| Vec3::new(pos.x, pos.y, pos.z));
        
        Self::new(
            level_obj.id.clone(),
            level_obj.title.clone(),
            level_obj.description.clone(),
            level_obj.objective_type.clone(),
            level_obj.target.clone(),
            level_obj.required_count,
            position,
            level_obj.completion_radius,
            level_obj.reward.clone(),
        )
    }

    pub fn is_completed(&self) -> bool {
        if let Some(required) = self.required_count {
            self.current_count >= required
        } else {
            self.completed
        }
    }

    pub fn add_progress(&mut self, amount: u32) {
        self.current_count += amount;
        if self.is_completed() {
            self.completed = true;
        }
    }
}

#[derive(Resource, Default)]
pub struct ObjectiveManager {
    pub objectives: Vec<Objective>,
    pub level_objectives_completed: u32,
    pub total_level_objectives: u32,
}

impl ObjectiveManager {
    pub fn add_objective(&mut self, objective: Objective) {
        self.objectives.push(objective);
        self.total_level_objectives += 1;
    }

    pub fn update_progress(&mut self, target: &str, amount: u32) {
        for objective in &mut self.objectives {
            if objective.target == target && !objective.completed {
                objective.add_progress(amount);
                if objective.completed {
                    self.level_objectives_completed += 1;
                }
            }
        }
    }

    pub fn get_objective(&self, id: &str) -> Option<&Objective> {
        self.objectives.iter().find(|obj| obj.id == id)
    }

    pub fn are_all_completed(&self) -> bool {
        self.level_objectives_completed >= self.total_level_objectives && self.total_level_objectives > 0
    }

    pub fn clear_objectives(&mut self) {
        self.objectives.clear();
        self.level_objectives_completed = 0;
        self.total_level_objectives = 0;
    }
}

// ===== PLUGIN =====

pub struct ObjectivesPlugin;

impl Plugin for ObjectivesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObjectiveManager>()
            .add_systems(OnEnter(Screen::GamePlay), setup_initial_objectives)
            .add_systems(
                Update,
                (update_objective_ui, check_level_completion).run_if(in_state(Screen::GamePlay)),
            )
            .add_systems(OnExit(Screen::GamePlay), clear_objectives_on_exit);
    }
}

// ===== SYSTEMS =====

fn setup_initial_objectives(
    mut objective_manager: ResMut<ObjectiveManager>,
    level_manager: Res<LevelManager>,
) {
    // Clear any existing objectives
    objective_manager.clear_objectives();

    // Get objectives from current level
    let current_level = level_manager.get_current_level();
    
    for level_objective in &current_level.objectives {
        let objective = Objective::from_level_data(level_objective);
        objective_manager.add_objective(objective);
    }
}

fn check_level_completion(
    objective_manager: Res<ObjectiveManager>,
    mut level_completed_events: EventWriter<LevelCompletedEvent>,
    level_manager: Res<LevelManager>,
) {
    if objective_manager.are_all_completed() {
        level_completed_events.write(LevelCompletedEvent {
            level_id: level_manager.current_level,
        });
    }
}

fn clear_objectives_on_exit(mut objective_manager: ResMut<ObjectiveManager>) {
    objective_manager.clear_objectives();
}

fn update_objective_ui(
    mut commands: Commands,
    objective_manager: Res<ObjectiveManager>,
    font_assets: Option<Res<crate::assets::FontAssets>>,
    ui_assets: Option<Res<crate::assets::UiAssets>>,
    _objectives_ui_query: Query<Entity, With<ObjectiveUI>>,
    objectives_list_query: Query<Entity, (With<Node>, With<Name>)>,
    existing_slots: Query<Entity, With<ObjectiveSlot>>,
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

    // Clear existing objective slots
    for slot_entity in existing_slots.iter() {
        commands.entity(slot_entity).despawn();
    }

    // Spawn new objective slots for each objective
    let font = font_assets.rajdhani_bold.clone();
    let coin_image = ui_assets.coin.clone(); // Using coin as placeholder for all items

    for objective in &objective_manager.objectives {
        let slot_entity = commands.spawn(create_objective_slot(objective, font.clone(), coin_image.clone(), ui_assets.green_check_icon.clone())).id();
        commands.entity(list_entity).add_child(slot_entity);
    }

    // Add "View More" button after objectives
    let view_more_entity = commands.spawn(create_view_more_button(font.clone())).id();
    commands.entity(list_entity).add_child(view_more_entity);
}

fn create_objective_slot(
    objective: &Objective,
    font: Handle<Font>,
    item_image: Handle<Image>,
    check_icon: Handle<Image>,
) -> impl Bundle {
    let progress_percent = if objective.required_count.is_some() {
        objective.current_count as f32 / objective.required_count.unwrap() as f32
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
            objective_id: objective.id.clone(),
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
                        Text::new(format!("{}/{}", objective.current_count, objective.required_count.unwrap_or(0))),
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

 