#[cfg(feature = "debug")]
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::prelude::*;
#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;
use magic_set::MagicSetPlugin;

fn main() {
    let mut app = App::new();
    // Configure the game window
    app.insert_resource(WindowDescriptor {
        // width: 960.0,
        width: 900.0,
        height: 720.0,
        title: "magic_set".to_string(),
        ..default()
    })
    .add_plugins(DefaultPlugins);
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ReportExecutionOrderAmbiguities);

    app.add_plugin(MagicSetPlugin).run();
}
