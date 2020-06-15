use specs::prelude::*;
use rand::Rng;

use crate::components;

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
            println!("Monster considers their own existence");
        }
    }
}