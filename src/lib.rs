use bevy::prelude::*;
use bevy::sprite::Anchor;
// use bevy_ecs_tilemap::{prelude::*, TilePos};
use bevy_ecs_tilemap::{
    map::{
        Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapTexture,
    },
    tiles::{Tile2dStorage, TileBundle, TilePos2d, TileTexture, TileVisible},
    Tilemap2dPlugin, TilemapBundle,
};
use bevy_inspector_egui::{egui::Event, Inspectable, RegisterInspectable};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

mod helpers;

pub struct MagicSetPlugin;

impl Plugin for MagicSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Tilemap2dPlugin)
            .add_event::<MatchedEvent>()
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

// struct Center(Vec2);

struct MatchedEvent(Entity);

// fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("magic_chains.png");
    let tilemap_size = Tilemap2dSize { x: 20, y: 8 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

    // let map_entity = commands.spawn().id();
    // let mut map = Map::new(0u16, map_entity);
    // let layer_settings = LayerSettings::new(
    //     MapSize(10, 4),
    //     ChunkSize(2, 2),
    //     TileSize(32.0, 32.0),
    //     TextureSize(96.0, 96.0),
    // );
    // let center = layer_settings.get_pixel_center();

    //do i need to figure a diff way to get center??
    //maybe figure out how to properly use child transform for placing cursor and mark??
    // let center = bevy_ecs_tilemap::helpers::get_centered_transform_2d(size, tile_size, z_index)
    // let center_res = Center(center);

    for x in 0..tilemap_size.x {
        for y in 0..tilemap_size.y {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .insert(rand::random::<Shape>())
                .insert(rand::random::<Color>())
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }

    let tile_size = Tilemap2dTileSize { x: 32.0, y: 32.0 };
    commands.insert_resource(tile_size);
    commands.insert_resource(tilemap_size);

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 32.0, y: 32.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 96.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                &tilemap_size,
                &tile_size,
                0.0,
            ),
            ..default()
        });

    // let (mut layer_builder, layer_entity) =
    //     LayerBuilder::<TileBundle>::new(&mut commands, layer_settings, 0u16, 0u16);

    //layer_builder.set_all(TileBundle::default());

    //current random board generation/initialization, adds random color and shape components
    // layer_builder.for_each_tiles_mut(|tile_entity, tile_data| {
    //     let random_color: Color = rand::random();
    //     let random_shape: Shape = rand::random();

    //     *tile_data = Some(TileBundle::new(
    //         Tile {
    //             texture_index: seed.gen_range(1..10),
    //             ..Default::default()
    //         },
    //         TilePos::default(),
    //     ));

    //     if tile_entity.is_none() {
    //         *tile_entity = Some(commands.spawn().id());
    //     }
    //     commands
    //         .entity(tile_entity.unwrap())
    //         .insert(random_color)
    //         .insert(random_shape);
    // });

    // map_query.build_layer(&mut commands, layer_builder, texture_handle);

    // commands.entity(layer_entity);

    // map.add_layer(&mut commands, 0u16, layer_entity);

    // commands
    //     .entity(map_entity)
    //     .insert(map)
    //     .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
    //     // .insert(Transform::default())
    //     .insert(GlobalTransform::default());
    // commands.insert_resource(layer_settings);
    // commands.insert_resource(center_res);
}

//ok we need a better way than random samples to build the board tiles (maybe)
//waveform collapse?
//psudeorandom algorithm of some kind

fn set_tiles(mut query: Query<(&Color, &Shape, &mut TileTexture)>) {
    for (color, shape, mut tile_texture) in query.iter_mut() {
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
        tile_texture.0 = index;
    }
}

fn update_tiles(
    // mut commands: Commands,
    // mut map_query: MapQuery,
    mut query: Query<(&mut Color, &mut Shape, &TilePos2d)>,
) {
    for (mut color, mut shape, tile_pos) in query.iter_mut() {
        // *color = rand::random();
        // *shape = rand::random();
        // map_query.notify_chunk_for_tile(*tile_pos, 0u16, 0u16)
    }
}

fn set_mark(
    mut query: Query<&TilePos2d, With<Cursor>>,
    mut tile_storage_query: Query<&Tile2dStorage>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for position in query.iter_mut() {
            let tile_storage = tile_storage_query.single_mut();
            if let Some(tile_entity) = tile_storage.get(position) {
                println!("{:?}", tile_entity);
                commands.entity(tile_entity).insert(Mark);
            }
        }
    }
}

