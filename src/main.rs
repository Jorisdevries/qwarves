use quicksilver::prelude::*;

use std::collections::HashMap;
//use rand::seq::SliceRandom; 
use rand::Rng;
use std::cmp;

use specs::prelude::*;
use specs_derive::Component;

pub mod map;

static TILE_EDGE_PIXELS: i32 = 24;
static WINDOW_WIDTH_TILES: i32 = 50;
static WINDOW_HEIGHT_TILES: i32 = 28;

static LEFT_OFFSET_TILES: i32 = 4;
static RIGHT_OFFSET_TILES: i32 = 4;
static TOP_OFFSET_TILES: i32 = 2;
static BOTTOM_OFFSET_TILES: i32 = 2;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: char,
    color: Color,
}

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct RandomMover {}

impl<'a> System<'a> for RandomMover {
    type SystemData = (ReadStorage<'a, RandomMover>, 
                        WriteStorage<'a, Position>);

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
    //map: Vec<map::Tile>,
    levels: Vec<Vec<map::Tile>>,
    level_index: usize,
    entity_manager: EntityManager,
    tileset: Asset<HashMap<char, Image>>,
    tile_size_px: Vector,
    ecs: World
}

#[derive(Clone, Debug, PartialEq)]
struct Entity {
    pos: Vector,
    glyph: char,
    color: Color,
    hp: i32,
    max_hp: i32,
    move_type: MoveType,
    id: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum MoveType {
    None, //will never move
    Stationary,
    Manual,
    Random,
}

#[derive(Clone, Debug, PartialEq)]
struct EntityManager {
    current_id: usize,
    entity_list: HashMap<usize, Entity>,
    movables: Vec<usize>,
}

impl EntityManager {
    fn new() -> EntityManager {
        EntityManager {
            current_id: 1,
            entity_list: HashMap::new(),
            movables: Vec::new(),
        }
    }

    fn new_entity(&mut self, pos: Vector, glyph: char, color: Color, hp: i32, max_hp: i32, move_type: MoveType) {
        let new_entity = Entity {
            pos,
            glyph,
            color,
            hp,
            max_hp,
            move_type,
            id: self.current_id,
        };

        self.current_id += 1;

        if new_entity.move_type != MoveType::None {
            let moving_id: usize = new_entity.id;
            self.movables.push(moving_id);
        }

        self.entity_list.insert(new_entity.id, new_entity);
    }

    fn move_entities(&mut self) {
        for id in &self.movables {
            let entity: &mut Entity = self.entity_list.get_mut(&id).unwrap(); 

            match entity.move_type {
                MoveType::Random => {
                    let mut rng = rand::thread_rng();
    
                    if rng.gen_range(1, 61) == 60 {
                        entity.pos.x += rng.gen_range(-1, 2) as f32;
                        entity.pos.y += rng.gen_range(-1, 2) as f32;
                    }
                },
                _ => (),
            }
        }
    }
}


fn generate_entities(ecs: &mut World) {
    ecs
    .create_entity()
    .with(Position { x: 40, y: 25 })
    .with(Renderable {
        glyph: '@',
        color: Color::BLACK,
    })
    .with(Player{})
    .build();

    ecs
    .create_entity()
    .with(Position { x: 30, y: 10 })
    .with(Renderable {
        glyph: 'g',
        color: Color::GREEN,
    })
    .with(RandomMover{})
    .build();


    //entity_manager.new_entity(Vector::new(9, 6), 'g', Color::RED, 1, 10, MoveType::Random);
                                
    /*
    vec![
        Entity {
            pos: Vector::new(9, 6),
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
            move_type: MoveType::Random,
        },
        Entity {
            pos: Vector::new(2, 4),
            glyph: 'g',
            color: Color::RED,
            hp: 1,
            max_hp: 1,
            move_type: MoveType::Random,
        },
        Entity {
            pos: Vector::new(7, 5),
            glyph: '%',
            color: Color::PURPLE,
            hp: 0,
            max_hp: 0,
            move_type: MoveType::Random,
        },
        Entity {
            pos: Vector::new(4, 8),
            glyph: '%',
            color: Color::PURPLE,
            hp: 0,
            max_hp: 0,
            move_type: MoveType::Random,
        },
    ]
    */
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

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

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
    let mut rw = RandomMover{};
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

        let map_size = Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT);

        let level_index = 0;
        let levels = map::generate_levels(5, map_size);
        //let map = map::generate_map(map_size);
        //let map = &levels[level_index];

        let mut entity_manager = EntityManager::new();
        entity_manager.new_entity(Vector::new(9, 6), '@', Color::BLACK, 10, 10, MoveType::Manual); //player first

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
        ecs.register::<Position>();
        ecs.register::<Renderable>();
        ecs.register::<Player>();
        ecs.register::<RandomMover>();

        generate_entities(&mut ecs);

        Ok(Self {
            inventory,
            map_size,
            //map,
            levels,
            level_index,
            entity_manager,
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
        let map_size_px = self.map_size.times(tile_size_px);
        let offset_px = Vector::new((LEFT_OFFSET_TILES - 1) * TILE_EDGE_PIXELS, TOP_OFFSET_TILES * TILE_EDGE_PIXELS);

        // Draw the map
        let (tileset, map) = (&mut self.tileset, &mut self.levels[self.level_index]);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let players = self.ecs.read_storage::<Player>();

        let mut player_pos: Vector = Vector::new(0, 0);
        for (pos, _player) in (&positions, &players).join() {
            player_pos = Vector::new(pos.x, pos.y);
        }

        tileset.execute(|tileset| {
            for tile in map.iter() {
                let mapped_position = camera_translate(player_pos, tile.pos, Vector::new(map::MAP_WIDTH, map::MAP_HEIGHT));
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
