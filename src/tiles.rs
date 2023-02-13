use bevy::prelude::*;
use rand::{distributions::Standard, prelude::Distribution, Rng};

#[derive(Component)]
struct Tile {
    //size: Size
    //tile: Img
    //state:
    //
}

#[derive(Bundle)]
pub struct TileBundle {
    tile: Tile,
    position: Position,
    color: Color,
    shape: Shape,
}

#[derive(Component)]
struct Position;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Blue,
    Red,
    Yellow,
}

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
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

struct Grid;
