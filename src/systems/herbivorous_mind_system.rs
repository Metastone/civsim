use crate::components::*;
use crate::ecs::{ArchetypeManager, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all inactive herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in iter_entities_with!(
            manager,
            HerbivorousComponent,
            PositionComponent,
            InactiveComponent
        ) {
            if let Some(position) =
                manager.get_component::<PositionComponent>(arch_index, entity_index)
            {
                herbivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_entity_of: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &herbivorous_positions {
            found.insert(*entity, false);

            // Check all foods
            if let Some((_, closest_entity)) =
                utils::find_closest(manager, position, to_ctype!(FoodComponent))
            {
                found.insert(*entity, true);
                closest_entity_of.insert(*entity, closest_entity);
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_entity, found) in found {
            if found {
                let found_entity = closest_entity_of.get(&herbivorous_entity).unwrap();
                manager.add_component(herbivorous_entity, &MoveToFoodComponent::new(*found_entity));
                manager.remove_component(herbivorous_entity, to_ctype!(InactiveComponent));
            }
        }
    }
}
