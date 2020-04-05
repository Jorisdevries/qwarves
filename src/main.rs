use quicksilver::prelude::*;

use std::collections::HashMap;
//use rand::Rng;
use std::cmp;

pub mod map;

static WINDOW_WIDTH: i32 = 1200;
static WINDOW_HEIGHT: i32 = 800;

static HALF_CAMERA_WIDTH: i32 = 18;
static HALF_CAMERA_HEIGHT: i32 = 12;

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
}

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
    title: Asset<Image>,
    cascadia_font_info: Asset<Image>,
    square_font_info: Asset<Image>,
    inventory: Asset<Image>,
    map_size: Vector,
    map: Vec<map::Tile>,
    entities: Vec<Entity>,
    player_id: usize,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
    camera_window: Vector,
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

fn render_text(window: &mut Window, text: &'static str, x_pos: i32, y_pos: i32) -> Result<()> {
    let mut test_draw = Asset::new(Font::load("Cascadia.ttf").and_then(move |font| {
        font.render(
            text,
            &FontStyle::new(20.0, Color::BLACK),
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

fn get_camera_origin(player_position: Vector, object_position: Vector, camera_window: Vector, map_size: Vector) -> Vector {

    let mut mapped_x = object_position.x - (player_position.x - camera_window.x);
    let mut mapped_y = object_position.y - (player_position.y - camera_window.y);

    if player_position.x <= camera_window.x {
        mapped_x = object_position.x;
    }

    if player_position.x >= map_size.x - camera_window.x {
        mapped_x = object_position.x - ((map_size.x - camera_window.x) - camera_window.x); 
    }

    if player_position.y <= camera_window.y {
        mapped_y = object_position.y;
    }

    if player_position.y >= map_size.y - camera_window.y {
        mapped_y = object_position.y - ((map_size.y - camera_window.y) - camera_window.y); 
    }

    let mapped_pos = Vector::new(mapped_x, mapped_y);

    mapped_pos
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
        let font_cascadia = "Cascadia.ttf";

        let title = Asset::new(Font::load(font_cascadia).and_then(|font| {
            font.render("Rust Roguelike", &FontStyle::new(72.0, Color::BLACK))
        }));

        let cascadia_font_info = Asset::new(Font::load(font_cascadia).and_then(|font| {
            font.render(
                "Footer 1",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let square_font_info = Asset::new(Font::load(font_cascadia).and_then(move |font| {
            font.render(
                "Footer 2",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let inventory = Asset::new(Font::load(font_cascadia).and_then(move |font| {
            font.render(
                "Inventory:\n[A] Sword\n[B] Shield\n[C] Darts",
                &FontStyle::new(20.0, Color::BLACK),
            )
        }));

        let map_size = Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT);
        let camera_window = Vector::new(HALF_CAMERA_WIDTH, HALF_CAMERA_HEIGHT);

        let map = map::generate_map(map_size);
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
        let tile_size_px = Vector::new(24, 24);
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
            title,
            cascadia_font_info,
            square_font_info,
            inventory,
            map_size,
            map,
            entities,
            player_id,
            tileset,
            tile_size_px,
            camera_window,
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        use ButtonState::*;

        let player = &mut self.entities[self.player_id];

        if window.keyboard()[Key::Left] == Pressed && player.pos.x > 0.0 {
            let perform_move = test_collision(&mut self.map, player.pos.x - 1.0, player.pos.y);

            if perform_move {
                player.pos.x -= 1.0;
            }
        }
        if window.keyboard()[Key::Right] == Pressed && player.pos.x < (map::MAP_WIDTH - 1) as f32 {
            let perform_move = test_collision(&mut self.map, player.pos.x + 1.0, player.pos.y);

            if perform_move {
                player.pos.x += 1.0;
            }
        }
        if window.keyboard()[Key::Up] == Pressed && player.pos.y > 0.0 {
            let perform_move = test_collision(&mut self.map, player.pos.x, player.pos.y - 1.0);

            if perform_move {
                player.pos.y -= 1.0;
            }
        }
        if window.keyboard()[Key::Down] == Pressed && player.pos.y < (map::MAP_HEIGHT - 1) as f32 {
            let perform_move = test_collision(&mut self.map, player.pos.x, player.pos.y + 1.0);

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

        render_text(window, "From function!", 2, window.screen_size().y as i32 - 90)?;
        render_text(window, "From function 2!", 2, window.screen_size().y as i32 - 120)?;

        // Draw the cascadia font credits
        self.cascadia_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 60)),
                Img(&image),
            );
            Ok(())
        })?;

        // Draw the Square font credits
        self.square_font_info.execute(|image| {
            window.draw(
                &image
                    .area()
                    .translate((2, window.screen_size().y as i32 - 30)),
                Img(&image),
            );
            Ok(())
        })?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new(50, 120);

        // Draw the map
        let (tileset, map, entities) = (&mut self.tileset, &self.map, &self.entities);
        let player = &self.entities[self.player_id];
        let camera_window = &self.camera_window;

        let map_size_px = self.map_size.times(tile_size_px);

        tileset.execute(|tileset| {
            for tile in map.iter() {

                let camera_origin = get_camera_origin(player.pos, tile.pos, *camera_window, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));

                // outside of camera range
                if tile.pos.x < camera_origin.x - camera_window.x ||
                    tile.pos.x > camera_origin.x + camera_window.x ||
                    tile.pos.y < camera_origin.y - camera_window.y ||
                    tile.pos.y > camera_origin.y + camera_window.y {
                    continue;
                }

                let px_pos = offset_px + camera_origin.times(tile_size_px);
                // outside of x or y margin range
                if px_pos.x < offset_px.x ||
                    px_pos.x > map_size_px.x - offset_px.x ||
                    px_pos.y < offset_px.y ||
                    px_pos.y > map_size_px.y - offset_px.y {
                    continue;
                }

                if let Some(image) = tileset.get(&tile.glyph) {
                    let pos_px = camera_origin.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(offset_px + pos_px, image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        tileset.execute(|tileset| {
            for entity in entities.iter() {

                let camera_origin = get_camera_origin(player.pos, entity.pos, *camera_window, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));

                if entity.pos.x < camera_origin.x - camera_window.x ||
                    entity.pos.x > camera_origin.x + camera_window.x ||
                    entity.pos.y < camera_origin.y - camera_window.y ||
                    entity.pos.y > camera_origin.y + camera_window.y {
                    continue;
                }

                if let Some(image) = tileset.get(&entity.glyph) {
                    let pos_px = offset_px + camera_origin.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(pos_px, image.area().size()),
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
    std::env::set_var("WINIT_HIDPI_FACTOR", "1.0");

    let settings = Settings {
        // If the graphics do need to be scaled (e.g. using
        // `with_center`), blur them. This looks better with fonts.
        //scale: quicksilver::graphics::ImageScaleStrategy::Blur,
        ..Default::default()
    };
    run::<Game>("Qwarves", Vector::new(WINDOW_WIDTH, WINDOW_HEIGHT), settings);
}
