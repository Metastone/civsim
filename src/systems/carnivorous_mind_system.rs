use crate::components::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use crate::systems::utils;
use std::collections::HashMap;

pub struct CarnivorousMindSystem;
impl System for CarnivorousMindSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all inactive carnivorous entities
        let mut carnivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in manager.iter_entities_with(&[
            ComponentType::Carnivorous,
            ComponentType::Position,
            ComponentType::Inactive,
        ]) {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                carnivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest corpse or herbivorous
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut is_corpse: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_entity_of: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &carnivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);

            // Check corpses
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(manager, position, ComponentType::Corpse)
            {
                closest_distance_squared = distance_squared;
                found.insert(*entity, true);
                is_corpse.insert(*entity, true);
                closest_entity_of.insert(*entity, closest_entity);
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(manager, position, ComponentType::Herbivorous)
            {
                if distance_squared < closest_distance_squared {
                    found.insert(*entity, true);
                    is_corpse.insert(*entity, false);
                    closest_entity_of.insert(*entity, closest_entity);
                }
            }
        }

        // Assign action component to carnivorous that found a target
        for (carnivorous_entity, found) in found {
            if found {
                let found_entity = closest_entity_of.get(&carnivorous_entity).unwrap();
                if *is_corpse.get(&carnivorous_entity).unwrap() {
                    manager.add_component(
                        carnivorous_entity,
                        &MoveToCorpseComponent::new(*found_entity),
                    );
                } else {
                    manager.add_component(
                        carnivorous_entity,
                        &MoveToHerbivorousComponent::new(*found_entity),
                    );
                }
                manager.remove_component(carnivorous_entity, &ComponentType::Inactive);
            }
        }
    }
}
