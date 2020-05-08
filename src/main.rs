use quicksilver::prelude::*;

use std::collections::HashMap;
//use rand::Rng;
//use std::cmp;

pub mod map;

static TILE_EDGE_PIXELS: i32 = 24;
static WINDOW_WIDTH_TILES: i32 = 50;
static WINDOW_HEIGHT_TILES: i32 = 28;

static LEFT_OFFSET_TILES: i32 = 4;
static RIGHT_OFFSET_TILES: i32 = 4;
static TOP_OFFSET_TILES: i32 = 2;
static BOTTOM_OFFSET_TILES: i32 = 2;

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
}

// TO DO:
// Multiple levels
// Hero view

impl Entity {
    fn move_pos(&mut self, delta_x: f32, delta_y: f32) {
            self.pos.x += delta_x;
            self.pos.y += delta_y;
    }
}

fn generate_entities() -> Vec<Entity> {
    vec![
        Entity {
            pos: Vector::new(9, 6),
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
        },
        Entity {
            pos: Vector::new(2, 4),
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
        },
        Entity {
            pos: Vector::new(7, 5),
            glyph: '%',
            color: Color::PURPLE,
            hp: 0,
            max_hp: 0,
        },
        Entity {
            pos: Vector::new(4, 8),
            glyph: '%',
            color: Color::PURPLE,
            hp: 0,
            max_hp: 0,
        },
    ]
}

struct Game {
    inventory: Asset<Image>,
    map_size: Vector,
    //map: Vec<map::Tile>,
    levels: Vec<Vec<map::Tile>>,
    level_index: usize,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
}

fn test_collision(map: &mut Vec<map::Tile> , new_x_position: f32, new_y_position: f32) -> bool {
    let new_index: usize = map::position_to_index(new_x_position, new_y_position);
    let collision_tile: &mut map::Tile = &mut map[new_index];
    let perform_move: bool = collision_tile.health <= 0.0;

    if collision_tile.health > 0.0 {
        collision_tile.health -= 1.0;
        collision_tile.glyph = '*';
    }

    if collision_tile.health <= 0.0 {
        collision_tile.glyph = '.';
    }

    return perform_move;    
}

fn render_text(window: &mut Window, text: &'static str, x_pos: i32, y_pos: i32, font_size: f32) -> Result<()> {
    let mut test_draw = Asset::new(Font::load("Cascadia.ttf").and_then(move |font| {
        font.render(
            text,
            &FontStyle::new(font_size, Color::BLACK),
        )
    }));

    test_draw.execute(|image| {
        window.draw(
            &image
                .area()
                .translate((x_pos, y_pos)),
            Img(&image),
        );
        Ok(())
    })?;

    Ok(())
}

fn render_bar(window: &mut Window, colour: Color, current_value: f32, origin: Vector, width: f32, height: f32) -> Result<()> {
    // Full health
    window.draw(
        &Rectangle::new(origin, (width, height)),
        Col(colour.with_alpha(0.5)),
    );

    // Current health
    window.draw(
        &Rectangle::new(origin, (current_value, height)),
        Col(colour),
    );

    Ok(())
}

fn camera_translate(player_position: Vector, object_position: Vector, map_size: Vector) -> Vector {
    let window_half_width = (WINDOW_WIDTH_TILES - LEFT_OFFSET_TILES - RIGHT_OFFSET_TILES) / 2;
    let window_half_height = (WINDOW_HEIGHT_TILES - TOP_OFFSET_TILES - BOTTOM_OFFSET_TILES) / 2;
    
    let mut translate_x = window_half_width as f32 - player_position.x; 
    let mut translate_y = window_half_height as f32 - player_position.y; 

    if player_position.x <= window_half_width as f32 {
        translate_x = 0.0;
    }

    if player_position.x >= map_size.x - window_half_width as f32 {
        translate_x = window_half_width as f32 - (map_size.x - window_half_width as f32);
    } 

    if player_position.y <= window_half_height as f32 {
        translate_y = 0.0;
    }

    if player_position.y >= map_size.y - window_half_height as f32 {
        translate_y = window_half_height as f32 - (map_size.y - window_half_height as f32);
    } 

    let result = Vector::new(object_position.x + translate_x, object_position.y + translate_y); 
    result
}

