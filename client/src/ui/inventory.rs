use bevy::prelude::*;

use crate::systems::collectibles::{CollectibleType, NextItemToAdd};
use crate::assets::FontAssets;
use crate::assets::UiAssets;

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
    pub count: usize
}

#[derive(Component)]
pub struct CountText;

#[derive(Resource)]
pub struct InventoryVisibilityState {
    pub visible: bool,
    pub timer: Timer,
}

impl Default for InventoryVisibilityState {
    fn default() -> Self {
        Self {
            visible: false,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        }
    }
}

pub fn spawn_inventory_ui<T: Component + Default>(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(15.0),
            position_type: PositionType::Absolute,
            bottom: Val::Percent(2.0),
            left: Val::Percent(25.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Percent(0.3)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.7, 0.2, 0.7)),
        BorderColor(Color::BLACK.into()),
        InventoryUI,
        Visibility::Hidden,
        T::default(),
    )).with_children(|parent| {
        for i in 0..6 {
            parent.spawn((
                Node {
                    width: Val::Percent(15.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                BorderColor(Color::BLACK.into()),
                InventorySlot { index: i },
            ));

            if i < 5 {
                parent.spawn((
                    Node {
                        margin: UiRect::all(Val::Percent(0.5)),
                        ..default()
                    },
                ));
            }
        }
    });
}

pub fn toggle_inventory_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<InventoryVisibilityState>,
    mut query: Query<&mut Visibility, With<InventoryUI>>,
    time: Res<Time>,
) {
    state.timer.tick(time.delta());

    if keyboard.just_pressed(KeyCode::KeyI) {
        for mut visibility in &mut query {
            *visibility = Visibility::Visible;
        }
        state.visible = true;
        state.timer.reset();
    }

    if state.visible && state.timer.finished() {
        for mut visibility in &mut query {
            *visibility = Visibility::Hidden;
        }
        state.visible = false;
    }
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
    let Some(collectible_type) = collectible_type else { return; };

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
                            if let Ok(grand_grandchildren) = children_query.get(grandchildren[1])
                            {
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
        let is_empty = children_query.get(slot_entity).map_or(true, |c| c.is_empty());
        if is_empty {
            commands.entity(slot_entity).with_children(|parent| {
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Start,
                        ..default()
                    },
                    ZIndex(-1),
                    InventoryItem {
                        item_type: collectible_type.0,
                        count: 1,
                    },
                )).with_children(|item_parent| {
                    // spawn the image
                    item_parent.spawn((
                        Node {
                            width: Val::Percent(120.0),
                            height: Val::Percent(120.0),
                            position_type: PositionType::Absolute,
                            align_content: AlignContent::Center,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            top: Val::Percent(-10.0),
                            left: Val::Percent(-10.0),
                            ..default()
                        },
                        ImageNode {
                            image: match collectible_type.0 {
                                CollectibleType::Coin => ui_assets.coin.clone(),
                                CollectibleType::Book => ui_assets.book.clone(),
                            },
                            ..default()
                        },
                        ZIndex(1),
                    ));
                    
                    // Spawn count text
                    item_parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(25.0),
                            height: Val::Percent(25.0),
                            left: Val::Percent(7.5),
                            top: Val::Percent(10.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BorderRadius::MAX,
                        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
                        ZIndex(2),
                    )).with_children(|text_parent| {
                        text_parent.spawn((
                            TextFont {
                                font_size: 20.0,
                                font: font_assets.rajdhani_extra_bold.clone(),
                                ..default()
                            },
                            Text::new("1"),
                            TextColor(Color::BLACK),
                            CountText,
                        ));
                    });
                });
            });

            commands.remove_resource::<NextItemToAdd>();
            return;
        }
    }
}
