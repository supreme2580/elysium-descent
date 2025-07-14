use crate::game::Player;
use bevy::{input::mouse::MouseWheel, prelude::*};

/// A third-person camera that follows the player
#[derive(Component)]
pub struct FlyCam {
    pub distance: f32,
    pub height_offset: f32,
    pub look_ahead: f32,
    pub smooth_speed: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            distance: 5.0,
            height_offset: 2.0,
            look_ahead: 2.0,
            smooth_speed: 5.0,
        }
    }
}

fn update_camera_position(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<FlyCam>)>,
    mut camera_query: Query<(&FlyCam, &mut Transform), Without<Player>>,
) {
    // Get player transform first
    let Ok(player_transform) = player_query.single() else {
        return;
    };

    // Then update camera positions
    for (camera, mut transform) in camera_query.iter_mut() {
        // Calculate desired camera position
        let player_pos = player_transform.translation;
        let player_forward = player_transform.forward();

        // Calculate camera position in front of player (opposite of before)
        let target_pos = player_pos + player_forward * camera.distance;
        let target_pos = Vec3::new(
            target_pos.x,
            player_pos.y + camera.height_offset,
            target_pos.z,
        );

        // Smoothly move camera to target position
        transform.translation = transform.translation.lerp(
            target_pos,
            (camera.smooth_speed * time.delta_secs()).min(1.0),
        );

        // Calculate look target (behind player now)
        let look_target = player_pos - player_forward * camera.look_ahead;

        // Make camera look at target
        transform.look_at(look_target, Vec3::Y);
    }
}

fn handle_camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut FlyCam>,
) {
    let delta: f32 = mouse_wheel_events.read().map(|e| e.y).sum();
    for mut camera in &mut query {
        // Adjust camera distance with zoom
        camera.distance -= delta * 0.5;
        camera.distance = camera.distance.clamp(12.0, 40.0); // Significantly increased zoom range
    }
}

/// Plugin for third-person camera behavior
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_camera_position, handle_camera_zoom));
    }
}
