use bevy::prelude::*;
use bevy_inspector_egui::WorldInspectorPlugin;
//use bevy::window::WindowMode;
use magic_set::MagicSetPlugin;

fn main() {
    App::new()
        // Configure the game window
        .insert_resource(WindowDescriptor {
            width: 960.0,
            height: 720.0,
            title: "magic_set".to_string(),
            //mode: WindowMode::BorderlessFullscreen,
            ..Default::default()
        })
        // Standard Bevy functionality
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        // Add plugins here
        .add_plugin(MagicSetPlugin)
        .run();
}
