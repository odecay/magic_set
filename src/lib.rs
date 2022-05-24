use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::{
    map::{
        Tilemap2dGridSize, Tilemap2dSize, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapTexture,
    },
    tiles::{Tile2dStorage, TileBundle, TilePos2d, TileTexture, TileVisible},
    Tilemap2dPlugin, TilemapBundle,
};
#[cfg(feature = "debug")]
use bevy_inspector_egui::{egui::Event, Inspectable, RegisterInspectable};
use iyes_loopless::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

mod helpers;

pub struct MagicSetPlugin;

impl Plugin for MagicSetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Tilemap2dPlugin)
            .add_event::<RemoveEvent>()
            // .add_event::<MoveEvent>()
            .add_loopless_state(GameState::Base)
            .add_startup_system(startup)
            .add_system(set_tiles.run_in_state(GameState::Base))
            .add_system(spawn_cursor.run_in_state(GameState::Base))
            .add_system(move_cursor.run_in_state(GameState::Base))
            .add_system(draw_mark)
            // .add_system(gravity)
            .add_system(move_tiles)
            .add_system(set_mark.run_in_state(GameState::Base))
            .add_system(
                check_match
                    .run_if(select_condition)
                    .run_in_state(GameState::Base),
            )
            .add_system(
                remove_tiles.run_in_state(GameState::Match), // .after(check_match),
            )
            .add_system(
                remove_mark
                    .run_in_state(GameState::Match)
                    .after(remove_tiles),
            )
            // .add_system(remove_all.after(check_match))
            .add_system(helpers::set_texture_filters_to_nearest);

        #[cfg(feature = "debug")]
        {
            app.register_inspectable::<Color>()
                .register_inspectable::<Shape>();
        }
    }
}

struct RemoveEvent(Entity, TilePos2d);

// struct MoveEvent(Entity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    Base,
    Select,
    Match,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle: Handle<Image> = asset_server.load("magic_chains.png");
    let tilemap_size = Tilemap2dSize { x: 12, y: 8 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();

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
}

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

fn set_mark(
    mut query: Query<&TilePos2d, With<Cursor>>,
    tile_storage_query: Query<&Tile2dStorage>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for position in query.iter_mut() {
            let tile_storage = tile_storage_query.single();
            if let Some(tile_entity) = tile_storage.get(position) {
                // println!("{:?}", tile_entity);
                commands.entity(tile_entity).insert(Mark);
            }
        }
    }
}

fn draw_mark(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&TilePos2d, (With<Mark>, Added<Mark>)>,
    tile_size: Res<Tilemap2dTileSize>,
    tilemap_size: Res<Tilemap2dSize>,
) {
    let handle: Handle<Image> = asset_server.load("select.png");
    for tile_pos in query.iter() {
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
    commands.insert_resource(NextState(GameState::Base));
}

fn select_condition(query: Query<Entity, With<Mark>>) -> bool {
    if query.iter().count() >= 3 {
        true
    } else {
        false
    }
}

fn check_match(
    query: Query<(&Color, &Shape), With<Mark>>,
    entities: Query<(Entity, &TilePos2d), (With<Mark>, With<Color>, With<Shape>)>,
    mut match_event: EventWriter<RemoveEvent>,
    mut commands: Commands,
) {
    let (mut colors, mut shapes): (Vec<&Color>, Vec<&Shape>) = query.iter().unzip();
    let mut color_match = false;
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
    let mut shape_match = false;
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
    if shape_match == true && color_match == true {
        println!("Match!!");
        for (entity, pos) in entities.iter() {
            match_event.send(RemoveEvent(entity, *pos))
        }
    }
    commands.insert_resource(NextState(GameState::Match));
}

fn remove_tiles(
    mut commands: Commands,
    mut match_reader: EventReader<RemoveEvent>,
    mut storage: Query<&mut Tile2dStorage>,
) {
    let mut tile_storage = storage.single_mut();
    for evt in match_reader.iter() {
        commands.entity(evt.0).despawn_recursive();
        tile_storage.set(&evt.1, None)
    }
}

// fn random_remove(
//     mut commands: Commands,
//     query: Query<(Entity, &TilePos2d), (With<Color>, With<Shape>)>,
//     mut storage_query: Query<&mut Tile2dStorage>,
// ) {
//     let mut storage = storage_query.single_mut();
//     let mut random = thread_rng();
//     let position = TilePos2d {
//         x: random.gen_range(0..3),
//         y: random.gen_range(0..3),
//     };

//     if let Some(tile_entity) = storage.get(&position) {
//         commands.entity(tile_entity).despawn_recursive();
//         // Don't forget to remove tiles from the tile storage!
//         storage.set(&position, None);
//     }
// }

fn gravity(
    mut query: Query<(Entity, &mut TilePos2d), With<TileVisible>>,
    mut storage: Query<&mut Tile2dStorage>,
    size: Query<&Tilemap2dSize>,
) {
    let bound = size.single();
    let mut tile_storage = storage.single_mut();
    //hopefully this will work eventually dependant on ecs_tilemap working with "move of tiles"

    for x in 0..bound.x {
        for y in 0..bound.y {
            let tile_pos = TilePos2d { x: x, y: y };
            if let Some(below_pos) = tile_storage.get_pos_below(&tile_pos) {
                if let None = tile_storage.get(&below_pos) {
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        if let Ok(mut entity_tile_pos) =
                            query.get_component_mut::<TilePos2d>(tile_entity)
                        {
                            tile_storage.set(&below_pos, Some(tile_entity));
                            tile_storage.set(&tile_pos, None);
                            entity_tile_pos.y -= 1u32;
                        }
                    }
                }
            }
        }
    }
}

