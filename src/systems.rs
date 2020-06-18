use specs::prelude::*;
use quicksilver::prelude::*;
use rand::Rng;

use crate::components;
use crate::map;
use rltk::{Algorithm2D};

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

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    type SystemData = ( WriteExpect<'a, map::Map>,
                        ReadExpect<'a, components::PlayerPosition>,
                        WriteStorage<'a, components::Viewshed>,
                        ReadStorage<'a, components::Monster>,
                        ReadStorage<'a, components::Name>,
                        WriteStorage<'a, components::Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, mut viewshed, monster, name, mut position) = data;

        for (viewshed, _monster, name, mut pos) in (&mut viewshed, &monster, &name, &mut position).join() {
            for i in &viewshed.visible_tiles {
                if i.x == player_pos.x && i.x == player_pos.x {
                    println!("{} shouts insults", name.name);
                    break;
                }
            }

            let path = rltk::a_star_search(
                map.point2d_to_index(rltk::Point::new(pos.x, pos.y)) as i32,
                map.point2d_to_index(rltk::Point::new(player_pos.x, player_pos.y)) as i32,
                &mut *map
            );
            if path.success && path.steps.len()>1 {
                pos.x = path.steps[1] as i32 % map.width;
                pos.y = path.steps[1] as i32 / map.width;
                viewshed.dirty = true;
            }
        }
    }
}

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = ( WriteExpect<'a, map::Map>,
                        Entities<'a>,
                        WriteStorage<'a, components::Viewshed>, 
                        WriteStorage<'a, components::Position>,
                        ReadStorage<'a, components::Player>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player) = data;

        for (ent,viewshed,pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = false;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles = rltk::field_of_view(rltk::Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                let p : Option<&components::Player> = player.get(ent);

                if let Some(_p) = p {
                    for t in map.visible_map.iter_mut() { *t = false };
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.point2d_to_index(rltk::Point::new(vis.x, vis.y));
                        map.revealed_map[idx] = true;
                        map.visible_map[idx] = true;
                    }
                }

            }
            
        }
    }
}

pub struct GlyphMapper {}

impl<'a> System<'a> for GlyphMapper {
    type SystemData = ( WriteExpect<'a, map::Map>,
                        WriteStorage<'a, components::Renderable>, 
                        WriteStorage<'a, components::Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, renderable, pos) = data;

        for (renderable, pos) in (&renderable, &pos).join() {
            let coords = rltk::Point::new(pos.x, pos.y);
            let index = map.point2d_to_index(coords);

            let glyph = renderable.glyph;
            *map.glyph_map.get_mut(&index).unwrap() = glyph;
        }
    }
}