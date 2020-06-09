use quicksilver::prelude::*;
use specs::prelude::*;
use specs::{Builder, World};

use std::collections::HashMap;
//use rand::seq::SliceRandom; 
use rand::Rng;
use std::cmp;

pub mod map;
pub mod components;

static TILE_EDGE_PIXELS: i32 = 24;
static WINDOW_WIDTH_TILES: i32 = 50;
static WINDOW_HEIGHT_TILES: i32 = 28;

static LEFT_OFFSET_TILES: i32 = 4;
static RIGHT_OFFSET_TILES: i32 = 8;
static TOP_OFFSET_TILES: i32 = 2;
static BOTTOM_OFFSET_TILES: i32 = 2;


impl<'a> System<'a> for components::RandomMover {
    type SystemData = (ReadStorage<'a, components::RandomMover>, 
                        WriteStorage<'a, components::Position>);

    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty,pos) in (&lefty, &mut pos).join() {
            let mut rng = rand::thread_rng();

            if rng.gen_range(1, 61) == 60 {
                pos.x += rng.gen_range(-1, 2) as i32;
                pos.y += rng.gen_range(-1, 2) as i32;
            }
        }
    }
}

struct Game {
    inventory: Asset<Image>,
    map_size: Vector,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
    ecs: World
}

fn generate_entities(ecs: &mut World) {
    ecs
    .create_entity()
    .with(components::Position { x: 40, y: 25 })
    .with(components::Renderable {
        glyph: '@',
        color: Color::BLACK,
    })
    .with(components::Player{})
    .build();

    ecs
    .create_entity()
    .with(components::Position { x: 30, y: 10 })
    .with(components::Renderable {
        glyph: 'g',
        color: Color::GREEN,
    })
    .with(components::RandomMover{})
    .build();
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

    if player_position.x >= -2.0 + map_size.x - window_half_width as f32 {
        translate_x = window_half_width as f32 - (-2.0 + map_size.x - window_half_width as f32);
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

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &World) {
    let mut positions = ecs.write_storage::<components::Position>();
    let mut players = ecs.write_storage::<components::Player>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = cmp::min(map::MAP_WIDTH as i32, cmp::max(0, pos.x + delta_x));
        pos.y = cmp::min(map::MAP_HEIGHT as i32, cmp::max(0, pos.y + delta_y));
    }
}

fn player_input(ecs: &World, window: &mut Window) {
    use ButtonState::*;

    if window.keyboard()[Key::Left] == Pressed {
        try_move_player(-1, 0, ecs);
    }
    if window.keyboard()[Key::Right] == Pressed {
        try_move_player(1, 0, ecs);
    }
    if window.keyboard()[Key::Up] == Pressed {
        try_move_player(0, -1, ecs);
    }
    if window.keyboard()[Key::Down] == Pressed {
        try_move_player(0, 1, ecs);
    }
    if window.keyboard()[Key::Escape].is_down() {
        window.close();
    }
}

fn run_systems(ecs: &mut World) {
    let mut rw = components::RandomMover{};
    rw.run_now(ecs);
    ecs.maintain();
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

        let mut ecs = World::new();
        ecs.register::<components::Position>();
        ecs.register::<components::Renderable>();
        ecs.register::<components::Player>();
        ecs.register::<components::RandomMover>();
        ecs.register::<components::Tile>();

        generate_entities(&mut ecs);
        let map_size = Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT);
        let new_map = map::generate_map_new(&mut ecs, map_size);

        for _loop in 1..7 {
            map::apply_ca(&mut ecs, &new_map.tiles);
        }

        Ok(Self {
            inventory,
            map_size,
            tileset,
            tile_size_px,
            ecs,
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {
        run_systems(&mut self.ecs);
        player_input(&self.ecs, window);

        Ok(())
    }


    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        render_text(window, "Test title", window.screen_size().x as i32 / 2 - 10, 25, 40.0)?;
        render_text(window, "From function!", 2, window.screen_size().y as i32 - 90, 20.0)?;
        render_text(window, "From function 2!", 2, window.screen_size().y as i32 - 120, 20.0)?;

        let tile_size_px = self.tile_size_px;
        let offset_px = Vector::new((LEFT_OFFSET_TILES - 1) * TILE_EDGE_PIXELS, TOP_OFFSET_TILES * TILE_EDGE_PIXELS);

        // Draw the map
        let tileset = &mut self.tileset;

        let positions = self.ecs.read_storage::<components::Position>();
        let renderables = self.ecs.read_storage::<components::Renderable>();
        let players = self.ecs.read_storage::<components::Player>();

        let mut player_pos: Vector = Vector::new(0, 0);
        for (pos, _player) in (&positions, &players).join() {
            player_pos = Vector::new(pos.x, pos.y);
        }

        tileset.execute(|tileset| {
            for (pos, render) in (&positions, &renderables).join() {
                let position = Vector::new(pos.x, pos.y);
                let mapped_position = camera_translate(player_pos, position, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));
                let px_pos = offset_px + mapped_position.times(tile_size_px);

                if !should_render(mapped_position) {
                    continue;
                }

                if let Some(image) = tileset.get(&render.glyph) {
                    window.draw(
                        &Rectangle::new(px_pos, image.area().size()),
                        Blended(&image, render.color),
                    );
                }
            }

            Ok(())
        })?;

        let full_health_width_px = 100.0;
        let current_health_width_px = (50 as f32 / 100 as f32) * full_health_width_px;

        let screen_width_tiles = WINDOW_WIDTH_TILES - RIGHT_OFFSET_TILES - 2;

        let health_bar_pos_px = offset_px + Vector::new(screen_width_tiles * TILE_EDGE_PIXELS, 0.0);
        let mana_bar_pos_px = offset_px + Vector::new(screen_width_tiles * TILE_EDGE_PIXELS, -30.0);

        render_bar(window, Color::RED, current_health_width_px, health_bar_pos_px, full_health_width_px, tile_size_px.y)?;
        render_bar(window, Color::BLUE, current_health_width_px, mana_bar_pos_px, full_health_width_px, tile_size_px.y)?;

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
