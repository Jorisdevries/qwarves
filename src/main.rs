use quicksilver::prelude::*;
use specs::prelude::*;
use specs::{Builder, World};

use std::collections::HashMap;
use std::cmp;

pub mod map;
pub mod components;
pub mod systems;

static TILE_EDGE_PIXELS: i32 = 24;
static WINDOW_WIDTH_TILES: i32 = 50;
static WINDOW_HEIGHT_TILES: i32 = 28;

static LEFT_OFFSET_TILES: i32 = 4;
static RIGHT_OFFSET_TILES: i32 = 4;
static TOP_OFFSET_TILES: i32 = 2;
static BOTTOM_OFFSET_TILES: i32 = 2;

struct PlayerPosition {
    x: i32,
    y: i32
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { Paused, Running }

struct ScreenLayout {
    tile_size_pixels: Vector,
    window_size: Vector,
    screen_size: Vector,
    screen_origin: Vector,
    
    left_panel_origin: Vector,
    right_panel_origin: Vector,
    top_panel_origin: Vector,
    bottom_panel_origin: Vector,
}

impl ScreenLayout {
    fn new(tile_size_pixels: Vector, window_size: Vector, screen_size: Vector, screen_origin: Vector) -> ScreenLayout{
            ScreenLayout {
                tile_size_pixels,
                window_size,
                screen_size,
                screen_origin,  

                left_panel_origin: Vector::new(0, 0),
                right_panel_origin: Vector::new(screen_origin.x + screen_size.x, screen_origin.y),
                top_panel_origin: Vector::new(screen_origin.x, 0),
                bottom_panel_origin: Vector::new(screen_origin.x, screen_origin.y + screen_size.y),
            }
    }
}

struct Game {
    tileset: Asset<HashMap<char, Image>>,
    screen_layout: ScreenLayout,
    ecs: World,
    runstate: RunState
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
    .with(components::Viewshed{ visible_tiles : Vec::new(), range : 8 })
    .build();

    ecs.insert(PlayerPosition { x: 40, y: 25 });

