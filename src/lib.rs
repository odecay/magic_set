use bevy::prelude::*;
use bevy_ecs_tilemap::{prelude::*, TilePos};
use bevy_inspector_egui::{egui::Event, Inspectable, RegisterInspectable};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

mod helpers;

pub struct MagicSetPlugin;

impl Plugin for MagicSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .add_event::<MatchEvent>()
            .add_startup_system(startup)
            .add_system(spawn_cursor)
            .add_system(set_tiles)
            .add_system(update_tiles.after(set_tiles))
            .add_system(move_cursor)
            .add_system(draw_mark)
            .register_inspectable::<Color>()
            .register_inspectable::<Shape>()
            .add_system(set_mark)
            .add_system(check_match)
            .add_system(remove_tiles.after(check_match))
            .add_system(remove_mark.after(remove_tiles))
            .add_system(helpers::set_texture_filters_to_nearest);

        //.add_system(build_map);
    }
}

struct Center(Vec2);

struct MatchEvent(Entity);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("magic_chains.png");

    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);
    let layer_settings = LayerSettings::new(
        MapSize(10, 4),
        ChunkSize(2, 2),
        TileSize(32.0, 32.0),
        TextureSize(96.0, 96.0),
    );
    let center = layer_settings.get_pixel_center();
    let center_res = Center(center);

    let mut seed = thread_rng();

    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16);

    //layer_builder.set_all(TileBundle::default());

    //current random board generation/initialization, adds random color and shape components
    layer_builder.for_each_tiles_mut(|tile_entity, tile_data| {
        let random_color: Color = rand::random();
        let random_shape: Shape = rand::random();

        *tile_data = Some(TileBundle::new(
            Tile {
                texture_index: seed.gen_range(1..10),
                ..Default::default()
            },
            TilePos::default(),
        ));

        if tile_entity.is_none() {
            *tile_entity = Some(commands.spawn().id());
        }
        commands
            .entity(tile_entity.unwrap())
            .insert(random_color)
            .insert(random_shape);
    });

    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands.entity(layer_entity);

    map.add_layer(&mut commands, 0u16, layer_entity);

    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        // .insert(Transform::default())
        .insert(GlobalTransform::default());
    commands.insert_resource(layer_settings);
    commands.insert_resource(center_res);
}

//ok we need a better way than random samples to build the board tiles (maybe)
//waveform collapse?
//psudeorandom algorithm of some kind

fn set_tiles(mut query: Query<(&Color, &Shape, &mut Tile)>) {
    for (color, shape, mut tile) in query.iter_mut() {
        let c = match color {
            Color::Blue => 0,
            Color::Red => 1,
            Color::Yellow => 2,
        };
        let s = match shape {
            Shape::Diamond => 0,
            Shape::Circle => 3,
            Shape::Triangle => 6,
        };
        let index = c + s;
        tile.texture_index = index;
    }
}

fn update_tiles(
    // mut commands: Commands,
    mut map_query: MapQuery,
    mut query: Query<(&mut Color, &mut Shape, &TilePos)>,
) {
    for (mut color, mut shape, tile_pos) in query.iter_mut() {
        // *color = rand::random();
        // *shape = rand::random();
        map_query.notify_chunk_for_tile(*tile_pos, 0u16, 0u16)
    }
}

fn set_mark(
    mut query: Query<&TilePos, With<Cursor>>,
    mut map_query: MapQuery,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for position in query.iter_mut() {
            let tile_entity = map_query.get_tile_entity(*position, 0u16, 0u16).unwrap();
            commands.entity(tile_entity).insert(Mark);
        }
    }
}