fn draw_mark(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(&Mark, &TilePos2d), Added<Mark>>,
    tile_size: Res<Tilemap2dTileSize>,
    tilemap_size: Res<Tilemap2dSize>,
) {
    let handle: Handle<Image> = asset_server.load("select.png");
    for (_mark, tile_pos) in query.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                texture: handle.clone(),
                transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                    &tilemap_size,
                    &tile_size,
                    1.0,
                )
                .mul_transform(Transform::from_xyz(
                    tile_pos.x as f32 * tile_size.x,
                    tile_pos.y as f32 * tile_size.y,
                    1.0,
                )),
                ..Default::default()
            })
            .insert(Selection);
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
    mut match_event: EventWriter<MatchedEvent>,
) {
    let (mut colors, mut shapes): (Vec<_>, Vec<_>) = query.iter().unzip();
    let mut color_match = false;
    if colors.len() == 3 {
        colors.sort_unstable();
        // println!("{:?}", colors);
        let mut unique_colors = colors.clone();
        unique_colors.dedup();
        // println!("{:?}", unique_colors);
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
        // println!("{:?}", unique_shapes);
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
            match_event.send(MatchedEvent(entity))
        }
    }
}

fn remove_tiles(
    mut commands: Commands,
    mut match_event: EventReader<MatchedEvent>,
    // query: Query<(&TilePos, &Color, &Shape)>, // query: Query<&Children, >
) {
    for entity in match_event.iter() {
        commands
            .entity(entity.0)
            .remove::<Shape>()
            .remove::<Color>()
            .insert(TileVisible(false));
        // .remove::<Tile>();
        //not sure what to replace Tile type with here??
        // .remove::<TilemapId>();
        // commands.entity(entity.0).despawn_recursive();
    }
    //
}

fn spawn_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // query: Query<(&Tilemap2dSize, &Tilemap2dTileSize), Added<Tile2dStorage>>,
    query: Query<Entity, (With<Tile2dStorage>, Added<Tile2dStorage>)>,
    tile_size: Res<Tilemap2dTileSize>,
    tilemap_size: Res<Tilemap2dSize>,
) {
    if let Ok(_) = query.get_single() {
        let handle: Handle<Image> = asset_server.load("cursor.png");
        // let (tilemap_size, tile_size) = query.single();
        let tile_pos = TilePos2d { x: 0, y: 0 };

        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::BottomLeft,
                    ..default()
                },
                texture: handle.clone(),
                transform: bevy_ecs_tilemap::helpers::get_centered_transform_2d(
                    &tilemap_size,
                    &tile_size,
                    1.0,
                ),

                //might have to come back and add in the half tile offset
                // transform: Transform::from_xyz(
                //     -center.x + (tile_size.0 / 2.0),
                //     -center.y + (tile_size.1 / 2.0),
                //     1.0,
                // ),
                ..default()
            })
            .insert(Cursor)
            .insert(tile_pos);
    }
}

fn move_cursor(
    mut query: Query<(&mut TilePos2d, &mut Transform), With<Cursor>>,
    // layer_settings: Res<LayerSettings>,
    settings_query: Query<&Tilemap2dSize>,
    keys: Res<Input<KeyCode>>,
) {
    // let map_size: Vec2 = layer_settings.map_size.into();
    // let chunk_size: Vec2 = layer_settings.chunk_size.into();
    // let bounds = map_size * chunk_size;
    let bounds = settings_query.single();
    for (mut tile_pos, mut transform) in query.iter_mut() {
        if keys.just_pressed(KeyCode::W) {
            if tile_pos.y < bounds.y as u32 - 1 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x,
                    y: tile_pos.y + 1,
                };
                transform.translation.y += 32.0;
            }
        }
        if keys.just_pressed(KeyCode::S) {
            if tile_pos.y > 0 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x,
                    y: tile_pos.y - 1,
                };
                transform.translation.y -= 32.0;
            }
        }
        if keys.just_pressed(KeyCode::D) {
            if tile_pos.x < bounds.x as u32 - 1 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x + 1,
                    y: tile_pos.y,
                };
                transform.translation.x += 32.0;
            }
        }
        if keys.just_pressed(KeyCode::A) {
            if tile_pos.x > 0 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x - 1,
                    y: tile_pos.y,
                };
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