fn should_render(mapped_position: Vector) -> bool {

    // outside of x or y margin range
    if mapped_position.x < 0.0 ||
    mapped_position.x > (WINDOW_WIDTH_TILES - LEFT_OFFSET_TILES - RIGHT_OFFSET_TILES + 1) as f32 ||
    mapped_position.y < 0.0 ||
    mapped_position.y > (WINDOW_HEIGHT_TILES - BOTTOM_OFFSET_TILES - TOP_OFFSET_TILES - 1) as f32 {
        return false;
    }

    true
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let font_cascadia = "Cascadia.ttf";

        let inventory = Asset::new(Font::load(font_cascadia).and_then(move |font| {
            font.render(
                "Inventory:\n[A] Sword\n[B] Shield\n[C] Darts",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let map_size = Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT);

        let mut level_index = 0;
        let levels = map::generate_levels(5, map_size);
        //let map = map::generate_map(map_size);
        //let map = &levels[level_index];
        let mut entities = generate_entities();
        let player_id = entities.len();

        entities.push(Entity {
            pos: Vector::new(5, 3),
            glyph: '@',
            color: Color::BLUE,
            hp: 3,
            max_hp: 5,
        });

        let font_square = "Square.ttf";
        let game_glyphs = "#@g.%|_o*";
        let tile_size_px = Vector::new(TILE_EDGE_PIXELS, TILE_EDGE_PIXELS);
        let tileset = Asset::new(Font::load(font_square).and_then(move |text| {
            let tiles = text
                .render(game_glyphs, &FontStyle::new(tile_size_px.y, Color::WHITE))
                .expect("Could not render the font tileset.");
            let mut tileset = HashMap::new();
            for (index, glyph) in game_glyphs.chars().enumerate() {
                let pos = (index as i32 * tile_size_px.x as i32, 0);
                let tile = tiles.subimage(Rectangle::new(pos, tile_size_px));
                tileset.insert(glyph, tile);
            }
            Ok(tileset)
        }));

        Ok(Self {
            inventory,
            map_size,
            //map,
            levels,
            level_index,
            entities,
            player_id,
            tileset,
            tile_size_px,
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        use ButtonState::*;

        let player = &mut self.entities[self.player_id];

        if window.keyboard()[Key::L] == Pressed {
            self.level_index += 1; 
        }

        if window.keyboard()[Key::Left] == Pressed && player.pos.x > 0.0 {
            let perform_move = test_collision(&mut self.levels[self.level_index], player.pos.x - 1.0, player.pos.y);

            if perform_move {
                player.pos.x -= 1.0;
            }
        }
        if window.keyboard()[Key::Right] == Pressed && player.pos.x < (map::MAP_WIDTH - 1) as f32 {
            let perform_move = test_collision(&mut self.levels[self.level_index], player.pos.x + 1.0, player.pos.y);

            if perform_move {
                player.pos.x += 1.0;
            }
        }
        if window.keyboard()[Key::Up] == Pressed && player.pos.y > 0.0 {
            let perform_move = test_collision(&mut self.levels[self.level_index], player.pos.x, player.pos.y - 1.0);

            if perform_move {
                player.pos.y -= 1.0;
            }
        }
        if window.keyboard()[Key::Down] == Pressed && player.pos.y < (map::MAP_HEIGHT - 1) as f32 {
            let perform_move = test_collision(&mut self.levels[self.level_index], player.pos.x, player.pos.y + 1.0);

            if perform_move {
                player.pos.y += 1.0;
            }
        }
        if window.keyboard()[Key::Escape].is_down() {
            window.close();
        }
        Ok(())
    }


    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        /*
        // Draw the game title
        self.title.execute(|image| {
            window.draw(
                &image
                    .area()
                    .with_center((window.screen_size().x as i32 / 2, 40)),
                Img(&image),
            );
            Ok(())
        })?;
        */

        render_text(window, "Test title", window.screen_size().x as i32 / 2, 40, 40.0)?;
        render_text(window, "From function!", 2, window.screen_size().y as i32 - 90, 20.0)?;
        render_text(window, "From function 2!", 2, window.screen_size().y as i32 - 120, 20.0)?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new((LEFT_OFFSET_TILES - 1) * TILE_EDGE_PIXELS, TOP_OFFSET_TILES * TILE_EDGE_PIXELS);

        // Draw the map
        let (tileset, map, entities) = (&mut self.tileset, &mut self.levels[self.level_index], &self.entities);
        let player = &self.entities[self.player_id];

        let map_size_px = self.map_size.times(tile_size_px);

        tileset.execute(|tileset| {
            for tile in map.iter() {

                let mapped_position = camera_translate(player.pos, tile.pos, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));
                let px_pos = offset_px + mapped_position.times(tile_size_px);

                if !should_render(mapped_position) {
                    continue;
                }

                if let Some(image) = tileset.get(&tile.glyph) {
                    window.draw(
                        &Rectangle::new(px_pos, image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        tileset.execute(|tileset| {
            for entity in entities.iter() {

                let mapped_position = camera_translate(player.pos, entity.pos, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));
                let px_pos = offset_px + mapped_position.times(tile_size_px);

                if !should_render(mapped_position) {
                    continue;
                }

                if let Some(image) = tileset.get(&entity.glyph) {
                    window.draw(
                        &Rectangle::new(px_pos, image.area().size()),
                        Blended(&image, entity.color),
                    );
                }
            }
            Ok(())
        })?;

        let player = &self.entities[self.player_id];
        let full_health_width_px = 100.0;
        let current_health_width_px =
            (player.hp as f32 / player.max_hp as f32) * full_health_width_px;

        let health_bar_pos_px = offset_px + Vector::new(map_size_px.x, 0.0);
        let mana_bar_pos_px = offset_px + Vector::new(map_size_px.x, -30.0);

        render_bar(window, Color::RED, current_health_width_px, health_bar_pos_px, full_health_width_px, tile_size_px.y)?;
        render_bar(window, Color::BLUE, current_health_width_px, mana_bar_pos_px, full_health_width_px, tile_size_px.y)?;

        /*
        // Full health
        window.draw(
            &Rectangle::new(health_bar_pos_px, (full_health_width_px, tile_size_px.y)),
            Col(Color::RED.with_alpha(0.5)),
        );

        // Current health
        window.draw(
            &Rectangle::new(health_bar_pos_px, (current_health_width_px, tile_size_px.y)),
            Col(Color::RED),
        );
        */

        self.inventory.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate(health_bar_pos_px + Vector::new(0, tile_size_px.y)),
                Img(&image),
            );
            Ok(())
        })?;

        Ok(())
    }
}

fn main() {
    // NOTE: Set HIDPI to 1.0 to get pixel-perfect rendering.
    // Otherwise the window resizes to whatever value the OS sets and
    // scales the contents.
    // https://docs.rs/glutin/0.19.0/glutin/dpi/index.html
    std::env::set_var("WINIT_HIDPI_FACTOR", "2.0");

    let settings = Settings {
        // If the graphics do need to be scaled (e.g. using
        // `with_center`), blur them. This looks better with fonts.
        scale: quicksilver::graphics::ImageScaleStrategy::Blur,
        ..Default::default()
    };
    run::<Game>("Qwarves", Vector::new(WINDOW_WIDTH_TILES * TILE_EDGE_PIXELS, WINDOW_HEIGHT_TILES * TILE_EDGE_PIXELS), settings);
}
