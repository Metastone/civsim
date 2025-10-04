use crate::components::all::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (creature, _) in iter_components!(ecs, CreatureComponent) {
            creature.energy -= HUNGER_RATE;
            if creature.energy <= 0.0 {
                creature.energy = 0.0;
            }
        }
    }
}
