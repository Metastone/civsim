use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, System};
use std::any::TypeId;

pub struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in iter_entities!(manager, CreatureComponent) {
            if let Some(creature) =
                manager.get_component_mut::<CreatureComponent>(arch_index, entity_index)
            {
                if creature.energy <= 0.0 {
                    creature.health -= EXHAUSTION_RATE;
                }
                if creature.health <= 0.0 {
                    creature.health = 0.0;
                }
            }
        }
    }
}
