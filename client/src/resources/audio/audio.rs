use crate::screens::Screen;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

use crate::assets::AudioAssets;

#[derive(Resource, Default)]
pub struct AudioResources {
    pub current_music: Option<Handle<AudioInstance>>,
    pub main_menu_track: Option<Handle<AudioSource>>,
    pub intro_track: Option<Handle<AudioSource>>,
}

#[derive(Resource)]
pub struct MusicChannel;

#[derive(Resource)]
pub struct SfxChannel;

#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f64,
    pub music_volume: f64,
    pub sfx_volume: f64,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 1.0,
            sfx_volume: 0.7,
            muted: false,
        }
    }
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioResources>()
            .init_resource::<AudioSettings>()
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SfxChannel>()
            .add_systems(OnEnter(Screen::MainMenu), setup_audio)
            .add_systems(
                Update,
                (
                    apply_audio_settings,
                    handle_screen_transitions,
                    instance_control,
                ),
            );
    }
}

fn setup_audio(audio_assets: Res<AudioAssets>, mut audio_resources: ResMut<AudioResources>) {
    audio_resources.main_menu_track = Some(audio_assets.main_menu_track.clone());
    audio_resources.intro_track = Some(audio_assets.intro_track.clone());
}

fn apply_audio_settings(
    settings: Res<AudioSettings>,
    music: Res<AudioChannel<MusicChannel>>,
    sfx: Res<AudioChannel<SfxChannel>>,
) {
    if settings.muted {
        music.set_volume(0.0);
        sfx.set_volume(0.0);
        return;
    }
    music.set_volume(settings.master_volume * settings.music_volume);
    sfx.set_volume(settings.master_volume * settings.sfx_volume);
}

fn handle_screen_transitions(
    mut audio_resources: ResMut<AudioResources>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    current_state: Res<State<Screen>>,
    mut prev_state: Local<Screen>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    if *prev_state == *current_state.get() {
        return;
    }

    *prev_state = current_state.get().clone();

    // Stop current music
    if let Some(handle) = audio_resources.current_music.take() {
        if let Some(instance) = audio_instances.get_mut(&handle) {
            instance.stop(AudioTween::new(
                Duration::from_secs(1),
                AudioEasing::OutPowf(2.0),
            ));
        }
    }

    // Play new track if needed
    if matches!(
        current_state.get(),
        Screen::MainMenu | Screen::NewGame | Screen::Settings
    ) {
        if let Some(track) = &audio_resources.main_menu_track {
            let handle = music_channel
                .play(track.clone())
                .looped()
                .fade_in(AudioTween::new(
                    Duration::from_secs(2),
                    AudioEasing::OutPowf(2.0),
                ))
                .handle();

            audio_resources.current_music = Some(handle);
        }
    }
}

fn instance_control(input: Res<ButtonInput<KeyCode>>, mut settings: ResMut<AudioSettings>) {
    if input.just_pressed(KeyCode::KeyM) {
        settings.muted = !settings.muted;
        info!("Mute toggled: {}", settings.muted);
    }
}
