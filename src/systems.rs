use specs::prelude::*;
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
    type SystemData = ( ReadStorage<'a, components::Position>,
                        ReadStorage<'a, components::Monster>);

    fn run(&mut self, data : Self::SystemData) {
        let (pos, monster) = data;

        for (_pos, _monster) in (&pos, &monster).join() {
            //println!("Monster considers their own existence");
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
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = rltk::field_of_view(rltk::Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );

            // If this is the player, reveal what they can see
            let p : Option<&components::Player> = player.get(ent);
            if let Some(_p) = p {
                for vis in viewshed.visible_tiles.iter() {
                    *map.revealed_map.get_mut(&(vis.x, vis.y)).unwrap() = true;
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