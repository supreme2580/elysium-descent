use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

use crate::screens::Screen;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(Screen::Loading)
                .continue_to_state(Screen::MainMenu)
                .load_collection::<UiAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<FontAssets>()
                .load_collection::<ModelAssets>(),
        );
    }
}

// UI Assets
#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    // Backgrounds
    #[asset(path = "images/ui/background.png")]
    pub background: Handle<Image>,

    // Panels
    #[asset(path = "images/ui/panel_menu.png")]
    pub panel_menu: Handle<Image>,

    // Title
    #[asset(path = "images/ui/title.png")]
    pub title: Handle<Image>,

    // Components
    #[asset(path = "images/ui/components/button_symetric_sliced.png")]
    pub button_symmetric: Handle<Image>,

    #[asset(path = "images/ui/components/chevron_left.png")]
    pub chevron_left: Handle<Image>,

    #[asset(path = "images/ui/components/chevron_right.png")]
    pub chevron_right: Handle<Image>,

    #[asset(path = "images/collectibles/book.png")]
    pub book: Handle<Image>,

    #[asset(path = "images/collectibles/coin.png")]
    pub coin: Handle<Image>,

    #[asset(path = "avatars/player.png")]
    pub player_avatar: Handle<Image>,

    #[asset(path = "avatars/enemy.png")]
    pub enemy_avatar: Handle<Image>,
}

// Audio Assets
#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/main_menu.ogg")]
    pub main_menu_track: Handle<AudioSource>,

    #[asset(path = "audio/intro.ogg")]
    pub intro_track: Handle<AudioSource>,
}

// Font Assets
#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/GoudyTrajan-Regular.otf")]
    pub rajdhani_bold: Handle<Font>,

    #[asset(path = "fonts/GoudyTrajan-Regular.otf")]
    pub rajdhani_medium: Handle<Font>,

    #[asset(path = "fonts/rajdhani/Rajdhani-Bold.ttf")]
    pub rajdhani_extra_bold: Handle<Font>,
}

// Model Assets (for future use)
#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/book.glb#Scene0")]
    pub book: Handle<Scene>,

    #[asset(path = "models/coin.glb#Scene0")]
    pub coin: Handle<Scene>,

    #[asset(path = "models/player.glb")]
    pub player: Handle<Gltf>,

    #[asset(path = "models/environment.glb#Scene0")]
    pub environment: Handle<Scene>,

    #[asset(path = "models/dungeon.glb#Scene0")]
    pub dungeon: Handle<Scene>,

    #[asset(path = "models/enemy.glb#Scene0")]
    pub enemy: Handle<Scene>,
}

// Movie/Video Assets
// #[derive(AssetCollection, Resource)]
// pub struct MovieAssets {
//     // Note: Bevy doesn't have built-in video support
//     // You'll need a plugin for video playback
//     // For now, we'll just store the path
//     #[asset(path = "movies/intro.webp")]
//     pub intro: Handle<Image>, // WebP can be animated
// }
