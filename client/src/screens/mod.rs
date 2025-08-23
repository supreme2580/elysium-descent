pub mod fight;
pub mod gameplay;
pub mod loading;
pub mod main_menu;
pub mod pregame_loading;  // Add new module
pub mod settings;

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
    PreGameLoading,  // New loading state before gameplay
    GamePlay,
    NewGame,
    Settings,
    FightScene,
}

#[derive(Resource, Default)]
pub struct MainTrack;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .add_systems(Startup, initial_state_setup)
        .add_systems(Update, handle_new_game_transition)
        .add_plugins((
            main_menu::plugin,
            settings::plugin,
            loading::plugin,
            pregame_loading::plugin,  // Add new plugin
            gameplay::plugin,
            fight::plugin,
        ));
}

fn initial_state_setup(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Loading);
}

fn handle_new_game_transition(
    mut next_state: ResMut<NextState<Screen>>,
    current_state: Res<State<Screen>>,
) {
    if current_state.get() == &Screen::NewGame {
        next_state.set(Screen::PreGameLoading);  // Go to PreGameLoading instead of GamePlay
    }
}
