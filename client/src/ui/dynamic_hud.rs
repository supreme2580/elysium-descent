use bevy::prelude::*;
use crate::game::resources::{GameTimer, PlayerProgression};
use crate::assets::{FontAssets, UiAssets};
use crate::ui::styles::ElysiumDescentColorPalette;
use crate::ui::widgets::HudPosition;
use crate::systems::timer::{TimerText, LevelText, XpText, TimerBar, XpBar};


#[derive(Component)]
pub struct DynamicPlayerHud;

pub fn spawn_dynamic_player_hud(
    commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    ui_assets: &Res<UiAssets>,
    timer: &Res<GameTimer>,
    progression: &Res<PlayerProgression>,
    position: HudPosition,
) -> Entity {
    let avatar = ui_assets.player_avatar.clone();
    let name = "0XJEHU";
    let font = font_assets.rajdhani_bold.clone();
    
    let time_percent = timer.get_time_percentage();
    let (current_level, max_level) = progression.get_xp_for_display();
    let xp_percent = current_level as f32 / max_level as f32;
    
    let (left, right, flex_direction) = match position {
        HudPosition::Left => (Val::Px(32.0), Val::Auto, FlexDirection::Row),
        HudPosition::Right => (Val::Auto, Val::Px(32.0), FlexDirection::RowReverse),
    };
    
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left,
            right,
            top: Val::Px(32.0),
            width: Val::Px(630.0),
            height: Val::Px(180.0),
            flex_direction,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(24.0)),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::DARK_GLASS),
        BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.6)),
        BorderRadius::all(Val::Px(24.0)),
        Name::new("Dynamic Player HUD"),
        DynamicPlayerHud,
    )).with_children(|parent| {
        // Avatar Container with Glow Effect
        parent.spawn((
            Node {
                width: Val::Px(120.0),
                height: Val::Px(120.0),
                margin: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(4.5)),
                ..default()
            },
            BorderColor(Color::ELYSIUM_GOLD.with_alpha(0.8)),
            BorderRadius::all(Val::Px(63.0)),
            BackgroundColor(Color::DARKER_GLASS),
        )).with_children(|parent| {
            // Avatar Image
            parent.spawn((
                ImageNode {
                    image: avatar.clone(),
                    ..Default::default()
                },
                Node {
                    width: Val::Px(105.0),
                    height: Val::Px(105.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(52.5)),
            ));
            
            // Level Badge
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(-12.0),
                    bottom: Val::Px(-12.0),
                    width: Val::Px(54.0),
                    height: Val::Px(54.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::ELYSIUM_GOLD),
                BorderColor(Color::ELYSIUM_GOLD_DIM),
                BorderRadius::MAX,
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(progression.current_level.to_string()),
                    TextFont {
                        font: font.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.1, 0.1, 0.1)),
                    LevelText,
                ));
            });
        });
        
        // Stats Container
        parent.spawn((
            Node {
                width: Val::Px(450.0),
                height: Val::Px(110.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                margin: UiRect::horizontal(Val::Px(18.0)),
                ..default()
            },
        )).with_children(|parent| {
            // Player Name
            parent.spawn((
                Text::new(name),
                TextFont {
                    font: font.clone(),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::ELYSIUM_GOLD),
                Node {
                    margin: UiRect::top(Val::Px(-14.0)),
                    ..default()
                },
            ));
            
            // Time Left Bar Container
            parent.spawn((
                Node {
                    width: Val::Px(420.0),
                    height: Val::Px(39.0),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            )).with_children(|parent| {
                // Time Left Label Row
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(3.0)),
                        ..default()
                    },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("TIME LEFT"),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.5,
                            ..default()
                        },
                        TextColor(Color::ENERGY_BLUE),
                    ));
                    parent.spawn((
                        Text::new(timer.get_time_remaining_formatted()),
                        TextFont {
                            font: font.clone(),
                            font_size: 16.5,
                            ..default()
                        },
                        TextColor::WHITE,
                        TimerText,
                    ));
                });
                
                // Time Left Bar
                parent.spawn((
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(21.0),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BackgroundColor(Color::DARKER_GLASS),
                    BorderColor(Color::ENERGY_BLUE_DARK.with_alpha(0.6)),
                    BorderRadius::all(Val::Px(10.5)),
                )).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(417.0 * time_percent),
                            height: Val::Px(18.0),
                            margin: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::ENERGY_BLUE),
                        BorderRadius::all(Val::Px(9.0)),
                        TimerBar,
                    ));
                });
            });
            
            // XP Bar Container
            parent.spawn((
                Node {
                    width: Val::Px(420.0),
                    height: Val::Px(33.0),
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::bottom(Val::Px(0.0)),
                    ..default()
                },
            )).with_children(|parent| {
                // XP Label Row
                parent.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(14.0)),
                        ..default()
                    },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new("XP"),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.5,
                            ..default()
                        },
                        TextColor(Color::XP_PURPLE),
                    ));
                    parent.spawn((
                        Text::new(format!("{}/{}", current_level, max_level)),
                        TextFont {
                            font: font.clone(),
                            font_size: 13.5,
                            ..default()
                        },
                        TextColor::WHITE,
                        XpText,
                    ));
                });
                
                // XP Bar
                parent.spawn((
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(21.0),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BackgroundColor(Color::DARKER_GLASS),
                    BorderColor(Color::XP_PURPLE_DARK.with_alpha(0.6)),
                    BorderRadius::all(Val::Px(10.5)),
                )).with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(417.0 * xp_percent),
                            height: Val::Px(18.0),
                            margin: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BackgroundColor(Color::XP_PURPLE),
                        BorderRadius::all(Val::Px(9.0)),
                        XpBar,
                    ));
                });
            });
        });
    }).id()
}
