// mod gameplay;
mod loading;
mod main_menu;
mod settings;

use bevy::prelude::*;

pub fn despawn_scene<S: Component>(mut commands: Commands, query: Query<Entity, With<S>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
pub(crate) enum Screen {
    #[default]
    Loading,
    MainMenu,
    GamePlay,
    NewGame,
    Settings,
}

#[derive(Resource, Default)]
pub struct MainTrack;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .add_systems(Startup, initial_state_setup)
        .add_plugins((main_menu::plugin, settings::plugin, loading::plugin));
}

fn initial_state_setup(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Loading);
}
