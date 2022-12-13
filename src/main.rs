#[cfg(feature = "debug")]
use bevy::ecs::schedule::ReportExecutionOrderAmbiguities;
use bevy::{prelude::*, render::texture::ImagePlugin, window::WindowPlugin};
#[cfg(feature = "debug")]
use bevy_inspector_egui::WorldInspectorPlugin;
use magic_set::MagicSetPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "magic_set".to_string(),
                    width: 900.0,
                    height: 720.0,
                    ..default()
                },
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );
    #[cfg(feature = "debug")]
    app.add_plugin(WorldInspectorPlugin::new())
        .insert_resource(ReportExecutionOrderAmbiguities);

    app.add_plugin(MagicSetPlugin).run();
}
