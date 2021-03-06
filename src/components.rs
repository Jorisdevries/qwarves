
use quicksilver::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

pub struct PlayerPosition {
    pub x: i32,
    pub y: i32
}

#[derive(Component, Debug)]
pub struct Name {
    pub name : String
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: char,
    pub color: Color,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles : Vec<rltk::Point>,
    pub range : i32,
    pub dirty : bool
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Tile{}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct RandomMover {}