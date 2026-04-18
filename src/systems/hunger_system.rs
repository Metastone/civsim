use crate::components::all::*;
use crate::configuration::Config;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        for (creature, _) in iter_components!(ecs, (), (CreatureComponent)) {
            creature.energy -= config.creature.hunger_rate;
            if creature.energy <= 0.0 {
                creature.energy = 0.0;
            }
        }
    }
}
