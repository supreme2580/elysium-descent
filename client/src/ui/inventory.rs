use bevy::prelude::*;

use crate::assets::FontAssets;
use crate::assets::UiAssets;
use crate::systems::collectibles::{CollectibleType, NextItemToAdd};

// Inventory UI marker
#[derive(Component)]
pub struct InventoryUI;

#[derive(Component)]
pub struct InventorySlot {
    pub index: usize,
}

#[derive(Component)]
pub struct InventoryItem {
    pub item_type: CollectibleType,
    pub count: usize,
}

#[derive(Component)]
pub struct CountText;

#[derive(Resource)]
pub struct InventoryVisibilityState {
    pub visible: bool,
    pub timer: Timer,
    pub shifted_up: bool, // Track if inventory is shifted up due to dialog
}

impl Default for InventoryVisibilityState {
    fn default() -> Self {
        Self {
            visible: true,  // Always visible by default
            timer: Timer::from_seconds(2.0, TimerMode::Once),
            shifted_up: false,
        }
    }
}

/// System to adjust inventory position when dialogs are visible to prevent overlap
pub fn adjust_inventory_for_dialogs(
    dialog_query: Query<&Visibility, With<crate::ui::dialog::Dialog>>,
    mut inventory_query: Query<&mut Node, (With<InventoryUI>, Without<crate::ui::dialog::Dialog>)>,
    mut visibility_state: ResMut<InventoryVisibilityState>,
    windows: Query<&Window>,
) {
    // Check if any dialog is visible
    let dialog_visible = dialog_query
        .iter()
        .any(|visibility| *visibility == Visibility::Visible);

    if let Ok(mut node) = inventory_query.single_mut() {
        if dialog_visible && !visibility_state.shifted_up {
            // Determine position based on screen height
            let screen_height = windows.single().expect("No window found").height();
            let inventory_padding = if screen_height > 1080.0 {
                200.0 // Fullscreen mode - higher position
            } else {
                0.0 // Windowed mode - lower position
            };
            
            let final_position = 200.0 + inventory_padding;
            node.bottom = Val::Px(final_position);
            visibility_state.shifted_up = true;
        } else if !dialog_visible && visibility_state.shifted_up {
            // Restore original position when dialog is hidden
            node.bottom = Val::Px(32.0); // Back to original position
            visibility_state.shifted_up = false;
        }
    }
}

pub fn spawn_inventory_ui<T: Component + Default>(commands: &mut Commands) {
    use crate::ui::styles::ElysiumDescentColorPalette;
    commands
        .spawn((
            Node {
                width: Val::Px(833.0),
                height: Val::Px(167.0),
                // Remove height so it fits children
                position_type: PositionType::Absolute,
                bottom: Val::Px(32.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-416.5)), // Center horizontally
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.14, 0.18, 0.85)), // glassy dark
            BorderColor(Color::ELYSIUM_GOLD),
            BorderRadius::all(Val::Px(21.0)),
            InventoryUI,
            Visibility::Visible,  // Start visible
            T::default(),
        ))
        .with_children(|parent| {
            for i in 0..6 {
                parent.spawn((
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(133.0),
                        margin: UiRect::all(Val::Px(8.0)),
                        padding: UiRect::horizontal(Val::Px(32.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.18, 0.20, 0.26, 0.65)),
                    BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.7)),
                    BorderRadius::all(Val::Px(12.0)),
                    InventorySlot { index: i },
                ))
                .with_children(|slot| {
                    // Always add a filler node to ensure consistent sizing
                    slot.spawn((
                        Node {
                            width: Val::Px(93.0),
                            height: Val::Px(107.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        ZIndex(-2),
                    ));
                });
            }
        });
}

pub fn toggle_inventory_visibility(
    _keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<InventoryVisibilityState>,
    mut query: Query<&mut Visibility, With<InventoryUI>>,
    _time: Res<Time>,
) {
    // Keep inventory always visible - no toggling or hiding
    for mut visibility in &mut query {
        *visibility = Visibility::Visible;
    }
    state.visible = true;
}