    ecs
    .create_entity()
    .with(components::Position { x: 30, y: 10 })
    .with(components::Renderable {
        glyph: 'g',
        color: Color::GREEN,
    })
    .with(components::RandomMover{})
    .with(components::Monster{})
    .build();
}

fn render_text(window: &mut Window, text: &'static str, position: Vector, font_size: f32) -> Result<()> {
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
                .translate((position.x, position.y)),
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

    if player_position.x >=  map_size.x - window_half_width as f32 {
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

fn should_render(mapped_position: Vector, screen_layout: &ScreenLayout) -> bool {
    // outside of x or y margin range
    if mapped_position.x < 0.0 ||
    mapped_position.x > screen_layout.screen_size.x - 1.0 ||
    mapped_position.y < 0.0 ||
    mapped_position.y > screen_layout.screen_size.y - 1.0 {
        return false;
    }

    true
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &World) {
    let mut positions = ecs.write_storage::<components::Position>();
    let mut players = ecs.write_storage::<components::Player>();
    let map = ecs.fetch::<map::Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        pos.x = cmp::min(map.width, cmp::max(0, pos.x + delta_x));
        pos.y = cmp::min(map.height, cmp::max(0, pos.y + delta_y));

        let mut player_position = ecs.write_resource::<PlayerPosition>();
        player_position.x = pos.x;
        player_position.y = pos.y;
    }
}

fn game_input(game: &mut Game, window: &mut Window) {
    if window.keyboard()[Key::P] == ButtonState::Pressed {
        if game.runstate == RunState::Running {
            game.runstate = RunState::Paused; 
        } else {
            game.runstate = RunState::Running; 
        }    
    }

    if window.keyboard()[Key::Escape].is_down() {
        window.close();
    }
}

fn player_input(game: &mut Game, window: &mut Window) {
    use ButtonState::*;

    if window.keyboard()[Key::Left] == Pressed {
        try_move_player(-1, 0, &game.ecs);
    }
    if window.keyboard()[Key::Right] == Pressed {
        try_move_player(1, 0, &game.ecs);
    }
    if window.keyboard()[Key::Up] == Pressed {
        try_move_player(0, -1, &game.ecs);
    }
    if window.keyboard()[Key::Down] == Pressed {
        try_move_player(0, 1, &game.ecs);
    }
    if window.keyboard()[Key::Escape].is_down() {
        window.close();
    }
}

fn register_components(ecs: &mut World) {
    ecs.register::<components::Position>();
    ecs.register::<components::Renderable>();
    ecs.register::<components::Player>();
    ecs.register::<components::RandomMover>();
    ecs.register::<components::Tile>();
    ecs.register::<components::Monster>();
    ecs.register::<components::Viewshed>();
}

fn run_systems(ecs: &mut World) {
    let mut rw = components::RandomMover{};
    rw.run_now(ecs);
    let mut mob = systems::MonsterAI{};
    mob.run_now(ecs);
    let mut vis = systems::VisibilitySystem{};
    vis.run_now(ecs);
    let mut gm = systems::GlyphMapper{};
    gm.run_now(ecs);

    ecs.maintain();
}

impl State for Game {
    /// Load the assets and initialise the game
    fn new() -> Result<Self> {
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
        register_components(&mut ecs);

        generate_entities(&mut ecs);
        let map_size = Vector::new(60, 50);
        map::generate_map_new(&mut ecs, map_size);

        {
            let mut new_map = ecs.fetch_mut::<map::Map>();
            for _loop in 1..7 { map::apply_ca(&ecs, &mut new_map); }
        }

        let screen_layout = ScreenLayout::new(Vector::new(24, 24), Vector::new(50, 28), Vector::new(42, 24), Vector::new(4, 2));

        Ok(Self {
            tileset,
            screen_layout,
            ecs,
            runstate : RunState::Running
        })
    }

    /// Process keyboard and mouse, update the game state
    fn update(&mut self, window: &mut Window) -> Result<()> {


        game_input(self, window);

        if self.runstate == RunState::Running {
            run_systems(&mut self.ecs);
            player_input(self, window);
        }

        Ok(())
    }


    /// Draw stuff on the screen
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        render_text(window, "Test title", self.screen_layout.top_panel_origin.times(self.screen_layout.tile_size_pixels), 40.0)?;
        render_text(window, "From function!", self.screen_layout.bottom_panel_origin.times(self.screen_layout.tile_size_pixels), 20.0)?;

        if self.runstate == RunState::Paused {
            render_text(window, "Paused", self.screen_layout.right_panel_origin.times(self.screen_layout.tile_size_pixels), 20.0)?;
        }

        let positions = self.ecs.read_storage::<components::Position>();
        let renderables = self.ecs.read_storage::<components::Renderable>();
        let tiles = self.ecs.read_storage::<components::Tile>();
        let viewsheds = self.ecs.write_storage::<components::Viewshed>();

        let map = self.ecs.fetch::<map::Map>();
        let player_pos = self.ecs.fetch::<PlayerPosition>();
        //println!("{}", player_pos);

        let tileset = &mut self.tileset;
        let offset_px = self.screen_layout.screen_origin.times(self.screen_layout.tile_size_pixels);
        let tile_pixels = self.screen_layout.tile_size_pixels;
        let screen_layout = &self.screen_layout;

        let mut visible_tiles: &Vec<rltk::Point> = &Vec::new(); 

        for (pos, viewshed) in (&positions, &viewsheds).join() {
            let pt = rltk::Point::new(pos.x, pos.y);
            visible_tiles = &viewshed.visible_tiles;
        }

        // render everything but tiles
        tileset.execute(|tileset| {
            for (pos, render) in (&positions, &renderables).join() {
                let position = Vector::new(pos.x, pos.y);
                let pt = rltk::Point::new(pos.x, pos.y);
                //let in_viewshed = visible_tiles.contains(&pt);
                let visible = map.revealed_map[&(pos.x as i32, pos.y as i32)];

                let mapped_position = camera_translate(Vector::new(player_pos.x, player_pos.y), position, Vector::new(map.width, map.height));
                let px_pos = offset_px + mapped_position.times(tile_pixels);

                if !should_render(mapped_position, screen_layout) || !visible {
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

        /*
        let full_health_width_px = 100.0;
        let current_health_width_px = (50 as f32 / 100 as f32) * full_health_width_px;

        let health_bar_pos_px = Vector::new(screen_width_tiles * TILE_EDGE_PIXELS, 0.0);
        let mana_bar_pos_px = Vector::new(screen_width_tiles * TILE_EDGE_PIXELS, -TILE_EDGE_PIXELS);

        render_bar(window, Color::RED, current_health_width_px, health_bar_pos_px, full_health_width_px, tile_size_px)?;
        render_bar(window, Color::BLUE, current_health_width_px, mana_bar_pos_px, full_health_width_px, tile_size_px)?;
        */

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

    

    run::<Game>("Qwarves", 
                Vector::new(WINDOW_WIDTH_TILES * TILE_EDGE_PIXELS, WINDOW_HEIGHT_TILES * TILE_EDGE_PIXELS), 
                settings);
}
