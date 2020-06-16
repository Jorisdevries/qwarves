use specs::prelude::*;
use quicksilver::prelude::*;
use rand::Rng;

use crate::components;
use crate::map;

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
    type SystemData = ( ReadExpect<'a, map::Map>,
                        WriteStorage<'a, components::Viewshed>, 
                        WriteStorage<'a, components::Position>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut viewshed, pos) = data;

        for (viewshed,pos) in (&mut viewshed, &pos).join() {
            println!("{}", Vector::new(pos.x, pos.y));
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = rltk::field_of_view(rltk::Point::new(pos.x, pos.y), viewshed.range, &*map);
            viewshed.visible_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1 );
        }
    }
}