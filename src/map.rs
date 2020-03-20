use quicksilver::prelude::*;
use rand::Rng;

pub static MAP_WIDTH: u32 = 40;
pub static MAP_HEIGHT: u32 = 32;
pub static TILE_HEALTH: f32 = 5.0;

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
    pub health: f32,
}

#[derive(Debug)]
pub struct TileNeighbourhood {
    number_filled: usize,
    number_empty: usize,
    size: usize,
}

pub fn get_index_edge(index: usize) -> Edge {
    let in_left_edge: bool = index < MAP_HEIGHT as usize;
    let in_right_edge: bool = index > ((MAP_WIDTH * MAP_HEIGHT) - MAP_HEIGHT - 1) as usize;
    let in_top_edge: bool = index == 0 || index % MAP_HEIGHT as usize == 0;
    let in_bottom_edge: bool = index != 0 && ((index + 1) % MAP_HEIGHT as usize) as usize == 0;

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
            Direction::Up => index - 1,
            Direction::Down => index + 1,
            Direction::Left => index - MAP_HEIGHT as usize,
            Direction::Right => index + MAP_HEIGHT as usize
        }
    } 
}

pub fn get_tile_neighbourhood(map: &Vec<Tile>, index: usize, glyph_type: char) -> TileNeighbourhood {
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

    let (mut number_filled, mut number_empty, mut n_size): (usize, usize, usize) = (0, 0, index_vector.len());

    for i in index_vector.iter() {
        if *i == index {
            n_size -= 1;
            continue;
        }

        //println!("> index {} has glyph {}", i, map[*i].glyph);
        if map[*i].glyph == glyph_type {
            number_filled += 1;
        } else {
            number_empty += 1; 
        }
    }

    TileNeighbourhood {
        number_filled: number_filled,
        number_empty: number_empty,
        size: n_size,
    }
}

pub fn position_to_index(x_coordinate: f32, y_coordinate: f32) -> usize {
    let index: usize = (x_coordinate as usize * MAP_HEIGHT as usize) + (y_coordinate as usize);
    index
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
                health: 0.0
            };

            let random_number: u32 = rng.gen_range(1, 100);
            
            if random_number <= 40 {
                tile.glyph = '#';
                tile.health = TILE_HEALTH;
            }

            map.push(tile);
        }
    }

    let number_loops: u16 = 7;

    for _loop in 0..number_loops {
        let mut indices_to_fill = Vec::<usize>::new();
        let mut indices_to_empty = Vec::<usize>::new();

        for i in 0..map.len() {
            let neighbourhood: TileNeighbourhood = get_tile_neighbourhood(&map, i, '#');
            //println!("Tile with index {} and glyph {} has neighbourhood {:?}", i, map[i].glyph, neighbourhood);

            if (map[i].glyph == '#' && neighbourhood.number_filled >= 4)
                || (map[i].glyph == '.' && neighbourhood.number_filled >= 5)  
                || (_loop <= 4 && map[i].glyph == '.' && neighbourhood.number_filled <= 0)  
                || get_index_edge(i) != Edge::None { 
               indices_to_fill.push(i); 
            } else {
                indices_to_empty.push(i);
            } 
        }

        for i in indices_to_fill.iter() {
            map[*i].glyph = '#';
            map[*i].health = TILE_HEALTH;
        }

        for i in indices_to_empty.iter() {
            map[*i].glyph = '.';
            map[*i].health = 0.0;
        }
    }

    map
}

