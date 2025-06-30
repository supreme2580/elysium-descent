use bevy::prelude::*;
use bevy::ecs::system::IntoObserverSystem;
use bevy::ecs::relationship::{RelatedSpawnerCommands, Relationship};
use crate::ui::styles::ElysiumDescentColorPalette;

pub fn label_widget(window_height: f32, font: Handle<Font>, text: impl Into<String> + Clone) -> impl Bundle {
    (
        Node {
            width: Val::Percent(50.0),
            height: Val::Percent(20.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Name::new(text.clone().into()),
        Pickable::IGNORE,
        children![
            (
                Text::new(text.into()),
                TextFont {
                    font_size: window_height * 0.04,
                    font,
                    ..default()
                },
                TextColor::WHITE,
            )
        ],
    )
}

fn volume_display_widget(window_height: f32, font: Handle<Font>, text: impl Into<String> + Clone) -> impl Bundle {
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
        children![
            (
                Text::new(text.into()),
                TextFont {
                    font_size: window_height * 0.03,
                    font,
                    ..default()
                },
                TextColor::WHITE,
            )
        ],
    )
}

fn button_widget(window_height: f32, font: Handle<Font>, text: impl Into<String> + Clone) -> impl Bundle {
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
        children![
            (
                Text::new(text.into()),
                TextFont {
                    font_size: window_height * 0.05,
                    font,
                    ..default()
                },
                TextColor(Color::BLACK),
            )
        ],
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

    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(40.0),
            top: Val::Percent(top),
            ..default()
        },
        Name::new("Sound settings row"),
        Pickable::IGNORE,
    )).with_children(|content| {
        content.spawn(
            label_widget(
                window_height,
                font.clone(), 
                text
            )
        );

        content.spawn(
            button_widget(
                window_height,
                font.clone(), 
                "-"
            )
        ).observe(lower_volume_system);

        content.spawn((
            Node {
                margin: UiRect::all(Val::Percent(0.5)),
                ..default()
            },
        ));

        content.spawn((
            volume_display_widget(
                window_height,
                font.clone(), 
                volume_value
            ),
        ));

        content.spawn((
            Node {
                margin: UiRect::all(Val::Percent(0.5)),
                ..default()
            },
        ));

        content.spawn(
            button_widget(
                window_height,
                font.clone(), 
                "+"
            )
        ).observe(raise_volume_system);
    });
}
