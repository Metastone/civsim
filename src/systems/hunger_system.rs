use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (arch_index, entity_index, _) in iter_entities!(ecs, CreatureComponent) {
            if let Some(creature) =
                ecs.get_component_mut::<CreatureComponent>(arch_index, entity_index)
            {
                creature.energy -= HUNGER_RATE;
                if creature.energy <= 0.0 {
                    creature.energy = 0.0;
                }
            }
        }
    }
}
