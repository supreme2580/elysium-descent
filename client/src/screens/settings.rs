use bevy::window::WindowResized;
use bevy::{prelude::*, sprite::Anchor};
use bevy_lunex::*;

use super::{MainTrack, Screen};
use crate::assets::{FontAssets, UiAssets};
use crate::audio::AudioSettings;
use crate::ui::styles::ElysiumDescentColorPalette;
use crate::ui::widgets::volume_widget;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    Controls,
    Sound,
    Graphics,
    Window,
}

#[derive(Resource, Default)]
struct SelectedTab(SettingsTab);

impl Default for SettingsTab {
    fn default() -> Self {
        SettingsTab::Controls
    }
}

#[derive(Resource, Default)]
struct LastRenderedTab(Option<SettingsTab>);

#[derive(Component)]
struct TabContentRoot;

#[derive(Component)]
struct SettingsContentRoot;

#[derive(Component)]
struct TabLabel(SettingsTab);

fn update_tab_colors(current: SettingsTab, mut query: Query<(&TabLabel, &mut UiColor)>) {
    for (label, mut color) in &mut query {
        color.insert(
            UiBase::id(),
            (if label.0 == current {
                Color::WHITE
            } else {
                Color::ELYSIUM_DESCENT_RED
            })
            .into(),
        );
    }
}

fn render_sound_settings(
    _: Trigger<Pointer<Click>>,
    mut selected_tab: ResMut<SelectedTab>,
    query: Query<(&TabLabel, &mut UiColor)>,
) {
    selected_tab.0 = SettingsTab::Sound;
    update_tab_colors(SettingsTab::Sound, query);
}

fn render_controls_settings(
    _: Trigger<Pointer<Click>>,
    mut selected_tab: ResMut<SelectedTab>,
    query: Query<(&TabLabel, &mut UiColor)>,
) {
    selected_tab.0 = SettingsTab::Controls;
    update_tab_colors(SettingsTab::Controls, query);
}

fn render_graphics_settings(
    _: Trigger<Pointer<Click>>,
    mut selected_tab: ResMut<SelectedTab>,
    query: Query<(&TabLabel, &mut UiColor)>,
) {
    selected_tab.0 = SettingsTab::Graphics;
    update_tab_colors(SettingsTab::Graphics, query);
}

fn render_windows_settings(
    _: Trigger<Pointer<Click>>,
    mut selected_tab: ResMut<SelectedTab>,
    query: Query<(&TabLabel, &mut UiColor)>,
) {
    selected_tab.0 = SettingsTab::Window;
    update_tab_colors(SettingsTab::Window, query);
}

fn lower_master_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.master_volume = (settings.master_volume - 0.1).clamp(0.0, 1.0);
    info!("Lowering master volume");
}

fn raise_master_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.master_volume = (settings.master_volume + 0.1).clamp(0.0, 1.0);
    info!("Raise master volume");
}

fn lower_music_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.music_volume = (settings.music_volume - 0.1).clamp(0.0, 1.0);
    info!("Lowering music volume");
}

fn raise_music_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.music_volume = (settings.music_volume + 0.1).clamp(0.0, 1.0);
    info!("Raise music volume");
}

fn lower_sfx_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.sfx_volume = (settings.sfx_volume - 0.1).clamp(0.0, 1.0);
    info!("Lowering sfx volume");
}

fn raise_sfx_volume(_: Trigger<Pointer<Click>>, mut settings: ResMut<AudioSettings>) {
    settings.sfx_volume = (settings.sfx_volume + 0.1).clamp(0.0, 1.0);
    info!("Raise sfx volume");
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Settings), SettingsScene::spawn)
        .add_systems(
            OnExit(Screen::Settings),
            super::despawn_scene::<SettingsScene>,
        )
        .add_systems(
            Update,
            render_tab_content.run_if(in_state(Screen::Settings)),
        )
        .insert_resource(SelectedTab::default())
        .insert_resource(LastRenderedTab::default())
        .init_resource::<MainTrack>();
}

// ===== SYSTEMS =====

