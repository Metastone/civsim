use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (creature, _) in iter_components!(ecs, CreatureComponent) {
            if creature.energy <= 0.0 {
                creature.health -= EXHAUSTION_RATE;
            }
            if creature.health <= 0.0 {
                creature.health = 0.0;
            }
        }
    }
}
