use crate::ui::styles::ElysiumDescentColorPalette;
use bevy::ecs::relationship::{RelatedSpawnerCommands, Relationship};
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

pub fn label_widget(
    window_height: f32,
    font: Handle<Font>,
    text: impl Into<String> + Clone,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Name::new(text.clone().into()),
        Pickable::IGNORE,
        children![(
            Text::new(text.into()),
            TextFont {
                font_size: window_height * 0.04,
                font,
                ..default()
            },
            TextColor::WHITE,
        )],
    )
}

fn volume_display_widget(
    window_height: f32,
    font: Handle<Font>,
    text: impl Into<String> + Clone,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(8.0),
            height: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Name::new(text.clone().into()),
        BorderColor(Color::ELYSIUM_DESCENT_BLUE),
        Pickable::IGNORE,
        BorderRadius::MAX,
        children![(
            Text::new(text.into()),
            TextFont {
                font_size: window_height * 0.03,
                font,
                ..default()
            },
            TextColor::WHITE,
        )],
    )
}

fn button_widget(
    window_height: f32,
    font: Handle<Font>,
    text: impl Into<String> + Clone,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(6.0),
            height: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        Button,
        Name::new(text.clone().into()),
        BackgroundColor(Color::ELYSIUM_DESCENT_RED),
        BorderColor(Color::BLACK),
        BorderRadius::MAX,
        children![(
            Text::new(text.into()),
            TextFont {
                font_size: window_height * 0.05,
                font,
                ..default()
            },
            TextColor(Color::BLACK),
        )],
    )
}

pub(crate) fn volume_widget<R, E, B, M, IL, IR>(
    parent: &mut RelatedSpawnerCommands<'_, R>,
    window_height: f32,
    font: Handle<Font>,
    text: impl Into<String> + Clone,
    volume_value: impl Into<String> + Clone,
    top: f32,
    lower_volume_system: IL,
    raise_volume_system: IR,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    R: Relationship,
    IL: IntoObserverSystem<E, B, M>,
    IR: IntoObserverSystem<E, B, M>,
{
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(40.0),
                top: Val::Percent(top),
                ..default()
            },
            Name::new("Sound settings row"),
            Pickable::IGNORE,
        ))
        .with_children(|content| {
            content.spawn(label_widget(window_height, font.clone(), text));

            content
                .spawn(button_widget(window_height, font.clone(), "-"))
                .observe(lower_volume_system);

            content.spawn((Node {
                margin: UiRect::all(Val::Percent(0.5)),
                ..default()
            },));

            content.spawn((volume_display_widget(
                window_height,
                font.clone(),
                volume_value,
            ),));

            content.spawn((Node {
                margin: UiRect::all(Val::Percent(0.5)),
                ..default()
            },));

            content
                .spawn(button_widget(window_height, font.clone(), "+"))
                .observe(raise_volume_system);
        });
}

pub enum HudPosition {
    Left,
    Right,
}

pub fn player_hud_widget(
    avatar: Handle<Image>,
    name: &str,
    level: u32,
    health: (u32, u32),
    xp: (u32, u32),
    font: Handle<Font>,
    position: HudPosition,
) -> impl Bundle {
    let health_percent = health.0 as f32 / health.1 as f32;
    let xp_percent = xp.0 as f32 / xp.1 as f32;
    let (left, right) = match position {
        HudPosition::Left => (Val::Px(48.0), Val::Auto),
        HudPosition::Right => (Val::Auto, Val::Px(-64.0)),
    };
    (
        Node {
            position_type: PositionType::Absolute,
            left,
            right,
            top: Val::Px(24.0),
            width: Val::Px(400.0),
            height: Val::Px(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            ..default()
        },
        Name::new("Player HUD"),
        children![
            // Avatar and Level
            (
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    margin: UiRect::all(Val::Px(4.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        ImageNode {
                            image: avatar.clone(),
                            ..Default::default()
                        },
                        BorderRadius::all(Val::Px(32.0)),
                    ),
                    (
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-10.0),
                            bottom: Val::Px(-10.0),
                            width: Val::Px(32.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::BLACK),
                        BorderRadius::MAX,
                        children![(
                            Text::new(level.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor::WHITE,
                        )]
                    )
                ]
            ),
            // Name and Bars
            (
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    margin: UiRect::left(Val::Px(20.0)),
                    ..default()
                },
                children![
                    // Name
                    (
                        Text::new(name),
                        TextFont {
                            font: font.clone(),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor::WHITE,
                    ),
                    // Health Bar
                    (
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(24.0),
                            border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::FlexStart,
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        children![
                            (
                                Node {
                                    width: Val::Px(200.0 * health_percent),
                                    height: Val::Px(24.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)), // green
                            ),
                            (
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(200.0),
                                    height: Val::Px(24.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                children![(
                                    Text::new(format!("{}/{}", health.0, health.1)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor::WHITE,
                                )]
                            )
                        ]
                    ),
                    // XP Bar
                    (
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(16.0),
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)), // dark gray
                        children![
                            (
                                Node {
                                    width: Val::Px(200.0 * xp_percent),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.6, 0.2, 0.7)), // purple
                            ),
                            (
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(200.0),
                                    height: Val::Px(16.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                children![(
                                    Text::new("LEVEL UP"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor::WHITE,
                                )]
                            )
                        ]
                    )
                ]
            )
        ],
    )
}