trait TileReturn {
    fn get_pos_below(&self, tile_pos: &TilePos2d) -> Option<TilePos2d>;
}

impl TileReturn for Tile2dStorage {
    fn get_pos_below(&self, tile_pos: &TilePos2d) -> Option<TilePos2d> {
        if tile_pos.y != 0 {
            Some(TilePos2d {
                x: tile_pos.x,
                y: tile_pos.y - 1,
            })
        } else {
            None
        }
    }
}

fn move_tiles(
    tile_query: Query<(&TilePos2d, &Color, &Shape)>,
    mut commands: Commands,
    mut storage: Query<&mut Tile2dStorage>,
    size: Query<&Tilemap2dSize>,
    tilemap_entity_query: Query<Entity, With<Tilemap2dSize>>,
) {
    let bound = size.single();
    let tilemap_entity = tilemap_entity_query.single();
    let mut tile_storage = storage.single_mut();
    //it works
    for x in 0..bound.x {
        for y in 0..bound.y {
            let tile_pos = TilePos2d { x: x, y: y };
            if let Some(below_pos) = tile_storage.get_pos_below(&tile_pos) {
                if let None = tile_storage.get(&below_pos) {
                    if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                        tile_storage.set(&tile_pos, None);
                        if let Ok((entity_tile_pos, color, shape)) = tile_query.get(tile_entity) {
                            //copy over components, spawn a new tile
                            let moved_tile = commands
                                .spawn()
                                .insert_bundle(TileBundle {
                                    position: below_pos,
                                    tilemap_id: TilemapId(tilemap_entity),
                                    ..default()
                                })
                                .insert(*color)
                                .insert(*shape)
                                .id();
                            tile_storage.set(&below_pos, Some(moved_tile));
                            commands.entity(tile_entity).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}

fn spawn_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<Entity, (With<Tile2dStorage>, Added<Tile2dStorage>)>,
    tile_size: Res<Tilemap2dTileSize>,
    tilemap_size: Res<Tilemap2dSize>,
) {
    if let Ok(_) = query.get_single() {
        let handle: Handle<Image> = asset_server.load("cursor.png");
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
                ..default()
            })
            .insert(Cursor)
            .insert(tile_pos);
    }
}

fn move_cursor(
    mut query: Query<(&mut TilePos2d, &mut Transform), With<Cursor>>,
    bound_query: Query<&Tilemap2dSize>,
    size_query: Query<&Tilemap2dTileSize>,
    tile_storage_query: Query<&Tile2dStorage>,
    keys: Res<Input<KeyCode>>,
) {
    let bounds = bound_query.single();
    let tile_size = size_query.single();
    for (mut tile_pos, mut transform) in query.iter_mut() {
        if keys.just_pressed(KeyCode::W) {
            if tile_pos.y < bounds.y as u32 - 1 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x,
                    y: tile_pos.y + 1,
                };
                transform.translation.y += tile_size.y;
            }
        }
        if keys.just_pressed(KeyCode::S) {
            if tile_pos.y > 0 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x,
                    y: tile_pos.y - 1,
                };
                transform.translation.y -= tile_size.y;
            }
        }
        if keys.just_pressed(KeyCode::D) {
            if tile_pos.x < bounds.x as u32 - 1 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x + 1,
                    y: tile_pos.y,
                };
                transform.translation.x += tile_size.x;
            }
        }
        if keys.just_pressed(KeyCode::A) {
            if tile_pos.x > 0 {
                *tile_pos = TilePos2d {
                    x: tile_pos.x - 1,
                    y: tile_pos.y,
                };
                transform.translation.x -= tile_size.x;
            }
        }
        //debug for tile_storage
        if keys.just_pressed(KeyCode::P) {
            println!("tile_storage {:?}", tile_storage_query.single());
        }
    }
}

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Mark;

#[derive(Component)]
struct Selection;

#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Color {
    Blue,
    Red,
    Yellow,
}
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::Inspectable))]
#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
