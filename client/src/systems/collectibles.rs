use crate::assets::ModelAssets;
use avian3d::prelude::*;
use bevy::prelude::*;
use std::sync::Arc;

use crate::systems::character_controller::CharacterController;

// ===== COMPONENTS & RESOURCES =====

#[derive(Resource)]
pub struct CollectibleCounter {
    pub collectibles_collected: u32,
}

#[derive(Component)]
pub struct Collectible {
    pub on_collect: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

#[derive(Component, Clone)]
pub struct CollectibleRotation {
    pub enabled: bool,
    pub clockwise: bool,
    pub speed: f32,
}

#[derive(Component)]
pub struct FloatingItem {
    pub base_height: f32,
    pub hover_amplitude: f32,
    pub hover_speed: f32,
}

#[derive(Component, Clone, Copy, Debug)]
pub enum CollectibleType {
    Book,
    FirstAidKit,
}

#[derive(Component)]
pub struct Sensor;

// Configuration for spawning collectibles
#[derive(Clone)]
pub struct CollectibleConfig {
    pub position: Vec3,
    pub collectible_type: CollectibleType,
    pub scale: f32,
    pub rotation: Option<CollectibleRotation>,
    pub on_collect: Arc<dyn Fn(&mut Commands, Entity) + Send + Sync>,
}

// ===== PLUGIN =====

pub struct CollectiblesPlugin;

impl Plugin for CollectiblesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CollectibleCounter {
            collectibles_collected: 0,
        })
        .add_systems(
            Update,
            (collect_items, update_floating_items, rotate_collectibles),
        );
    }
}

// ===== SYSTEMS =====

pub fn spawn_collectible(
    commands: &mut Commands,
    assets: &Res<ModelAssets>,
    config: CollectibleConfig,
) {
    let model_handle = match config.collectible_type {
        CollectibleType::Book => assets.book.clone(),
        CollectibleType::FirstAidKit => assets.first_aid_kit.clone(),
    };

    let mut entity = commands.spawn((
        Name::new(format!("{:?}", config.collectible_type)),
        SceneRoot(model_handle),
        Transform {
            translation: config.position,
            scale: Vec3::splat(config.scale),
            ..default()
        },
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Kinematic,
        Friction::new(0.5),
        Restitution::new(0.0),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Collectible {
            on_collect: config.on_collect,
        },
        config.collectible_type,
        FloatingItem {
            base_height: config.position.y,
            hover_amplitude: 0.2,
            hover_speed: 2.0,
        },
        Sensor,
    ));

    if let Some(rotation) = config.rotation {
        entity.insert(rotation);
    }
}

fn collect_items(
    mut commands: Commands,
    mut collectible_counter: ResMut<CollectibleCounter>,
    player_query: Query<&Transform, With<CharacterController>>,
    collectible_query: Query<(Entity, &Transform, &CollectibleType, &Collectible), With<Sensor>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (collectible_entity, collectible_transform, collectible_type, collectible) in
        collectible_query.iter()
    {
        let distance = player_transform
            .translation
            .distance(collectible_transform.translation);
        if distance < 5.0 {
            // Collection radius
            info!("Collected a {:?}!", collectible_type);
            (collectible.on_collect)(&mut commands, collectible_entity);
            collectible_counter.collectibles_collected += 1;
            info!(
                "Total collectibles collected: {}",
                collectible_counter.collectibles_collected
            );
        }
    }
}

fn update_floating_items(time: Res<Time>, mut query: Query<(&FloatingItem, &mut Transform)>) {
    for (floating, mut transform) in query.iter_mut() {
        let time = time.elapsed_secs();
        let hover_offset = (time * floating.hover_speed).sin() * floating.hover_amplitude;
        transform.translation.y = floating.base_height + hover_offset;
    }
}

pub fn rotate_collectibles(
    mut collectible_query: Query<(&mut Transform, &CollectibleRotation)>,
    time: Res<Time>,
) {
    for (mut transform, rotation) in collectible_query.iter_mut() {
        if rotation.enabled {
            let rotation_amount = if rotation.clockwise {
                rotation.speed * time.delta_secs()
            } else {
                -rotation.speed * time.delta_secs()
            };
            transform.rotate_y(rotation_amount);
        }
    }
}
