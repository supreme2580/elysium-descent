use bevy::prelude::*;
use crate::assets::{FontAssets, UiAssets};
use crate::ui::styles::ElysiumDescentColorPalette;
use crate::systems::objectives::ObjectiveManager;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};

// ===== MODAL COMPONENTS =====

#[derive(Component)]
pub struct ModalUI;

#[derive(Component)]
pub struct ModalBackground;

#[derive(Component)]
pub struct ModalContent;

#[derive(Component)]
pub struct NavigationTab {
    #[allow(dead_code)]
    pub tab_name: String,
    #[allow(dead_code)]
    pub is_active: bool,
}

#[derive(Component)]
pub struct QuestEntry {
    #[allow(dead_code)]
    pub quest_id: String,
}

#[derive(Component)]
pub struct QuestIcon;

#[derive(Component)]
pub struct QuestReward;

#[derive(Component)]
pub struct QuestEntriesContainer;

#[derive(Resource)]
pub struct ModalState {
    pub visible: bool,
    #[allow(dead_code)]
    pub active_tab: String,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            visible: false,
            active_tab: "QUESTS".to_string(),
        }
    }
}

// ===== MODAL SYSTEMS =====

pub fn spawn_objectives_modal(commands: &mut Commands, font_assets: &Res<FontAssets>, ui_assets: &Res<UiAssets>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent background
            ModalBackground,
            Visibility::Hidden,
            ZIndex(99), // High Z-index for background, just below modal content
        ))
        .with_children(|parent| {
            // Main modal panel - scaled up by 1.5x
            parent.spawn((
                Node {
                    width: Val::Px(1350.0), // Scaled up by 1.5x from 900
                    height: Val::Px(900.0), // Scaled up by 1.5x from 600
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)), // Thin gold border
                    padding: UiRect::all(Val::Px(60.0)), // Increased padding to accommodate flower images
                    margin: UiRect::top(Val::Px(-75.0)), // Scaled up by 1.5x from -50
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.95)), // Darker background
                BorderColor(Color::ELYSIUM_GOLD),
                BorderRadius::all(Val::Px(12.0)), // Scaled up by 1.5x from 8
                ModalContent,
                ZIndex(1000), // Very high Z-index to ensure it's always above other UI elements
            ))
            .with_children(|modal| {
                // Decorative corner elements (flower images)
                // Top-left corner
                modal.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(15.0), // Increased padding
                        left: Val::Px(15.0), // Increased padding
                        width: Val::Px(60.0), // Larger size for flower
                        height: Val::Px(60.0), // Larger size for flower
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: ui_assets.flower.clone(),
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(0.0)), // No rotation
                ));
                
                // Top-right corner
                modal.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(15.0), // Increased padding
                        right: Val::Px(15.0), // Increased padding
                        width: Val::Px(60.0), // Larger size for flower
                        height: Val::Px(60.0), // Larger size for flower
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: ui_assets.flower.clone(),
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(90.0_f32.to_radians())), // 90 degrees
                ));
                
                // Bottom-left corner
                modal.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(15.0), // Increased padding
                        left: Val::Px(15.0), // Increased padding
                        width: Val::Px(60.0), // Larger size for flower
                        height: Val::Px(60.0), // Larger size for flower
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: ui_assets.flower.clone(),
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(270.0_f32.to_radians())), // 270 degrees
                ));
                
                // Bottom-right corner
                modal.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(15.0), // Increased padding
                        right: Val::Px(15.0), // Increased padding
                        width: Val::Px(60.0), // Larger size for flower
                        height: Val::Px(60.0), // Larger size for flower
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: ui_assets.flower.clone(),
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(180.0_f32.to_radians())), // 180 degrees
                ));
                
                // Navigation tabs - more prominent
                let tabs = ["INVENTORY", "QUESTS", "CONTROLLER", "SETTINGS", "STATS"];
                modal.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(75.0), // Scaled up by 1.5x from 50
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceEvenly,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(45.0)), // Scaled up by 1.5x from 30
                        ..default()
                    },
                ))
                .with_children(|tabs_parent| {
                    // Spawn each tab
                    for (_i, tab_name) in tabs.iter().enumerate() {
                        let is_active = *tab_name == "QUESTS";
                        tabs_parent.spawn((
                            Node {
                                padding: UiRect::all(Val::Px(18.0)), // Scaled up by 1.5x from 12
                                border: if is_active { UiRect::bottom(Val::Px(3.0)) } else { UiRect::all(Val::Px(0.0)) }, // Scaled up by 1.5x from 2
                                ..default()
                            },
                            BackgroundColor(if is_active { 
                                Color::srgba(0.15, 0.17, 0.21, 0.8) 
                            } else { 
                                Color::NONE 
                            }),
                            BorderColor(Color::ELYSIUM_GOLD),
                            NavigationTab {
                                tab_name: tab_name.to_string(),
                                is_active,
                            },
                        ))
                        .with_children(|tab| {
                            tab.spawn((
                                Text::new(*tab_name),
                                TextFont {
                                    font: font_assets.rajdhani_medium.clone(),
                                    font_size: 27.0, // Scaled up by 1.5x from 18
                                    ..default()
                                },
                                TextColor(if is_active { Color::ELYSIUM_GOLD } else { Color::WHITE.with_alpha(0.7) }),
                            ));
                        });
                    }
                });
                
                // Title section with horizontal line and infinity symbol
                modal.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(120.0), // Scaled up by 1.5x from 80
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(45.0)), // Scaled up by 1.5x from 30
                        ..default()
                    },
                    children![
                        // Main title with line underneath
                        (
                            Node {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            children![
                                // Title text
                                (
                                    Text::new("QUESTS"),
                                    TextFont {
                                        font: font_assets.rajdhani_bold.clone(),
                                        font_size: 54.0, // Scaled up by 1.5x from 36
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    Node {
                                        margin: UiRect::bottom(Val::Px(22.5)), // Scaled up by 1.5x from 15
                                        ..default()
                                    },
                                ),
                                // Horizontal line with infinity symbol underneath
                                (
                                    Node {
                                        width: Val::Px(300.0), // Scaled up by 1.5x from 200
                                        height: Val::Px(3.0), // Scaled up by 1.5x from 2
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        position_type: PositionType::Relative,
                                        ..default()
                                    },
                                    BackgroundColor(Color::ELYSIUM_GOLD),
                                    children![
                                        (
                                            Text::new("∞"),
                                            TextFont {
                                                font_size: 21.0, // Scaled up by 1.5x from 14
                                                ..default()
                                            },
                                            TextColor(Color::ELYSIUM_GOLD),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                padding: UiRect::horizontal(Val::Px(12.0)), // Scaled up by 1.5x from 8
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.08, 0.10, 0.14, 0.95)),
                                        )
                                    ]
                                )
                            ]
                        )
                    ]
                ));
                
                // Quest list container with proper scrollbar - scaled up
                modal.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(525.0), // Scaled up by 1.5x from 350
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(15.0)), // Scaled up by 1.5x from 10
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.07, 0.11, 0.8)),
                    BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.3)),
                    BorderRadius::all(Val::Px(6.0)),
                    children![
                        // Quest entries container - scrollable
                        (
                            Node {
                                width: Val::Px(1230.0), // Scaled up by 1.5x from 820
                                height: Val::Px(495.0), // Scaled up by 1.5x from 330
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(15.0)), // Scaled up by 1.5x from 10
                                overflow: Overflow::scroll_y(), // Enable vertical scrolling
                                ..default()
                            },
                            QuestEntriesContainer,
                            children![
                                // Quest entries will be spawned here dynamically
                            ]
                        ),
                        // Scrollbar
                        (
                            Node {
                                width: Val::Px(9.0), // Scaled up by 1.5x from 6
                                height: Val::Px(495.0), // Scaled up by 1.5x from 330
                                margin: UiRect::right(Val::Px(12.0)), // Scaled up by 1.5x from 8
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                            BorderRadius::all(Val::Px(3.0)),
                        )
                    ]
                ));
            });
        });
}

