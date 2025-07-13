use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use std::any::TypeId;
use std::collections::HashMap;

pub struct EatCorpseSystem;
impl System for EatCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Make sure that a corpse is not eaten by more than one creature
        let mut corpse_to_creature: HashMap<EntityId, (usize, usize, EntityId)> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in iter_entities!(ecs, EatingCorpseComponent) {
            if let Some(eating_corpse) =
                ecs.get_component::<EatingCorpseComponent>(arch_index, entity_index)
            {
                corpse_to_creature.insert(
                    eating_corpse.corpse_entity,
                    (entity, arch_index, entity_index),
                );
            }
            creatures_trying_to_eat.push(entity);
        }

        // Increase energy of creatures that ate a corpse
        for (_, arch_index, entity_index) in corpse_to_creature.values() {
            if let Some(creature) =
                ecs.get_component_mut::<CreatureComponent>(*arch_index, *entity_index)
            {
                creature.energy += CORPSE_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten corpse entities
        for corpse_entity in corpse_to_creature.keys() {
            ecs.remove_entity(*corpse_entity);
        }

        // Go into inactive state
        for entity in creatures_trying_to_eat.iter() {
            ecs.remove_component(*entity, to_ctype!(EatingCorpseComponent));
            ecs.add_component(*entity, &InactiveComponent::new());
        }
    }
}
