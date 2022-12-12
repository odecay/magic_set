#[cfg(feature = "debug")]
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::{prelude::*, render::texture::{ImageSettings, ImagePlugin}, window::WindowPlugin};
#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;
use magic_set::MagicSetPlugin;

fn main() {
    let mut app = App::new();
    app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        WindowDescriptor {
            width: 900.0,
            heigh: 720.0,
            title: "magic_set".to_string(),
            ..default()
        }
    }, ImagePlugin::default_nearest()));
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ReportExecutionOrderAmbiguities);

    app.add_plugin(MagicSetPlugin).run();
}