pub fn update_quest_list(
    mut commands: Commands,
    _objective_manager: Res<ObjectiveManager>,
    font_assets: Option<Res<FontAssets>>,
    ui_assets: Option<Res<UiAssets>>,
    quest_container_query: Query<Entity, With<QuestEntriesContainer>>,
    existing_quests: Query<Entity, With<QuestEntry>>,
) {
    let Some(font_assets) = font_assets else { return; };
    let Some(ui_assets) = ui_assets else { return; };

    let Some(quest_container_entity) = quest_container_query.iter().next() else { return; };

    // Clear existing quest entries
    for entity in existing_quests.iter() {
        commands.entity(entity).despawn();
    }

    // Create 10 quests total - use existing objectives plus additional ones
    let quest_titles = [
        "COLLECT HEALTH POTIONS",
        "FIND SURVIVAL KITS", 
        "GATHER ANCIENT BOOKS",
        "COLLECT GOLDEN COINS",
        "EXPLORE ANCIENT RUINS",
        "DEFEAT DARK CREATURES",
        "RETRIEVE LOST ARTIFACTS",
        "MASTER THE ELEMENTS",
        "UNLOCK HIDDEN PASSAGES",
        "RESTORE THE TEMPLE"
    ];
    
    let quest_descriptions = [
        "Collect 5 health potions scattered throughout the realm.",
        "Find 3 survival kits hidden in the wilderness.",
        "Gather 7 ancient books from the library ruins.",
        "Collect 10 golden coins from fallen enemies.",
        "Explore 4 ancient ruins and discover their secrets.",
        "Defeat 8 dark creatures that roam the shadows.",
        "Retrieve 6 lost artifacts from the depths.",
        "Master the four elements: fire, water, earth, and air.",
        "Unlock 5 hidden passages throughout the realm.",
        "Restore the ancient temple to its former glory."
    ];

    // Spawn 10 quests
    for i in 0..10 {
        let quest_objective = crate::systems::objectives::Objective {
            id: i.to_string(),
            title: quest_titles[i].to_string(),
            description: quest_descriptions[i].to_string(),
            objective_type: "quest".to_string(),
            target: "quest_completion".to_string(),
            required_count: Some(((i + 1) * 2) as u32),
            current_count: if i < 2 { ((i + 1) * 2) as u32 } else { 0 }, // First 2 are completed
            completed: i < 2,
            position: None,
            completion_radius: None,
            reward: format!("{} Gold", (i + 1) * 250),
        };
        
        let quest_entity = spawn_quest_entry(&mut commands, &quest_objective, &font_assets, &ui_assets, i);
        commands.entity(quest_container_entity).add_child(quest_entity);
    }
}

