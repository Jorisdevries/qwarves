
use quicksilver::prelude::*;
use specs::prelude::*;
use specs_derive::Component;

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
    pub range : i32
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct Tile{}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct RandomMover {}