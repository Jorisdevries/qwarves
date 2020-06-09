use quicksilver::prelude::*;
use specs::{Builder, Entity, World};
use specs::prelude::*;

use rand::Rng;
use std::collections::HashMap;

pub static MAP_WIDTH: u32 = 60;
pub static MAP_HEIGHT: u32 = 50;
pub static TILE_HEALTH: f32 = 1.0;

use crate::components;

#[derive(Default)]
pub struct Map {
    pub tiles: HashMap<(i32, i32), Entity>,
}

impl Map {
    pub fn new() -> Map {
        Map {
            tiles: HashMap::new(),
        }
    }
}

pub fn generate_tile(ecs: &mut World, glyph: char, pos_x: i32, pos_y: i32) -> Entity {
    let entity = ecs
    .create_entity()
    .with(components::Position { x: pos_x, y: pos_y})
    .with(components::Renderable {
        glyph: glyph,
        color: Color::BLACK,
    })
    .with(components::Tile{})
    .build();

    entity
}


pub fn generate_map_new(ecs: &mut World, size: Vector) -> Map {
    let width = size.x as usize;
    let height = size.y as usize;
    
    let mut map = Map::new(); 
    let mut rng = rand::thread_rng();

    for x in 0..width {
        for y in 0..height {
            
            let mut glyph = '.';

            let random_number: u32 = rng.gen_range(1, 100);
            if random_number <= 45 {
                glyph = '#';
            }

            let tile = generate_tile(ecs, glyph, x as i32, y as i32);
            map.tiles.insert((x as i32, y as i32), tile);
        }
    }

    //ecs.insert(map);
    map
}

pub fn count_surrounding(coords: (i32, i32), tile_map: &HashMap<(i32, i32), Entity>, renderables: &ReadStorage<components::Renderable>) -> i32 {
    let mut n_surrounding = 0;

    for x in -1..=1 {
        for y in -1..=1 {
            if let Some(tile_id) = tile_map.get(&(coords.0 + x, coords.1 + y)) {
                if renderables.get(*tile_id).is_some() {
                    if renderables.get(*tile_id).unwrap().glyph == '#' {
                        n_surrounding +=1 ;
                    }
                }
            }
        }
    }

    n_surrounding
}

pub fn get_fill_data(ecs: &mut World, tile_map: &HashMap<(i32, i32), Entity>) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let mut coordinates_to_fill: Vec<(i32, i32)> = Vec::new();
    let mut coordinates_to_empty: Vec<(i32, i32)> = Vec::new();

    let positions = ecs.read_storage::<components::Position>();
    let renderables = ecs.read_storage::<components::Renderable>();
    let tiles = ecs.read_storage::<components::Tile>();

    for (pos, render, _tile) in (&positions, &renderables, &tiles).join() {
        let n_solid = count_surrounding((pos.x, pos.y), tile_map, &renderables); 

        if (render.glyph == '#' && n_solid >= 4) || (render.glyph == '.' && n_solid >= 5) { 
            coordinates_to_fill.push((pos.x, pos.y)); 
        } else {
            coordinates_to_empty.push((pos.x, pos.y));
        } 
    }

    (coordinates_to_fill, coordinates_to_empty)
}

pub fn apply_ca(ecs: &mut World, tile_map: &HashMap<(i32, i32), Entity>) {
    let fill_info = get_fill_data(ecs, tile_map);
    let coordinates_to_fill: Vec<(i32, i32)> = fill_info.0;
    let coordinates_to_empty: Vec<(i32, i32)> = fill_info.1;

    let mut positions = ecs.write_storage::<components::Position>();
    let mut renderables = ecs.write_storage::<components::Renderable>();
    let mut tiles = ecs.write_storage::<components::Tile>();

    for (pos, render, _tile) in (&mut positions, &mut renderables, &mut tiles).join() {
        if coordinates_to_fill.iter().any(|&i| i == (pos.x, pos.y)) {
            render.glyph = '#';
        } else if coordinates_to_empty.iter().any(|&i| i == (pos.x, pos.y)) {
            render.glyph = '.';
        }
    }
}