fn draw_mark(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layer_settings: Res<LayerSettings>,
    center: Res<Center>,
    query: Query<(&Mark, &TilePos), Added<Mark>>,
) {
    //maybe have this draw as a child of the tile?? how do i remove the visual when i clean up mark
    let handle: Handle<Image> = asset_server.load("select.png");
    let tile_size = layer_settings.tile_size;
    for (_mark, tile_pos) in query.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                texture: handle.clone(),
                transform: Transform::from_xyz(
                    -center.0.x + (tile_size.0) * tile_pos.0 as f32 + (tile_size.0 / 2.0),
                    -center.0.y + (tile_size.1) * tile_pos.1 as f32 + (tile_size.1 / 2.0),
                    0.4,
                ),
                ..Default::default()
            })
            .insert(Selection);
        // commands.entity(entity).push_children(&[child]);
    }
}
fn remove_mark(
    mut commands: Commands,
    query: Query<Entity, With<Mark>>,
    sprite_query: Query<Entity, With<Selection>>,
) {
    if query.iter().count() == 3 {
        println!("3+ marks");
        for entity in query.iter() {
            commands.entity(entity).remove::<Mark>();
        }
    }

    //probably move this into its own system
    if sprite_query.iter().count() == 3 {
        for entity in sprite_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn check_match(
    query: Query<(&Color, &Shape), With<Mark>>,
    entities: Query<Entity, (With<Mark>, With<Color>, With<Shape>)>,
    mut match_event: EventWriter<MatchEvent>,
) {
    let (mut colors, mut shapes): (Vec<_>, Vec<_>) = query.iter().unzip();
    let mut color_match = false;
    if colors.len() == 3 {
        colors.sort_unstable();
        // println!("{:?}", colors);
        let mut unique_colors = colors.clone();
        unique_colors.dedup();
        println!("{:?}", unique_colors);
        if colors.len() == unique_colors.len() {
            color_match = true;
            // println!("match of different colors!")
        } else if unique_colors.len() == 1 {
            color_match = true;
            // println!("match of same color!")
        } else {
            // println!("no match")
        }
    }
    let mut shape_match = false;
    if shapes.len() == 3 {
        shapes.sort_unstable();
        // println!("{:?}", shapes);
        let mut unique_shapes = shapes.clone();
        unique_shapes.dedup();
        println!("{:?}", unique_shapes);
        if shapes.len() == unique_shapes.len() {
            shape_match = true;
            // println!("match of different shapes!")
        } else if unique_shapes.len() == 1 {
            shape_match = true;
            // println!("match of same shape!")
        } else {
            // println!("no match")
        }
    }
    if shape_match == true && color_match == true {
        println!("Match!!");
        for entity in entities.iter() {
            match_event.send(MatchEvent(entity))
        }
    }
}

fn remove_tiles(mut commands: Commands, mut match_event: EventReader<MatchEvent>) {
    for entity in match_event.iter() {
        commands.entity(entity.0).despawn_recursive();
    }
    //
}

fn spawn_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&Layer, Added<Layer>>,
) {
    let handle: Handle<Image> = asset_server.load("cursor.png");
    for layer in query.get_single() {
        let tile_pos = TilePos(0, 0);
        let center = layer.settings.get_pixel_center();
        let tile_size = layer.settings.tile_size;
        commands
            .spawn_bundle(SpriteBundle {
                texture: handle.clone(),
                transform: Transform::from_xyz(
                    -center.x + (tile_size.0 / 2.0),
                    -center.y + (tile_size.1 / 2.0),
                    1.0,
                ),
                ..Default::default()
            })
            .insert(Cursor)
            .insert(tile_pos);
    }
}

fn move_cursor(
    mut query: Query<(&mut TilePos, &mut Transform), With<Cursor>>,
    layer_settings: Res<LayerSettings>,
    keys: Res<Input<KeyCode>>,
) {
    let map_size: Vec2 = layer_settings.map_size.into();
    let chunk_size: Vec2 = layer_settings.chunk_size.into();
    let bounds = map_size * chunk_size;
    for (mut tile_pos, mut transform) in query.iter_mut() {
        if keys.just_pressed(KeyCode::W) {
            if tile_pos.1 < bounds.y as u32 - 1 {
                *tile_pos = TilePos(tile_pos.0, tile_pos.1 + 1);
                transform.translation.y += 32.0;
            }
        }
        if keys.just_pressed(KeyCode::S) {
            if tile_pos.1 > 0 {
                *tile_pos = TilePos(tile_pos.0, tile_pos.1 - 1);
                transform.translation.y -= 32.0;
            }
        }
        if keys.just_pressed(KeyCode::D) {
            if tile_pos.0 < bounds.x as u32 - 1 {
                *tile_pos = TilePos(tile_pos.0 + 1, tile_pos.1);
                transform.translation.x += 32.0;
            }
        }
        if keys.just_pressed(KeyCode::A) {
            if tile_pos.0 > 0 {
                *tile_pos = TilePos(tile_pos.0 - 1, tile_pos.1);
                transform.translation.x -= 32.0;
            }
        }
    }
}

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Mark;

#[derive(Component)]
struct Selection;

#[derive(Component)]
struct Card;

#[derive(Inspectable, Component, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
    Blue,
    Red,
    Yellow,
}

#[derive(Inspectable, Component, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Shape {
    Diamond,
    Circle,
    Triangle,
    //Star,
    //Cross,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..=2) {
            0 => Color::Blue,
            1 => Color::Red,
            _ => Color::Yellow,
        }
    }
}

impl Distribution<Shape> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Shape {
        match rng.gen_range(0..=2) {
            0 => Shape::Diamond,
            1 => Shape::Circle,
            _ => Shape::Triangle,
        }
    }
}
