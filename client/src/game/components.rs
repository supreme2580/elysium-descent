use bevy::prelude::*;

/******************************************************************************
 *                            CORE GAME COMPONENTS                            *
 ******************************************************************************/

#[derive(Component, Reflect)]
pub struct Player;

#[derive(Component, Reflect)]
pub struct Opponent;

#[derive(Component, Reflect)]
pub struct Health {
    pub current: u32,
    pub max: u32,
}

#[derive(Component, Reflect)]
pub struct Fruit;

/******************************************************************************
 *                                   INPUT                                    *
 ******************************************************************************/

#[derive(Component, Default)]
pub struct PlayerInput;

/******************************************************************************
 *                              UI MARKER COMPONENTS                          *
 ******************************************************************************/

// #[derive(Component)]
// pub struct MainMenuButton;

// #[derive(Component)]
// pub struct StartGameButton;

// #[derive(Component)]
// pub struct OptionsButton;

// #[derive(Component)]
// pub struct QuitGameButton; 