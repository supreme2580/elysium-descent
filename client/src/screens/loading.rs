use bevy::prelude::*;

use super::Screen;

#[derive(Component)]
struct LoadingScreen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), setup_loading_screen)
        .add_systems(
            OnExit(Screen::Loading),
            super::despawn_scene::<LoadingScreen>,
        );
}

fn setup_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            LoadingScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
            ));
        });
}
