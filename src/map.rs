use quicksilver::prelude::*;
use specs::{Builder, Entity, World};
use specs::prelude::*;
use rltk::{ BaseMap, Algorithm2D, Point };

use rand::Rng;
use std::collections::HashMap;

use crate::components;

#[derive(Default)]
pub struct Map {
    pub tiles: HashMap<(i32, i32), Entity>,
    pub glyph_map: Vec<char>,
    pub revealed_map: Vec<bool>,
    pub visible_map: Vec<bool>,
    pub blocked : Vec<bool>,
    pub index_to_position_map: HashMap<usize, (i32, i32)>,
    pub position_to_index_map: HashMap<(i32, i32), usize>,
    pub width: i32,
    pub height: i32
}

impl Map {
    pub fn new(width: i32, height: i32) -> Map {
        Map {
            tiles: HashMap::new(),
            glyph_map: vec!['?'; (width * height) as usize],
            revealed_map: vec![false; (width * height) as usize],
            visible_map: vec![false; (width * height) as usize],
            blocked: vec![false; (width * height) as usize],
            index_to_position_map: HashMap::new(),
            position_to_index_map: HashMap::new(),
            width: width,
            height: height
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.glyph_map.iter_mut().enumerate() {
            self.blocked[i] = *tile == '#';
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 { 
            return false; 
        }

        let idx = self.position_to_index_map[&(x, y)];
        !self.blocked[idx]
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.glyph_map[idx] == '#' 
    }

    fn get_available_exits(&self, idx:usize) -> Vec<(usize, f32)> {
        let mut exits : Vec<(usize, f32)> = Vec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
    
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };
    
        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }

    fn index_to_point2d(&self, index: usize) -> rltk::Point {
        let coords = self.index_to_position_map[&index];
        rltk::Point::new(coords.0, coords.1)
    }

    fn point2d_to_index(&self, point: Point) -> usize {
        let coords = (point.x, point.y);
        let index = self.position_to_index_map[&coords];
        index
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


pub fn generate_map_new(ecs: &mut World, size: Vector) {
    let width = size.x as usize;
    let height = size.y as usize;
    
    let mut map = Map::new(size.x as i32, size.y as i32); 
    let mut rng = rand::thread_rng();
    let mut index = 0;

    for x in 0..width {
        for y in 0..height {
            let mut glyph = '.';

            let random_number: u32 = rng.gen_range(1, 100);
            if random_number <= 45 {
                glyph = '#';
            }

            let tile = generate_tile(ecs, glyph, x as i32, y as i32);
            
            let coords = (x as i32, y as i32);
            map.tiles.insert(coords, tile);
            map.glyph_map[index] = glyph;

            map.index_to_position_map.insert(index as usize, coords);
            map.position_to_index_map.insert(coords, index as usize);
            map.revealed_map[index] = false;
            map.visible_map[index] = false;

            index += 1;
        }
    }

    ecs.insert(map);
    //map
}

pub fn count_surrounding(coords: (i32, i32), tile_map: &HashMap<(i32, i32), Entity>, renderables: &ReadStorage<components::Renderable>) -> i32 {
    let mut n_surrounding = 0;

    for x in -1..=1 {
        for y in -1..=1 {
            if let Some(tile_id) = tile_map.get(&(coords.0 + x, coords.1 + y)) {
                if renderables.get(*tile_id).is_some() {
                    if renderables.get(*tile_id).unwrap().glyph == '#' {
                        n_surrounding += 1 ;
                    }
                }
            }
        }
    }

    n_surrounding
}

pub fn get_fill_data(ecs: &World, tile_map: &HashMap<(i32, i32), Entity>) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let mut coordinates_to_fill: Vec<(i32, i32)> = Vec::new();
    let mut coordinates_to_empty: Vec<(i32, i32)> = Vec::new();

    let positions = ecs.read_storage::<components::Position>();
    let renderables = ecs.read_storage::<components::Renderable>();
    let tiles = ecs.read_storage::<components::Tile>();

    for (pos, render, _tile) in (&positions, &renderables, &tiles).join() {
        let coords = (pos.x, pos.y);
        let n_solid = count_surrounding(coords, tile_map, &renderables); 

        if (render.glyph == '#' && n_solid >= 4) || (render.glyph == '.' && n_solid >= 5) { 
            coordinates_to_fill.push(coords); 
        } else {
            coordinates_to_empty.push(coords);
        } 
    }

    (coordinates_to_fill, coordinates_to_empty)
}

pub fn apply_ca(ecs: &World, map: &mut Map) {
    let fill_info = get_fill_data(ecs, &map.tiles);
    let coordinates_to_fill: Vec<(i32, i32)> = fill_info.0;
    let coordinates_to_empty: Vec<(i32, i32)> = fill_info.1;

    let mut positions = ecs.write_storage::<components::Position>();
    let mut renderables = ecs.write_storage::<components::Renderable>();
    let mut tiles = ecs.write_storage::<components::Tile>();

    for (pos, render, _tile) in (&mut positions, &mut renderables, &mut tiles).join() {
        let coords = (pos.x, pos.y);

        if coordinates_to_fill.iter().any(|&i| i == coords) {
            render.glyph = '#';
        } else if coordinates_to_empty.iter().any(|&i| i == coords) {
            render.glyph = '.';
        }
    }
}