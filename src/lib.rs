use bevy::prelude::*;
use bevy_ecs_tilemap::{prelude::*, TilePos};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

// pub mod tiles?
mod helpers;

pub struct MagicSetPlugin;

impl Plugin for MagicSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(TilemapPlugin)
            .add_startup_system(startup)
            .add_system(spawn_cursor)
            .add_system(set_tiles)
            .add_system(update_tiles.after(set_tiles))
            .add_system(move_cursor)
            .add_system(draw_mark)
            .register_inspectable::<Color>()
            .register_inspectable::<Shape>()
            .add_system(set_mark)
            .add_system(helpers::set_texture_filters_to_nearest);

        //.add_system(build_map);
    }
}

struct Center(Vec2);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("magic_chains.png");

    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);
    let layer_settings = LayerSettings::new(
        MapSize(4, 3),
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

        //figure out how to randomize selecting an enum variant
        if tile_entity.is_none() {
            *tile_entity = Some(commands.spawn().id());
        }
        commands
            .entity(tile_entity.unwrap())
            .insert(random_color)
            .insert(random_shape);
    });

    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    // commands.entity(layer_entity).insert(LastUpdate::default());
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
//rename to build_board or generate board?
//how to add custom components for each tile?
//ok we need a better way than random samples to build the board tiles (maybe)

// fn build_map(mut map_query: MapQuery, mut commands: Commands) {
//     let mut seed = thread_rng();

//     //might need diff range for loop
//     for _ in 0..2 {
//         let position = TilePos(seed.gen_range(0..16), seed.gen_range(0..12));

//         let _ = map_query.set_tile(
//             &mut commands,
//             position,
//             Tile {
//                 texture_index: seed.gen_range(0..10),
//                 ..Default::default()
//             },
//             0u16,
//             0u16,
//         );
//         map_query.notify_chunk_for_tile(position, 0u16, 0u16);
//     }
// }

fn set_tiles(mut query: Query<(&Color, &Shape, &mut Tile, &TilePos)>) {
    for (color, shape, mut tile, tile_pos) in query.iter_mut() {
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
        // println!(
        //     "texture_index {:?}, tilepos {:?}",
        //     tile.texture_index, tile_pos
        // );
    }
}

fn update_tiles(
    mut commands: Commands,
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
) {
    for position in query.iter_mut() {
        let tile_entity = map_query.get_tile_entity(*position, 0u16, 0u16).unwrap();
        commands.entity(tile_entity).insert(Mark);
    }
}

fn draw_mark(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    layer_settings: Res<LayerSettings>,
    center: Res<Center>,
    query: Query<(&Mark, &TilePos), Added<Mark>>,
) {
    let handle: Handle<Image> = asset_server.load("select.png");
    let tile_size = layer_settings.tile_size;
    for (_mark, tile_pos) in query.iter() {
        commands.spawn_bundle(SpriteBundle {
            texture: handle.clone(),
            transform: Transform::from_xyz(
                -center.0.x + (tile_size.0) * tile_pos.0 as f32 + (tile_size.0 / 2.0),
                -center.0.y + (tile_size.1) * tile_pos.1 as f32 + (tile_size.1 / 2.0),
                0.4,
            ),
            ..Default::default()
        });
    }
}

fn spawn_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&Layer, Added<Layer>>,
) {
    let handle: Handle<Image> = asset_server.load("cursor.png");
    for layer in query.get_single() {
        println!("found layersettings, spawning cursor");
        let tile_pos = TilePos(0, 0);
        let center = layer.settings.get_pixel_center();
        let tile_size = layer.settings.tile_size;
        // let center = Vec2::new(x = 128.0, y = 128.0);
        // let center = (128.0, 128.0);
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
    // let tile_size = &layer.settings.tile_size;

    // let tile_size = TileSize(32.0, 32.0);
    // let tile_pos = TilePos(0, 0);
    // x_offset = ((map_size.0 * chunk_size.0) as f32 * tile_size.0) / 2.0
    // y_offset = ((map_size.1 * chunk_size.1) as f32 * tile_size.1) / 2.0
    // ((self.map_size.0 * self.chunk_size.0) as f32 * self.tile_size.0) / 2.0,
    // ((self.map_size.1 * self.chunk_size.1) as f32 * self.tile_size.1) / 2.0,
    // let x = tile_size.0 * tile_pos.0 as f32 + tile_size.0 / 2.0;
    // let y = tile_size.1 * tile_pos.1 as f32 + tile_size.1 / 2.0;

    // let translation = Vec3::new(x, y, 1.0);
    // commands
    //     .spawn_bundle(SpriteBundle {
    //         texture: handle,
    //         transform: Transform::from_xyz(-center.x, -center.y, 1.0),
    //         ..Default::default()
    //     })
    //     .insert(Cursor)
    //     .insert(tile_pos);
    // let cursor = commands.spawn().insert(Cursor).insert(tile_pos);
}

fn move_cursor(
    mut query: Query<(&mut TilePos, &mut Transform), With<Cursor>>,
    keys: Res<Input<KeyCode>>,
) {
    // the oob checks should be changed to check the LayerSettings
    for (mut tile_pos, mut transform) in query.iter_mut() {
        if keys.just_pressed(KeyCode::W) {
            if tile_pos.1 < 5 {
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
            if tile_pos.0 < 7 {
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
//maybe turn this into a resource? which would just contain a tile position

#[derive(Component)]
struct Mark;

#[derive(Component)]
struct Card;

#[derive(Component)]
struct Grid;

#[derive(Inspectable, Component)]
// #[derive(Component)]
enum Color {
    Blue,
    Red,
    Yellow,
}

#[derive(Inspectable, Component)]
// #[derive(Component)]
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

// trait SampleableEnum {
//     fn enum_sample(R: Range(0..=2)) -> Self;
// }

// impl SampleableEnum for Color {
//     fn enum_sample(&self) -> Color {
//         match rng.gen_range(0..=2) {
//             0 => Color::Red,
//             1 => Color::Yellow,
//             _ => Color::Blue,

//         }
// }
//     }

// impl Distribution<T: SampleableEnum> for Standard {
//     fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
//         self.enum_sample()
//     }
// }
