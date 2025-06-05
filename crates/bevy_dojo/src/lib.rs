use bevy::prelude::*;

pub struct BevyDojoPlugin;

impl Plugin for BevyDojoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MinimalPlugins);
    }
}