fn spawn_quest_entry(
    commands: &mut Commands,
    objective: &crate::systems::objectives::Objective,
    font_assets: &Res<FontAssets>,
    ui_assets: &Res<UiAssets>,
    index: usize,
) -> Entity {
    let is_active = index < 2; // First two quests are active (lighter background)
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(127.5), // Scaled up by 1.5x from 85
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(12.0)), // Scaled up by 1.5x from 8
            padding: UiRect::all(Val::Px(22.5)), // Scaled up by 1.5x from 15
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(if is_active { 
            Color::srgba(0.12, 0.14, 0.18, 0.8) 
        } else { 
            Color::srgba(0.08, 0.10, 0.14, 0.6) 
        }),
        BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.2)),
        BorderRadius::all(Val::Px(6.0)),
        QuestEntry { quest_id: objective.id.clone() },
        children![
            // Quest icon (coin image)
            (
                Node {
                    width: Val::Px(105.0), // Scaled up by 1.5x from 70
                    height: Val::Px(105.0), // Scaled up by 1.5x from 70
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::right(Val::Px(-50.0)), // Add spacing between icon and text
                    ..default()
                },
                ImageNode {
                    image: ui_assets.coin.clone(),
                    ..default()
                },
                QuestIcon,
            ),
            // Quest info (title and description)
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center, // Center like the reward section
                    align_items: AlignItems::FlexStart,
                    width: Val::Px(675.0), // Scaled up by 1.5x from 450
                    height: Val::Px(97.5), // Scaled up by 1.5x from 65
                    margin: UiRect::right(Val::Px(22.5)), // Scaled up by 1.5x from 15
                    ..default()
                },
                children![
                    // Quest title
                    (
                        Text::new(&objective.title),
                        TextFont {
                            font: font_assets.rajdhani_bold.clone(),
                            font_size: 27.0, // Scaled up by 1.5x from 18
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(9.0)), // Scaled up by 1.5x from 6
                            ..default()
                        },
                    ),
                    // Quest description
                    (
                        Text::new(&objective.description),
                        TextFont {
                            font: font_assets.rajdhani_medium.clone(),
                            font_size: 21.0, // Scaled up by 1.5x from 14
                            ..default()
                        },
                        TextColor(Color::WHITE.with_alpha(0.7)),
                    )
                ]
            ),
            // Quest reward section
            (
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexEnd,
                    width: Val::Px(225.0), // Scaled up by 1.5x from 150
                    height: Val::Px(97.5), // Scaled up by 1.5x from 65
                    ..default()
                },
                children![
                    // Reward label
                    (
                        Text::new("REWARD"),
                        TextFont {
                            font: font_assets.rajdhani_medium.clone(),
                            font_size: 18.0, // Scaled up by 1.5x from 12
                            ..default()
                        },
                        TextColor(Color::WHITE.with_alpha(0.6)),
                        Node {
                            margin: UiRect::bottom(Val::Px(6.0)), // Scaled up by 1.5x from 4
                            ..default()
                        },
                    ),
                    // Reward icon and amount
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            // Reward coin icon
                            (
                                Node {
                                    width: Val::Px(45.0), // Scaled up by 1.5x from 30
                                    height: Val::Px(45.0), // Scaled up by 1.5x from 30
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::right(Val::Px(6.0)), // Scaled up by 1.5x from 4
                                    ..default()
                                },
                                ImageNode {
                                    image: ui_assets.coin.clone(),
                                    ..default()
                                },
                                QuestReward,
                            ),
                            // Reward amount
                            (
                                Text::new(format!("{} Gold", objective.reward)),
                                TextFont {
                                    font: font_assets.rajdhani_medium.clone(),
                                    font_size: 24.0, // Scaled up by 1.5x from 16
                                    ..default()
                                },
                                TextColor(Color::ELYSIUM_GOLD),
                            )
                        ]
                    )
                ]
            )
        ]
    )).id()
}