pub fn add_item_to_inventory(
    mut commands: Commands,
    mut slot_query: Query<(Entity, &InventorySlot)>,
    children_query: Query<&Children>,
    mut item_query: Query<&mut InventoryItem>,
    mut text_query: Query<&mut Text>,
    font_assets: Res<FontAssets>,
    ui_assets: Res<UiAssets>,
    collectible_type: Option<Res<NextItemToAdd>>,
    mut visibility_state: ResMut<InventoryVisibilityState>,
    mut ui_query: Query<&mut Visibility, With<InventoryUI>>,
) {
    let Some(collectible_type) = collectible_type else {
        return;
    };

    if let Ok(mut visibility) = ui_query.single_mut() {
        *visibility = Visibility::Visible;
        visibility_state.visible = true;
        visibility_state.timer.reset();
    }

    // First: look for a slot that already has this item
    for (slot_entity, _) in slot_query.iter_mut() {
        if let Ok(children) = children_query.get(slot_entity) {
            for child in children.iter() {
                if let Ok(mut item) = item_query.get_mut(child) {
                    if item.item_type == collectible_type.0 {
                        // Found matching item → increase count
                        item.count += 1;

                        // Now do some iterations to update the text count
                        if let Ok(grandchildren) = children_query.get(child) {
                            if let Ok(grand_grandchildren) = children_query.get(grandchildren[1]) {
                                if let Some(&leaf) = grand_grandchildren.first() {
                                    if let Ok(mut text) = text_query.get_mut(leaf) {
                                        text.clear();
                                        text.push_str(&item.count.to_string());
                                    }
                                }
                            }
                        };

                        commands.remove_resource::<NextItemToAdd>();
                        return;
                    }
                }
            }
        }
    }

    // Otherwise: find first empty slot and add the item
    let mut sorted_slots: Vec<(Entity, &InventorySlot)> = slot_query.iter_mut().collect();
    sorted_slots.sort_by_key(|(_, slot)| slot.index);
    for (slot_entity, _) in sorted_slots {
        // Check if slot has any InventoryItem children (not just if it has any children)
        let has_item = children_query
            .get(slot_entity)
            .map_or(false, |children| {
                children.iter().any(|child| item_query.get(child).is_ok())
            });
        
        if !has_item {
            commands.entity(slot_entity).with_children(|parent| {
                parent
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        ZIndex(-1),
                        InventoryItem {
                            item_type: collectible_type.0,
                            count: 1,
                        },
                    ))
                    .with_children(|item_parent| {
                        // spawn the image (larger, centered)
                        item_parent.spawn((
                            Node {
                                width: Val::Px(133.0),
                                height: Val::Px(133.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ImageNode {
                                image: ui_assets.coin.clone(),
                                ..default()
                            },
                            ZIndex(1),
                        ));

                        // Spawn count text (perfect circle badge, bottom right of image)
                        item_parent
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(33.0),  // Slightly smaller for better proportion
                                    height: Val::Px(33.0), // Same as width for circle
                                    right: Val::Px(-20.0),  // Positioned relative to image edge
                                    bottom: Val::Px(13.0), // Positioned relative to image edge
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(Val::Px(1.0)), // Add border for definition
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(16.5)), // Half of width/height for perfect circle
                                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.4)), // Slightly darker border
                                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.98)), // More opaque
                                ZIndex(2),
                            ))
                            .with_children(|text_parent| {
                                text_parent.spawn((
                                    TextFont {
                                        font_size: 19.0, // Adjusted for smaller badge
                                        font: font_assets.rajdhani_extra_bold.clone(),
                                        ..default()
                                    },
                                    Text::new("1"),
                                    TextColor(Color::srgb(0.1, 0.1, 0.1)), // Dark gray instead of pure black
                                ));
                            });
                    });
            });

            commands.remove_resource::<NextItemToAdd>();
            return;
        }
    }
}
