use super::{Screen, despawn_scene};
use crate::assets::ModelAssets;
use crate::systems::character_controller::CharacterControllerBundle;
use avian3d::prelude::{
    Collider, ColliderConstructor, ColliderConstructorHierarchy, Friction, GravityScale,
    Restitution, RigidBody, CollisionEventsEnabled,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;
use bevy_gltf_animation::prelude::GltfSceneRoot;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::FightScene), spawn_fight_scene)
        .add_systems(OnExit(Screen::FightScene), despawn_scene::<FightScene>)
        .add_systems(
            Update,
            handle_fight_input.run_if(in_state(Screen::FightScene)),
        )
        .add_systems(
            Update,
            camera_follow_fight_player.run_if(in_state(Screen::FightScene)),
        );
}

// ===== SYSTEMS =====

fn spawn_fight_scene(
    mut commands: Commands,
    assets: Res<ModelAssets>,
    ui_assets: Res<crate::assets::UiAssets>,
    font_assets: Res<crate::assets::FontAssets>,
) {
    // Set up ambient light (match gameplay)
    commands.insert_resource(AmbientLight {
        color: Color::srgb_u8(68, 71, 88),
        brightness: 120.0,
        ..default()
    });

    // Spawn the dungeon model (match gameplay environment)
    commands.spawn((
        Name::new("Fight Dungeon"),
        SceneRoot(assets.dungeon.clone()),
        Transform {
            translation: Vec3::new(0.0, -1.5, 0.0),
            rotation: Quat::from_rotation_y(-core::f32::consts::PI * 0.5),
            scale: Vec3::splat(7.5),
        },
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
        FightScene,
    ));

    // Add directional light (match gameplay)
    commands.spawn((
        Name::new("Directional Light"),
        DirectionalLight {
            illuminance: 80_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_3,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        FightScene,
    ));

    // Spawn the player model (match gameplay)
    commands
        .spawn((
            Name::new("Fight Player"),
            GltfSceneRoot::new(assets.player.clone()),
            Transform {
                translation: Vec3::new(5.0, 2.0, -10.0),
                scale: Vec3::splat(4.0),
                ..default()
            },
            CharacterControllerBundle::new(),
            Friction::new(0.5),
            Restitution::new(0.0),
            GravityScale(1.0),
            CollisionEventsEnabled, // Enable collision events
            Actions::<crate::keybinding::Player>::default(),
            FightScene,
        ))
        .observe(crate::systems::character_controller::setup_idle_animation);

    // Spawn the enemy model at the opposite end (unchanged)
    commands.spawn((
        Name::new("Fight Enemy"),
        SceneRoot(assets.enemy.clone()),
        Transform {
            translation: Vec3::new(5.0, -1.65, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(4.0),
            ..default()
        },
        Collider::capsule(0.5, 1.5),
        RigidBody::Static,
        FightScene,
    ));

    // Add a camera (match gameplay)
    commands.spawn((
        Name::new("Fight Camera"),
        Camera3d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, -12.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        FightScene,
    ));

    // Add a simple UI text to show we're in the fight scene
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            FightScene,
        ))
        .with_children(|parent| {
            // Health bars row
            // Use player_hud_widget for both player and enemy
            parent.spawn(crate::ui::widgets::player_hud_widget(
                ui_assets.player_avatar.clone(),
                "Player",
                2,         // example level
                (80, 100), // example health
                (50, 100), // example xp
                font_assets.rajdhani_bold.clone(),
                crate::ui::widgets::HudPosition::Left,
            ));
            parent.spawn(crate::ui::widgets::player_hud_widget(
                ui_assets.enemy_avatar.clone(),
                "Enemy",
                3,          // example level
                (120, 150), // example health
                (90, 100),  // example xp
                font_assets.rajdhani_medium.clone(),
                crate::ui::widgets::HudPosition::Right,
            ));
            parent.spawn((
                Text::new("FIGHT SCENE\nPress ESC to return to gameplay"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor::WHITE,
                FightScene,
            ));
        });
}

fn handle_fight_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    // Return to gameplay when ESC is pressed
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(Screen::GamePlay);
    }
}

fn camera_follow_fight_player(
    player_query: Query<
        &Transform,
        (
            With<crate::systems::character_controller::CharacterController>,
            With<FightScene>,
        ),
    >,
    mut camera_query: Query<
        &mut Transform,
        (
            With<Camera3d>,
            With<FightScene>,
            Without<crate::systems::character_controller::CharacterController>,
        ),
    >,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            let player_pos = player_transform.translation;
            let player_rotation = player_transform.rotation;

            // Calculate camera position behind player (inverted Z)
            let camera_offset = player_rotation * Vec3::new(0.0, 4.0, -12.0);
            let target_pos = player_pos + camera_offset;

            // Smoothly move camera to new position
            camera_transform.translation = camera_transform
                .translation
                .lerp(target_pos, (5.0 * time.delta_secs()).min(1.0));

            // Make camera look at player
            camera_transform.look_at(player_pos + Vec3::Y * 2.0, Vec3::Y);
        }
    }
}

// ===== SCENE MARKER =====

#[derive(Component, Default, Clone)]
struct FightScene;
