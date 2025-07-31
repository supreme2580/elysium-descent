use bevy::{picking::*, prelude::*, sprite::Anchor};
use bevy_lunex::*;

use super::{Screen, despawn_scene};
use crate::assets::{FontAssets, UiAssets};
use crate::ui::styles::ElysiumDescentColorPalette;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::MainMenu), (reset_ui_camera, spawn).chain())
        .add_systems(OnExit(Screen::MainMenu), despawn_scene::<MainMenuScene>);
}

#[derive(Component)]
struct MainMenuScene;

/// Reset the UI camera to its proper state when entering main menu
fn reset_ui_camera(
    mut ui_cameras: Query<&mut Transform, (With<Camera2d>, With<bevy_lunex::UiSourceCamera<0>>)>,
) {
    if let Ok(mut camera_transform) = ui_cameras.single_mut() {
        // Reset to the proper UI camera position
        *camera_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 1000.0));

    }
}

fn spawn(mut commands: Commands, ui_assets: Res<UiAssets>, font_assets: Res<FontAssets>) {
    // Create UI
    commands
        .spawn((
            UiLayoutRoot::new_2d(),
            UiFetchFromCamera::<0>,
            MainMenuScene,
        ))
        .with_children(|ui| {
            // Spawn the background
            ui.spawn((
                // You can name your entites for easier debug
                Name::new("Background"),
                UiLayout::solid()
                    .size((1920.0, 1080.0))
                    .scaling(Scaling::Fill)
                    .pack(),
                Sprite::from_image(ui_assets.background.clone()),
            ));

            // Add the panel boundary
            ui.spawn((UiLayout::solid()
                .size((881.0, 1600.0))
                .align_x(-0.74)
                .pack(),))
                .with_children(|ui| {
                    // Spawn the panel
                    ui.spawn((
                        Name::new("Panel"),
                        UiLayout::window()
                            .x(Rl(50.0))
                            .anchor(Anchor::TopCenter)
                            .size(Rl(105.0))
                            .pack(),
                        Sprite::from_image(ui_assets.panel_menu.clone()),
                    ));
                    // Spawn the logo boundary
                    ui.spawn((UiLayout::window()
                        .y(Rl(11.0))
                        .size(Rl((105.0, 20.0)))
                        .pack(),))
                        .with_children(|ui| {
                            // Spawn the logo
                            ui.spawn((
                                Name::new("Logo"),
                                UiLayout::solid().size((1240.0, 381.0)).pack(),
                                Sprite::from_image(ui_assets.title.clone()),
                            ));
                        });

                    ui.spawn((
                        UiLayout::window().pos(Rl((22.0, 33.0))).size(Rl((55.0, 34.0))).pack(),
                    )).with_children(|ui| {

                        // Spawn buttons
                        let gap = 3.0;
                        let size = 14.0;
                        let mut offset = 0.0;
                        for button in ["Continue", "New Game", "Load Game", "Settings", "Credits", "Quit Game"] {

                            // Spawn the button
                            let mut button_entity = ui.spawn((
                                Name::new(button),
                                UiLayout::window().y(Rl(offset)).size(Rl((100.0, size))).pack(),
                                OnHoverSetCursor::new(bevy::window::SystemCursorIcon::Pointer),
                            ));
                            button_entity.with_children(|ui| {
                                // Spawn the image
                                ui.spawn((
                                    // You can define layouts for multiple states
                                    UiLayout::new(vec![
                                        (UiBase::id(), UiLayout::window().full()),
                                        (UiHover::id(), UiLayout::window().x(Rl(10.0)).full())
                                    ]),
                                    // Like this you can enable a state
                                    UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                    // You can specify colors for multiple states
                                    UiColor::new(vec![
                                        (UiBase::id(), Color::ELYSIUM_DESCENT_RED.with_alpha(0.15)),
                                        (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                    ]),
                                    Sprite {
                                        image: ui_assets.button_symmetric.clone(),
                                        // Here we enable sprite slicing
                                        image_mode: SpriteImageMode::Sliced(TextureSlicer { border: BorderRect::all(32.0), ..default() }),
                                        ..default()
                                    },
                                    // Make sure it does not cover the bounding zone of parent
                                    Pickable::IGNORE,
                                )).with_children(|ui| {

                                    // Spawn the text
                                    ui.spawn((
                                        // For text always use window layout to position it
                                        UiLayout::window().pos((Rh(40.0), Rl(50.0))).anchor(Anchor::CenterLeft).pack(),
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                        ]),
                                        UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                        // You can control the size of the text
                                        UiTextSize::from(Rh(60.0)),
                                        // You can attach text like this
                                        Text2d::new(button),
                                        TextFont {
                                            font: font_assets.rajdhani_medium.clone(),
                                            font_size: 64.0,
                                            ..default()
                                        },
                                        // Make sure it does not cover the bounding zone of parent
                                        Pickable::IGNORE,
                                    ));

                                    // Spawn the fluff
                                    ui.spawn((
                                        // For text always use window layout to position it
                                        UiLayout::window().pos(Rl((90.0, 50.0))).anchor(Anchor::CenterRight).pack(),
                                        UiColor::new(vec![
                                            (UiBase::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(0.2)),
                                            (UiHover::id(), Color::ELYSIUM_DESCENT_YELLOW.with_alpha(1.2))
                                        ]),
                                        UiHover::new().forward_speed(20.0).backward_speed(4.0),
                                        // You can control the size of the text
                                        UiTextSize::from(Rh(60.0)),
                                        // You can attach text like this
                                        Text2d::new("<-"),
                                        TextFont {
                                            font: font_assets.rajdhani_bold.clone(),
                                            font_size: 64.0,
                                            ..default()
                                        },
                                    ));
                                });

                            // Enable the transition on hover
                            }).observe(hover_set::<Pointer<Over>, true>).observe(hover_set::<Pointer<Out>, false>);

                            // Assign a functionality to the buttons
                            match button {
                                "New Game" => {
                                    button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                        // Change the state to settings
                                        next.set(Screen::NewGame);
                                    });
                                },
                                "Settings" => {
                                    button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                        // Change the state to settings
                                        next.set(Screen::Settings);
                                    });
                                },
                                "Continue" => {
                                    button_entity.observe(|_: Trigger<Pointer<Click>>, mut next: ResMut<NextState<Screen>>| {
                                        // Change the state to PreGameLoading
                                        next.set(Screen::PreGameLoading);
                                    });
                                },
                                "Quit Game" => {
                                    button_entity.observe(|_: Trigger<Pointer<Click>>, mut exit: EventWriter<AppExit>| {
                                        // Close the app
                                        exit.write(AppExit::Success);
                                    });
                                },
                                _ => {
                                    button_entity.observe(|c_trigger: Trigger<Pointer<Click>>, c_button: Query<NameOrEntity, With<UiLayout>>| {
                                      info!("Clicked: {}", c_button.get(c_trigger.target()).unwrap());
                                    });
                                }
                            }

                            offset += gap + size;
                        }
                    });
                });
        });
}
