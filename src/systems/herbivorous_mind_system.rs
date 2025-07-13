use crate::components::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Get the positions of all inactive herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in iter_entities_with!(
            ecs,
            HerbivorousComponent,
            PositionComponent,
            InactiveComponent
        ) {
            if let Some(position) = ecs.get_component::<PositionComponent>(arch_index, entity_index)
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
                utils::find_closest(ecs, position, to_ctype!(FoodComponent))
            {
                found.insert(*entity, true);
                closest_entity_of.insert(*entity, closest_entity);
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_entity, found) in found {
            if found {
                let found_entity = closest_entity_of.get(&herbivorous_entity).unwrap();
                ecs.add_component(herbivorous_entity, &MoveToFoodComponent::new(*found_entity));
                ecs.remove_component(herbivorous_entity, to_ctype!(InactiveComponent));
            }
        }
    }
}
