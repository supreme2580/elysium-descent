use bevy::prelude::*;
use crate::game::resources::{GameTimer, PlayerProgression};

pub fn update_game_timer(
    mut timer: ResMut<GameTimer>,
    time: Res<Time>,
) {
    if timer.is_running && timer.time_left_seconds > 0.0 {
        timer.time_left_seconds -= time.delta_secs();
        
        // Clamp to 0 to prevent negative time
        if timer.time_left_seconds < 0.0 {
            timer.time_left_seconds = 0.0;
            timer.is_running = false;
            // TODO: Handle game over/time up event
        }
    }
}

pub fn update_hud_display(
    timer: Res<GameTimer>,
    progression: Res<PlayerProgression>,
    mut query: Query<&mut Text, With<TimerText>>,
    mut level_query: Query<&mut Text, (With<LevelText>, Without<TimerText>)>,
    mut xp_query: Query<&mut Text, (With<XpText>, Without<TimerText>, Without<LevelText>)>,
) {
    // Update timer text
    for mut text in query.iter_mut() {
        text.0 = timer.get_time_remaining_formatted();
    }
    
    // Update level badge
    for mut text in level_query.iter_mut() {
        text.0 = progression.current_level.to_string();
    }
    
    // Update XP text
    for mut text in xp_query.iter_mut() {
        let (current, max) = progression.get_xp_for_display();
        text.0 = format!("{}/{}", current, max);
    }
}

pub fn update_progress_bars(
    timer: Res<GameTimer>,
    progression: Res<PlayerProgression>,
    mut timer_bar_query: Query<&mut Node, (With<TimerBar>, Without<XpBar>)>,
    mut xp_bar_query: Query<&mut Node, (With<XpBar>, Without<TimerBar>)>,
) {
    // Update timer progress bar width
    let time_percent = timer.get_time_percentage();
    for mut node in timer_bar_query.iter_mut() {
        node.width = Val::Px(417.0 * time_percent);
    }
    
    // Update XP progress bar width
    let (current, max) = progression.get_xp_for_display();
    let xp_percent = if max > 0 { current as f32 / max as f32 } else { 0.0 };
    for mut node in xp_bar_query.iter_mut() {
        node.width = Val::Px(417.0 * xp_percent);
    }
}

// Component markers for different text elements
#[derive(Component)]
pub struct TimerText;

#[derive(Component)]
pub struct LevelText;

#[derive(Component)]
pub struct XpText;

// Component markers for progress bars
#[derive(Component)]
pub struct TimerBar;

#[derive(Component)]
pub struct XpBar;
