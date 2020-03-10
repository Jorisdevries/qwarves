use quicksilver::prelude::*;

use std::collections::HashMap;
//use rand::Rng;
//use std::cmp;

pub mod map;

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
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

struct Item {
    name: String,
}

struct Inventory {
    size: i32,
    contents: Vec<Item>,
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

        let map_size = Vector::new(40, 32);
        let camera_window = Vector::new(10, 8);

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
        let game_glyphs = "#@g.%|_";
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
        if window.keyboard()[Key::Left] == Pressed {
            player.pos.x -= 1.0;
        }
        if window.keyboard()[Key::Right] == Pressed {
            player.pos.x += 1.0;
        }
        if window.keyboard()[Key::Up] == Pressed {
            player.pos.y -= 1.0;
        }
        if window.keyboard()[Key::Down] == Pressed {
            player.pos.y += 1.0;
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

        tileset.execute(|tileset| {
            for tile in map.iter() {

                if tile.pos.x < player.pos.x - camera_window.x ||
                    tile.pos.x > player.pos.x + camera_window.x ||
                    tile.pos.y < player.pos.y - camera_window.y ||
                    tile.pos.y > player.pos.y + camera_window.y {
                    continue;
                }

                let mapped_x = tile.pos.x - (player.pos.x - camera_window.x);
                let mapped_y = tile.pos.y - (player.pos.y - camera_window.y);
                let mapped_pos = Vector::new(mapped_x, mapped_y);

                if let Some(image) = tileset.get(&tile.glyph) {
                    let pos_px = mapped_pos.times(tile_size_px);
                    window.draw(
                        &Rectangle::new(offset_px + pos_px, image.area().size()),
                        Blended(&image, tile.color),
                    );
                }
            }
            Ok(())
        })?;

        // Draw entities
        //let (tileset, entities) = (&mut self.tileset, &self.entities);

        tileset.execute(|tileset| {
            for entity in entities.iter() {
                if entity.pos.x < player.pos.x - camera_window.x ||
                    entity.pos.x > player.pos.x + camera_window.x ||
                    entity.pos.y < player.pos.y - camera_window.y ||
                    entity.pos.y > player.pos.y + camera_window.y {
                    continue;
                }

                let mapped_x = entity.pos.x - (player.pos.x - camera_window.x);
                let mapped_y = entity.pos.y - (player.pos.y - camera_window.y);
                let mapped_pos = Vector::new(mapped_x, mapped_y);

                if let Some(image) = tileset.get(&entity.glyph) {
                    let pos_px = offset_px + mapped_pos.times(tile_size_px);
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

        let map_size_px = self.map_size.times(tile_size_px);
        let health_bar_pos_px = offset_px + Vector::new(map_size_px.x, 0.0);

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
    run::<Game>("Quicksilver Roguelike", Vector::new(800, 600), settings);
}