fn render_tab_content(
    selected_tab: Res<SelectedTab>,
    mut last_rendered: ResMut<LastRenderedTab>,
    mut commands: Commands,
    content_query: Query<Entity, With<TabContentRoot>>,
    parent_query: Query<Entity, With<SettingsContentRoot>>,
    font_assets: Res<FontAssets>,
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    last_height: Local<Option<f32>>,
    audio_settings: Res<AudioSettings>,
) {
    let window = windows.single().unwrap();
    let window_height = window.height();

    // Detect if the window height changed
    let resized = resize_events.read().next().is_some() && Some(window_height) != *last_height;

    let tab_changed = last_rendered.0 != Some(selected_tab.0);

    let settings_changed = audio_settings.is_changed();

    if tab_changed || resized || settings_changed {
        last_rendered.0 = Some(selected_tab.0);

        // Remove old tab content
        for entity in &content_query {
            commands.entity(entity).despawn();
        }

        let window = windows.single();
        let window_height = window.unwrap().height();

        // Get parent container for tab content
        if let Some(parent) = parent_query.iter().next() {
            commands.entity(parent).with_children(|parent| {
                // Spawn a wrapper node for the new content
                parent
                    .spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(90.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::BLACK.with_alpha(0.5)),
                        TabContentRoot,
                    ))
                    .with_children(|content| {
                        match selected_tab.0 {
                            SettingsTab::Sound => {
                                // Spawn the button
                                content
                                    .spawn((Node {
                                        position_type: PositionType::Absolute,
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },))
                                    .with_children(|content| {
                                        volume_widget(
                                            content,
                                            window_height,
                                            font_assets.rajdhani_medium.clone(),
                                            "Master Volume",
                                            ((audio_settings.master_volume * 10.0).floor() as i32)
                                                .to_string(),
                                            30.0,
                                            lower_master_volume,
                                            raise_master_volume,
                                        );

                                        volume_widget(
                                            content,
                                            window_height,
                                            font_assets.rajdhani_medium.clone(),
                                            "Music Volume",
                                            ((audio_settings.music_volume * 10.0).floor() as i32)
                                                .to_string(),
                                            50.0,
                                            lower_music_volume,
                                            raise_music_volume,
                                        );

                                        volume_widget(
                                            content,
                                            window_height,
                                            font_assets.rajdhani_medium.clone(),
                                            "SFX Volume",
                                            ((audio_settings.sfx_volume * 10.0).floor() as i32)
                                                .to_string(),
                                            70.0,
                                            lower_sfx_volume,
                                            raise_sfx_volume,
                                        );
                                    });
                            }
                            SettingsTab::Controls => {}
                            SettingsTab::Graphics => {}
                            SettingsTab::Window => {}
                        }
                    });
            });
        }
    }
}

// ===== RESOURCES & COMPONENTS =====

#[derive(Component)]
struct SettingsScene;

// ===== SETTINGS SCENE IMPLEMENTATION =====

