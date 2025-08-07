// Sound Effects
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::assets::AudioAssets;
use crate::resources::audio::{SfxChannel, AudioSettings};
use crate::systems::character_controller::{AnimationState, CharacterController};
use avian3d::prelude::LinearVelocity;


#[derive(Event)]
pub struct PlaySfxEvent {
    pub sfx_type: SfxType,
}

#[derive(Event)]
pub struct StopMovementAudioEvent;

#[derive(Resource, Default)]
pub struct MovementAudioState {
    pub is_moving: bool,
    pub is_running: bool,
    pub current_sound: Option<SfxType>,
    pub current_audio_handle: Option<Handle<AudioInstance>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SfxType {
    CoinCollect,
    Walking,
    Running,
}

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementAudioState>()
            .add_event::<PlaySfxEvent>()
            .add_event::<StopMovementAudioEvent>()
            .add_systems(Update, play_sfx_events)
            .add_systems(Update, stop_movement_audio)
            .add_systems(Update, handle_movement_sfx.run_if(in_state(crate::screens::Screen::GamePlay)))
            .add_systems(Update, handle_movement_sfx.run_if(in_state(crate::screens::Screen::FightScene)));
    }
}

fn play_sfx_events(
    mut sfx_events: EventReader<PlaySfxEvent>,
    audio_assets: Option<Res<AudioAssets>>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
    audio_settings: Res<AudioSettings>,
) {
    let Some(assets) = audio_assets else {
        return;
    };

    for event in sfx_events.read() {
        if audio_settings.muted {
            continue;
        }

        match event.sfx_type {
            SfxType::CoinCollect => {
                sfx_channel.play(assets.coin_sound.clone());
            }
            SfxType::Walking => {
                sfx_channel.play(assets.walking_sound.clone()).looped();
            }
            SfxType::Running => {
                sfx_channel.play(assets.running_sound.clone()).looped();
            }
        }
    }
}

fn handle_movement_sfx(
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut stop_events: EventWriter<StopMovementAudioEvent>,
    mut movement_state: ResMut<MovementAudioState>,
    character_query: Query<(&LinearVelocity, &AnimationState), With<CharacterController>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check if any movement keys are pressed
    let is_movement_pressed = keyboard.any_pressed([
        KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD,
        KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight,
    ]);

    // Check if character is moving (based on velocity)
    let is_moving = character_query.iter().any(|(velocity, _)| {
        let horizontal_velocity = Vec2::new(velocity.x, velocity.z);
        horizontal_velocity.length() > 0.05
    });

    // Check if character is running
    let is_running = character_query.iter().any(|(_, animation_state)| {
        animation_state.forward_hold_time >= 3.0
    });



    // Handle state changes - prioritize input over velocity for immediate response
    let _should_play_sound = if is_movement_pressed && is_moving {
        let sound_to_play = if is_running { SfxType::Running } else { SfxType::Walking };
        
        // Check if we need to change the sound
        let should_change = match movement_state.current_sound {
            Some(current) => current != sound_to_play,
            None => true, // No current sound, need to play one
        };

        if should_change {
            // Stop any current sound by sending a "stop" event (we'll handle this in the audio system)
            if movement_state.current_sound.is_some() {
                // For now, we'll just change the sound immediately
                movement_state.current_sound = Some(sound_to_play);
                sfx_events.write(PlaySfxEvent { sfx_type: sound_to_play });
            } else {
                // First time playing a sound
                movement_state.current_sound = Some(sound_to_play);
                sfx_events.write(PlaySfxEvent { sfx_type: sound_to_play });
            }
        }
        
        true
    } else {
        // Not moving or no movement keys pressed, stop current sound immediately
        if movement_state.current_sound.is_some() {
            movement_state.current_sound = None;
            stop_events.write(StopMovementAudioEvent);
        }
        false
    };

    // Update state
    movement_state.is_moving = is_moving && is_movement_pressed;
    movement_state.is_running = is_running && is_movement_pressed;
}

fn stop_movement_audio(
    mut stop_events: EventReader<StopMovementAudioEvent>,
    sfx_channel: Res<AudioChannel<SfxChannel>>,
) {
    for _event in stop_events.read() {
        // Stop all audio on the SFX channel (this will stop movement sounds)
        sfx_channel.stop();
    }
}
