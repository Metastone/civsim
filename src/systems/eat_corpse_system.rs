use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use std::collections::HashMap;

pub struct EatCorpseSystem;
impl System for EatCorpseSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Make sure that a corpse is not eaten by more than one creature
        let mut corpse_to_creature: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::EatingCorpse)
        {
            if let Some(eating_corpse) = manager.get_component::<EatingCorpseComponent>(
                arch_index,
                entity_index,
                &ComponentType::EatingCorpse,
            ) {
                corpse_to_creature.insert(eating_corpse.corpse_entity, entity);
            }
            creatures_trying_to_eat.push(entity);
        }

        // Increase energy of creatures that ate a corpse
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::EatingCorpse, ComponentType::Creature])
        {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if corpse_to_creature
                    .values()
                    .any(|&creature_entity| creature_entity == entity)
                {
                    creature.energy += CORPSE_ENERGY;
                    if creature.energy > MAX_ENERGY {
                        creature.energy = MAX_ENERGY;
                    }
                }
            }
        }

        // Remove eaten corpse entities
        for corpse_entity in corpse_to_creature.keys() {
            manager.remove_entity(*corpse_entity);
        }

        // Remove all "eating corpse" components
        for entity in creatures_trying_to_eat.iter() {
            manager.remove_component(*entity, &ComponentType::EatingCorpse);
        }
    }
}
