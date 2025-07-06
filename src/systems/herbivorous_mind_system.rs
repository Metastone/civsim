use crate::components::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all inactive herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in manager.iter_entities_with(&[
            ComponentType::Herbivorous,
            ComponentType::Position,
            ComponentType::Inactive,
        ]) {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                herbivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &herbivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);

            // Check all foods
            for (arch_index, entity_index, food_entity) in
                manager.iter_entities_with(&[ComponentType::Food, ComponentType::Position])
            {
                if let Some(food_position) = manager.get_component::<PositionComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::Position,
                ) {
                    let distance_squared = (food_position.x - position.x).powi(2)
                        + (food_position.y - position.y).powi(2);
                    if distance_squared < closest_distance_squared {
                        closest_distance_squared = distance_squared;
                        found.insert(*entity, true);
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_entity, found) in found {
            if found {
                let found_entity = closest_entity.get(&herbivorous_entity).unwrap();
                manager.add_component(herbivorous_entity, &MoveToFoodComponent::new(*found_entity));
                manager.remove_component(herbivorous_entity, &ComponentType::Inactive);
            }
        }
    }
}
