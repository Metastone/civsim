use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, System};

pub struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in manager.iter_entities(ComponentType::Creature) {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
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