pub fn toggle_modal_visibility(
    mut modal_state: ResMut<ModalState>,
    mut background_query: Query<&mut Visibility, With<ModalBackground>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Handle Escape key - close modal if open, otherwise let the global handler deal with it
    if keyboard.just_pressed(KeyCode::Escape) {
        if modal_state.visible {
            // Close the modal
            modal_state.visible = false;
            
            for mut visibility in &mut background_query {
                *visibility = Visibility::Hidden;
            }
        }
        // If modal is not visible, let the global ESC handler (ReturnToMainMenu) take over
    }
}

pub fn handle_view_more_click(
    mut modal_state: ResMut<ModalState>,
    mut background_query: Query<&mut Visibility, With<ModalBackground>>,
    mut interaction_query: Query<(&Interaction, &Name), Changed<Interaction>>,
) {
    for (interaction, name) in &mut interaction_query {
        if name.as_str() == "View More Button" {
            match *interaction {
                Interaction::Pressed => {
                    modal_state.visible = true;
                    for mut visibility in &mut background_query {
                        *visibility = Visibility::Visible;
                    }
                }
                Interaction::Hovered => {
                    // No hover effect - removed
                }
                Interaction::None => {
                    // No background color change - removed
                }
            }
        }
    }
}

pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut bevy::ui::ScrollPosition>,
) {
    for event in mouse_wheel_events.read() {
        for mut scroll in &mut query {
            let dy = match event.unit {
                MouseScrollUnit::Line => event.y * 20.0,
                MouseScrollUnit::Pixel => event.y,
            };
            scroll.offset_y -= dy;
        }
    }
}

pub fn despawn_modal(mut commands: Commands, query: Query<Entity, With<ModalBackground>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== MODAL PLUGIN =====

pub struct ModalPlugin;

impl Plugin for ModalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModalState>()
            .add_systems(Update, (
                toggle_modal_visibility,
                handle_view_more_click,
                update_quest_list,
                update_scroll_position,
            ));
    }
} 