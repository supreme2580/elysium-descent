use crate::assets::FontAssets;
use crate::ui::widgets::label_widget;
use bevy::prelude::*;

// ===== DIALOG COMPONENTS =====

#[derive(Component)]
pub struct Dialog;

#[derive(Component)]
pub struct DialogProximity {
    pub target_position: Vec3,
    pub proximity_threshold: f32,
}

#[derive(Resource, Clone)]
pub struct DialogConfig {
    pub text: String,
    pub width: f32,  // Percentage of screen width
    #[allow(dead_code)]
    pub height: f32, // Percentage of screen height
    pub position: DialogPosition,
    pub background_color: Color,
    pub border_color: Color,
    pub border_width: f32,
}

#[derive(Clone, Copy)]
pub enum DialogPosition {
    BottomCenter { bottom_margin: f32 },
}

impl Default for DialogConfig {
    fn default() -> Self {
        Self {
            text: "Press E to enter".to_string(),
            width: 26.67, // Reduced from 40.0 by 1.5x (40.0 / 1.5)
            height: 5.33, // Reduced from 8.0 by 1.5x (8.0 / 1.5)
            position: DialogPosition::BottomCenter { bottom_margin: 4.0 },
            background_color: Color::srgba(0.1, 0.1, 0.2, 0.9), // More opaque background for better text visibility
            border_color: Color::srgba(0.2, 0.2, 0.3, 0.8),
            border_width: 2.0,
        }
    }
}

// ===== DIALOG SYSTEMS =====

pub fn spawn_dialog(
    commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    windows: Query<&Window>,
    config: DialogConfig,
    scene_marker: impl Component + Clone,
) {
    spawn_dialog_with_proximity(commands, font_assets, windows, config, scene_marker, None);
}

pub fn spawn_dialog_with_proximity(
    commands: &mut Commands,
    font_assets: &Res<FontAssets>,
    windows: Query<&Window>,
    config: DialogConfig,
    scene_marker: impl Component + Clone,
    proximity: Option<DialogProximity>,
) {
    let window = windows.single().expect("No primary window");
    let window_height = window.height();
    let _window_width = window.width();

    let (_, bottom) = match config.position {
        DialogPosition::BottomCenter { bottom_margin } => {
            (50.0 - config.width / 2.0, bottom_margin)
        }
    };

    // Calculate dynamic font size based on screen size for better readability
    // Use screen height as the base for font size calculation
    let base_font_size = window_height * 0.03; // 3% of screen height
    let responsive_font_size = base_font_size.max(20.0).min(60.0); // Min 20px, max 60px
    
    // Calculate responsive font size based on screen size
    
    // Use exact same width as inventory (833px)
    let dialog_width_px = 833.0;
    
    // Calculate dynamic dialog size based on text content and font size
    // Estimate text width based on character count and font size
    let text_length = config.text.len() as f32;
    let estimated_text_width = text_length * responsive_font_size * 0.6; // Approximate character width
    let estimated_text_height = responsive_font_size * 1.5; // Approximate line height
    
    // Calculate minimum dialog size with padding
    let padding = responsive_font_size * 0.5; // Padding proportional to font size
    let _min_dialog_width = estimated_text_width + (padding * 2.0);
    let min_dialog_height = estimated_text_height + (padding * 2.0);
    
    // Use inventory width (833px), but ensure minimum height for text
    let dynamic_width_px = dialog_width_px;
    let dynamic_height_px = min_dialog_height.max(100.0); // Minimum 100px height
    
    // Use exact same centering method as inventory
    // Inventory uses: left: Val::Percent(50.0), margin: UiRect::left(Val::Px(-416.5))
    // where -416.5 is half of 833px width
    
    // Calculate dynamic dialog dimensions

    let mut entity_commands = commands.spawn((
        Node {
            width: Val::Px(dynamic_width_px),
            height: Val::Px(dynamic_height_px),
            position_type: PositionType::Absolute,
            bottom: Val::Percent(bottom),
            left: Val::Percent(50.0), // Same as inventory
            margin: UiRect::left(Val::Px(-416.5)), // Same as inventory (-833/2)
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(config.border_width)),
            padding: UiRect::all(Val::Px(padding as f32)), // Add padding around text
            ..default()
        },
        BackgroundColor(config.background_color),
        BorderColor(config.border_color),
        Dialog,
        scene_marker.clone(),
        Name::new(format!("Dialog: {}", config.text)),
        Visibility::Hidden, // Start hidden
    ));

    // Add proximity component if provided
    if let Some(proximity) = proximity {
        entity_commands.insert(proximity);
    }

    entity_commands.with_children(|parent| {
        parent.spawn(label_widget(
            responsive_font_size,
            font_assets.rajdhani_bold.clone(),
            config.text.clone(),
        ));
    });
}

pub fn animate_dialog(time: Res<Time>, mut query: Query<&mut BackgroundColor, With<Dialog>>) {
    let t = (time.elapsed_secs().sin() * 0.5 + 0.5) * 0.5 + 0.5;
    for mut bg in &mut query {
        let base_alpha = 0.4;
        let pulse_alpha = 0.3;
        let new_alpha = base_alpha + pulse_alpha * t;

        // Create a new color with the same RGB values but animated alpha
        let new_color = Color::srgba(0.1, 0.1, 0.2, new_alpha);
        *bg = BackgroundColor(new_color);
    }
}

pub fn check_dialog_proximity(
    player_query: Query<
        &Transform,
        With<crate::systems::character_controller::CharacterController>,
    >,
    mut dialog_query: Query<(&mut Visibility, Option<&DialogProximity>), With<Dialog>>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    for (mut visibility, proximity) in dialog_query.iter_mut() {
        if let Some(proximity) = proximity {
            // Check distance to target position
            let distance = player_transform.translation.distance(proximity.target_position);
            if distance <= proximity.proximity_threshold {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            // For dialogs without proximity, keep them hidden
            *visibility = Visibility::Hidden;
        }
    }
}

// ===== CONVENIENCE FUNCTIONS =====

// ===== DIALOG PLUGIN =====

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_dialog, check_dialog_proximity));
    }
}