impl SettingsScene {
    fn spawn(mut commands: Commands, ui_assets: Res<UiAssets>, font_assets: Res<FontAssets>) {
        // Create UI
        commands
            .spawn((
                UiLayoutRoot::new_2d(),
                // Make the UI synchronized with camera viewport size
                UiFetchFromCamera::<0>,
                // A scene marker for later mass scene despawn, not UI related
                SettingsScene,
            ))
            .with_children(|ui| {
                // Spawn the background
                ui.spawn((
                    Name::new("Background"),
                    UiLayout::solid()
                        .size((1920.0, 1080.0))
                        .scaling(Scaling::Fill)
                        .pack(),
                    Sprite::from_image(ui_assets.background.clone()),
                ));

                // Spawn the settings content
                ui.spawn((UiLayout::solid().pack(),)).with_children(|ui| {
                    // Spawn the tab bar
                    ui.spawn((
                        UiLayout::window().size(Rl((100.0, 8.0))).pack(),
                        Pickable::IGNORE,
                    ))
                    .with_children(|ui| {
                        // Spawn left chevron
                        ui.spawn((
                            Name::new("Chevron Left"),
                            UiLayout::window()
                                .pos(Rl((5.0, 50.0)))
                                .anchor(Anchor::Center)
                                .size(Rh(35.0))
                                .pack(),
                            Sprite::from_image(ui_assets.chevron_left.clone()),
                            UiHover::new().instant(true),
                            UiColor::new(vec![
                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2)),
                            ]),
                        ))
                        .observe(hover_set::<Pointer<Over>, true>)
                        .observe(hover_set::<Pointer<Out>, false>);

                        // Spawn right chevron
                        ui.spawn((
                            Name::new("Chevron Right"),
                            UiLayout::window()
                                .pos(Rl((95.0, 50.0)))
                                .anchor(Anchor::Center)
                                .size(Rh(35.0))
                                .pack(),
                            Sprite::from_image(ui_assets.chevron_right.clone()),
                            UiHover::new().instant(true),
                            UiColor::new(vec![
                                (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                (UiHover::id(), Color::ELYSIUM_DESCENT_BLUE.with_alpha(1.2)),
                            ]),
                        ))
                        .observe(hover_set::<Pointer<Over>, true>)
                        .observe(hover_set::<Pointer<Out>, false>);

                        // Spawn the control bar
                        ui.spawn((UiLayout::window()
                            .x(Rl(10.0))
                            .size(Rl((80.0, 100.0)))
                            .pack(),))
                            .with_children(|ui| {
                                let categories = ["Controls", "Sound", "Graphics", "Window"];
                                let categories_actions = [
                                    render_controls_settings,
                                    render_sound_settings,
                                    render_graphics_settings,
                                    render_windows_settings,
                                ];
                                let pos = 100.0 / categories.len() as f32;
                                for (i, category) in categories.into_iter().enumerate() {
                                    let tab_enum = match category {
                                        "Controls" => SettingsTab::Controls,
                                        "Sound" => SettingsTab::Sound,
                                        "Graphics" => SettingsTab::Graphics,
                                        "Window" => SettingsTab::Window,
                                        _ => continue,
                                    };

                                    // Spawn the button
                                    ui.spawn((
                                        Name::new(category),
                                        UiLayout::window()
                                            .x(Rl(pos * i as f32))
                                            .size(Rl((pos, 100.0)))
                                            .pack(),
                                    ))
                                    .with_children(|ui| {
                                        // Spawn the background
                                        ui.spawn((
                                            UiLayout::window()
                                                .full()
                                                .y(Rl(10.0))
                                                .height(Rl(80.0))
                                                .pack(),
                                            UiHover::new().forward_speed(20.0).backward_speed(5.0),
                                            UiColor::new(vec![
                                                (
                                                    UiBase::id(),
                                                    Color::ELYSIUM_DESCENT_RED.with_alpha(0.0),
                                                ),
                                                (
                                                    UiHover::id(),
                                                    Color::ELYSIUM_DESCENT_RED.with_alpha(0.4),
                                                ),
                                            ]),
                                            Sprite {
                                                image: ui_assets.button_symmetric.clone(),
                                                image_mode: SpriteImageMode::Sliced(
                                                    TextureSlicer {
                                                        border: BorderRect::all(32.0),
                                                        ..default()
                                                    },
                                                ),
                                                ..default()
                                            },
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(
                                            |ui| {
                                                // Spawn the text
                                                ui.spawn((
                                                    UiLayout::window()
                                                        .pos(Rl(50.0))
                                                        .anchor(Anchor::Center)
                                                        .pack(),
                                                    UiColor::new(vec![
                                                        (UiBase::id(), Color::ELYSIUM_DESCENT_RED),
                                                        (
                                                            UiHover::id(),
                                                            Color::ELYSIUM_DESCENT_BLUE
                                                                .with_alpha(1.2),
                                                        ),
                                                    ]),
                                                    UiHover::new().instant(true),
                                                    UiTextSize::from(Rh(50.0)),
                                                    Text2d::new(category.to_ascii_uppercase()),
                                                    TextFont {
                                                        font: font_assets.rajdhani_medium.clone(),
                                                        font_size: 64.0,
                                                        ..default()
                                                    },
                                                    TabLabel(tab_enum),
                                                    Pickable::IGNORE,
                                                ));
                                            },
                                        );

                                        // Add the observers
                                    })
                                    .observe(hover_set::<Pointer<Over>, true>)
                                    .observe(hover_set::<Pointer<Out>, false>)
                                    .observe(categories_actions[i]);
                                }
                            });
                    });
                });
            });

        // The Bevy UI nodes must be here to work
        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(70.),
                top: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            // Scene markers
            SettingsScene,
            SettingsContentRoot,
        ));
    }
}
