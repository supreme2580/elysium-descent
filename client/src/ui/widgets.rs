use crate::ui::styles::ElysiumDescentColorPalette;
use bevy::ecs::relationship::{RelatedSpawnerCommands, Relationship};
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

pub fn label_widget(
    font_size: f32,
    font: Handle<Font>,
    text: impl Into<String> + Clone,
) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto, // Auto height to fit content
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Name::new(text.clone().into()),
        Pickable::IGNORE,
        children![(
            Text::new(text.into()),
            TextFont {
                font_size,
                font,
                ..default()
            },
            TextColor(Color::srgb(1.0, 1.0, 1.0)), // Bright white for maximum visibility
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
            content.spawn(label_widget(window_height * 0.04, font.clone(), text));

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
    let (left, right, flex_direction) = match position {
        HudPosition::Left => (Val::Px(32.0), Val::Auto, FlexDirection::Row),
        HudPosition::Right => (Val::Auto, Val::Px(32.0), FlexDirection::RowReverse),
    };
    
    (
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
        Name::new("Modern Player HUD"),
        children![
            // Avatar Container with Glow Effect
            (
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
                children![
                    // Avatar Image
                    (
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
                    ),
                    // Level Badge
                    (
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
                        children![(
                            Text::new(level.to_string()),
                            TextFont {
                                font: font.clone(),
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.1, 0.1, 0.1)),
                        )]
                    )
                ]
            ),
            // Stats Container
            (
                Node {
                    width: Val::Px(450.0),
                    height: Val::Px(110.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    margin: UiRect::horizontal(Val::Px(18.0)),
                    ..default()
                },
                children![
                    // Player Name
                    (
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
                    ),
                    // Health Bar Container
                    (
                        Node {
                            width: Val::Px(420.0),
                            height: Val::Px(39.0),
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                        children![
                            // Health Label Row
                            (
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(Val::Px(3.0)),
                                    ..default()
                                },
                                children![
                                    (
                                        Text::new("HEALTH"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 16.5,
                                            ..default()
                                        },
                                        TextColor(Color::HEALTH_GREEN),
                                    ),
                                    (
                                        Text::new(format!("{}/{}", health.0, health.1)),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 16.5,
                                            ..default()
                                        },
                                        TextColor::WHITE,
                                    )
                                ]
                            ),
                            // Health Bar
                            (
                                Node {
                                    width: Val::Px(420.0),
                                    height: Val::Px(21.0),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::DARKER_GLASS),
                                BorderColor(Color::HEALTH_GREEN_DARK.with_alpha(0.6)),
                                BorderRadius::all(Val::Px(10.5)),
                                children![
                                    (
                                        Node {
                                            width: Val::Px(417.0 * health_percent),
                                            height: Val::Px(18.0),
                                            margin: UiRect::all(Val::Px(1.5)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::HEALTH_GREEN),
                                        BorderRadius::all(Val::Px(9.0)),
                                    )
                                ]
                            )
                        ]
                    ),
                    // XP Bar Container
                    (
                        Node {
                            width: Val::Px(420.0),
                            height: Val::Px(33.0),
                            flex_direction: FlexDirection::Column,
                            margin: UiRect::bottom(Val::Px(0.0)),
                            ..default()
                        },
                        children![
                            // XP Label Row
                            (
                                Node {
                                    width: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    margin: UiRect::top(Val::Px(14.0)),
                                    ..default()
                                },
                                children![
                                    (
                                        Text::new("EXPERIENCE"),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.5,
                                            ..default()
                                        },
                                        TextColor(Color::XP_PURPLE),
                                    ),
                                    (
                                        Text::new(format!("{}/{}", xp.0, xp.1)),
                                        TextFont {
                                            font: font.clone(),
                                            font_size: 13.5,
                                            ..default()
                                        },
                                        TextColor::WHITE,
                                    )
                                ]
                            ),
                            // XP Bar
                            (
                                Node {
                                    width: Val::Px(420.0),
                                    height: Val::Px(21.0),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BackgroundColor(Color::DARKER_GLASS),
                                BorderColor(Color::XP_PURPLE_DARK.with_alpha(0.6)),
                                BorderRadius::all(Val::Px(10.5)),
                                children![
                                    (
                                        Node {
                                            width: Val::Px(417.0 * xp_percent),
                                            height: Val::Px(18.0),
                                            margin: UiRect::all(Val::Px(1.5)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::XP_PURPLE),
                                        BorderRadius::all(Val::Px(9.0)),
                                    )
                                ]
                            )
                        ]
                    )
                ]
            )
        ],
    )
}
