use quicksilver::prelude::*;
use rand::Rng;

pub static MAP_WIDTH: u32 = 40;
pub static MAP_HEIGHT: u32 = 32;

#[derive(PartialEq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Debug)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeftCorner,
    TopRightCorner,
    BottomLeftCorner,
    BottomRightCorner,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub pos: Vector,
    pub glyph: char,
    pub color: Color,
}

pub fn get_index_edge(index: usize) -> Edge {
    let in_left_edge: bool = index < MAP_HEIGHT as usize;
    let in_right_edge: bool = index > ((MAP_WIDTH * MAP_HEIGHT) - MAP_HEIGHT - 1) as usize;
    let in_bottom_edge: bool = index == 0 || index % MAP_HEIGHT as usize == 0;
    let in_top_edge: bool = index != 0 && ((index + 1) % MAP_HEIGHT as usize) as usize == 0;

    if in_left_edge && in_top_edge {
        Edge::TopLeftCorner
    }
    else if in_right_edge && in_top_edge {
        Edge::TopRightCorner
    }
    else if in_left_edge && in_bottom_edge {
        Edge::BottomLeftCorner
    }
    else if in_right_edge && in_bottom_edge {
        Edge::BottomRightCorner
    }
    else if in_left_edge {
        Edge::Left
    } 
    else if in_right_edge {
        Edge::Right
    } 
    else if in_bottom_edge {
        Edge::Bottom
    } 
    else if in_top_edge {
        Edge::Top
    } 
    else {
        Edge::None
    }
}

pub fn valid_move(direction: &Direction, index: usize) -> bool {
    if get_index_edge(index) == Edge::None {
	    return true;
    }

    let invalid_left_move = get_index_edge(index) == Edge::Left && direction == &Direction::Left;
    let invalid_right_move = get_index_edge(index) == Edge::Right && direction == &Direction::Right;
    let invalid_bottom_move = get_index_edge(index) == Edge::Bottom && direction == &Direction::Down;
    let invalid_top_move = get_index_edge(index) == Edge::Top && direction == &Direction::Up;

    let invalid_top_left_move = get_index_edge(index) == Edge::TopLeftCorner && (direction == &Direction::Left || direction == &Direction::Up);
    let invalid_top_right_move = get_index_edge(index) == Edge::TopRightCorner && (direction == &Direction::Right || direction == &Direction::Up);
    let invalid_bottom_left_move = get_index_edge(index) == Edge::BottomLeftCorner && (direction == &Direction::Left || direction == &Direction::Down);
    let invalid_bottom_right_move = get_index_edge(index) == Edge::BottomRightCorner && (direction == &Direction::Right || direction == &Direction::Down);

    !(invalid_left_move || invalid_right_move || invalid_bottom_move || invalid_top_move ||
      invalid_top_left_move || invalid_top_right_move || invalid_bottom_left_move ||invalid_bottom_right_move)
}

pub fn get_move_index(direction: &Direction, index: usize) -> usize {
    if !valid_move(&direction, index)
    {
        index
    } else {
        match direction {
            Direction::Up => index + 1,
            Direction::Down => index - 1,
            Direction::Left => index - MAP_HEIGHT as usize,
            Direction::Right => index + MAP_HEIGHT as usize
        }
    } 
}

pub fn surrounding_tiles_fraction(map: &Vec<Tile>, index: usize, glyph_type: char) -> f32 {
    let left_index: usize = get_move_index(&Direction::Left, index);
    let right_index: usize = get_move_index(&Direction::Right, index);
    let top_index: usize = get_move_index(&Direction::Up, index);
    let bottom_index: usize = get_move_index(&Direction::Down, index);

    let top_left_diagonal_index = get_move_index(&Direction::Left, top_index);
    let top_right_diagonal_index = get_move_index(&Direction::Right, top_index);
    let bottom_left_diagonal_index = get_move_index(&Direction::Left, bottom_index);
    let bottom_right_diagonal_index = get_move_index(&Direction::Right, bottom_index);

    let mut index_vector = vec![left_index, right_index, top_index, 
                                bottom_index, top_left_diagonal_index, top_right_diagonal_index,
			                    bottom_left_diagonal_index, bottom_right_diagonal_index]; 

    index_vector.sort();
    index_vector.dedup();

    let mut glyph_counter: usize = 0;

    for i in index_vector.iter() {
        if map[*i].glyph == glyph_type {
            glyph_counter += 1;
        }
    }

    glyph_counter as f32/index_vector.len() as f32
}

pub fn generate_map(size: Vector) -> Vec<Tile> {
    let width = size.x as usize;
    let height = size.y as usize;
    let mut map = Vec::with_capacity(width * height);

    let mut rng = rand::thread_rng();

    for x in 0..width {
        for y in 0..height {
            let mut tile = Tile {
                pos: Vector::new(x as f32, y as f32),
                glyph: '.',
                color: Color::BLACK,
            };

            let random_number: u32 = rng.gen_range(0, 100);
            
            if x == 0 || x == width - 1 {
                tile.glyph = '|';
            }
            else if y == 0 {
                tile.glyph = '_';
            }
            
            if random_number < 45 {
                tile.glyph = '#';
            }

            map.push(tile);
        }
    }

    let number_loops: u16 = 1;

    println!("Map size: {}", map.len() as i16);

    for _loop in 0..number_loops {
        for i in 0..map.len() {
	    let fill_fraction: f32 = surrounding_tiles_fraction(&map, i, '#');

	    if fill_fraction >= 0.5 {
                map[i].glyph = '#';
            }
            else {
                map[i].glyph = '.';
            }
        }
    }

    map
}

