use super::{Screen, despawn_scene};
use crate::assets::ModelAssets;
use crate::systems::character_controller::CharacterControllerBundle;
use crate::systems::enemy_ai::EnemyBundle;
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, Friction, GravityScale,
    Restitution, RigidBody, CollisionEventsEnabled,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::Actions;
use bevy_gltf_animation::prelude::GltfSceneRoot;

// ===== PLUGIN SETUP =====

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::FightScene), (spawn_fight_scene, despawn_collectibles))
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
        color: Color::srgb(0.8, 0.7, 0.6), // Warm, golden ambient light
        brightness: 0.3, // Reduced brightness for more natural look
        affects_lightmapped_meshes: false,
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
            illuminance: 15_000.0, // Reduced from 80_000 for softer lighting
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

    // Add a second, softer directional light for fill lighting
    commands.spawn((
        Name::new("Fill Light"),
        DirectionalLight {
            illuminance: 5_000.0, // Much softer fill light
            shadows_enabled: false, // No shadows for fill light
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_6, // Different angle
            std::f32::consts::FRAC_PI_2, // Different angle
            0.0,
        )),
        FightScene,
    ));

    // Add a warm point light for atmospheric lighting
    commands.spawn((
        Name::new("Atmospheric Point Light"),
        PointLight {
            color: Color::srgb(1.0, 0.9, 0.7), // Warm, golden light
            intensity: 800.0, // Moderate intensity
            range: 15.0, // Good range for atmospheric lighting
            radius: 0.5,
            shadows_enabled: false, // No shadows for point light to avoid performance issues
            ..default()
            },
        Transform::from_xyz(0.0, 8.0, 0.0), // Position above the scene
        FightScene,
    ));

    // Add a subtle rim light for depth
    commands.spawn((
        Name::new("Rim Light"),
        PointLight {
            color: Color::srgb(0.7, 0.8, 1.0), // Slight blue tint for contrast
            intensity: 300.0, // Very subtle
            range: 20.0, // Wide range for subtle effect
            radius: 1.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-15.0, 5.0, 0.0), // Position to the side
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

    // Spawn the enemy model with AI and animations
    commands.spawn((
        Name::new("Fight Enemy"),
        GltfSceneRoot::new(assets.enemy.clone()),
        Transform {
            translation: Vec3::new(5.0, -1.65, 0.0),
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            scale: Vec3::splat(4.0),
            ..default()
        },
        EnemyBundle::default(),
        Friction::new(0.5),
        Restitution::new(0.0),
        GravityScale(1.0),
        CollisionEventsEnabled, // Enable collision events
        FightScene,
    )).observe(crate::systems::character_controller::setup_idle_animation);

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
                "1/50",         // level as string
                (300, 300), // time left (5:00 minutes countdown)
                (50, 100), // example xp
                font_assets.rajdhani_bold.clone(),
                crate::ui::widgets::HudPosition::Left,
            ));
            parent.spawn(crate::ui::widgets::player_hud_widget(
                ui_assets.enemy_avatar.clone(),
                "Enemy",
                "3/50",          // level as string
                (300, 300), // time left (5:00 minutes countdown)
                (90, 100),  // example xp
                font_assets.rajdhani_medium.clone(),
                crate::ui::widgets::HudPosition::Right,
            ));
            parent.spawn((
                Text::new("FIGHT SCENE\nPress ESC to return to gameplay\nPress COMMA from gameplay to enter fight"),
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

fn despawn_collectibles(mut commands: Commands, query: Query<Entity, With<crate::systems::collectibles::Collectible>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

// ===== SCENE MARKER =====

#[derive(Component, Default, Clone)]
struct FightScene;
