use bevy::prelude::*;
use avian3d::prelude::*;
use crate::assets::ModelAssets;
use crate::ui::dialog::{spawn_dialog_with_proximity, DialogConfig, DialogPosition, DialogProximity};

// ===== COMPONENTS =====

#[derive(Component)]
pub struct Book;

// ===== SYSTEMS =====

pub fn spawn_book(
    mut commands: Commands,
    assets: Res<ModelAssets>,
) {
    commands.spawn((
        Name::new("Book"),
        SceneRoot(assets.book.clone()),
        Transform {
            translation: Vec3::new(90.0, 22.0, -54.0),
            scale: Vec3::splat(1.0),
            ..default()
        },
        Collider::cuboid(1.0, 1.0, 1.0), // Add collision box
        RigidBody::Static, // Make it static so it doesn't move
        Book,
    ));
}

pub fn spawn_book_modal(
    mut commands: Commands,
    font_assets: Res<crate::assets::FontAssets>,
    windows: Query<&Window>,
) {
    spawn_dialog_with_proximity(
        &mut commands,
        &font_assets,
        windows,
        DialogConfig {
            text: "Press E to Burn Items to Enter Realm".to_string(),
            position: DialogPosition::BottomCenter { bottom_margin: 4.0 },
            ..Default::default()
        },
        crate::screens::gameplay::PlayingScene,
        Some(DialogProximity {
            target_position: Vec3::new(90.0, 22.0, -54.0),
            proximity_threshold: 5.0,
        }),
    );
}

// ===== PLUGIN =====

pub struct BookInteractionPlugin;

impl Plugin for BookInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::screens::Screen::GamePlay), spawn_book)
            .add_systems(OnEnter(crate::screens::Screen::GamePlay), spawn_book_modal);
    }
} 