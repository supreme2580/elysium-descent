use bevy::prelude::*;

use super::{Screen, despawn_scene};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GamePlay), spawn)
        .add_systems(OnExit(Screen::GamePlay), despawn_scene::<GameplayScene>);
}

#[derive(Component)]
struct GameplayScene;

fn spawn() {}
