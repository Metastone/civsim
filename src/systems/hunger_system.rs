use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, System};
use std::any::TypeId;

pub struct HungerSystem;
impl System for HungerSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in iter_entities!(manager, CreatureComponent) {
            if let Some(creature) =
                manager.get_component_mut::<CreatureComponent>(arch_index, entity_index)
            {
                creature.energy -= HUNGER_RATE;
                if creature.energy <= 0.0 {
                    creature.energy = 0.0;
                }
            }
        }
    }
}
