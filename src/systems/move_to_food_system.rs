use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use std::collections::HashMap;

pub struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Herbivorous, ComponentType::Position])
        {
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
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &herbivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);
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
                        closest_position.insert(*entity, *food_position);
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Move all herbivorous entities in direction of the closest food
        let mut creature_to_food: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Herbivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component_mut::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                if *found.get(&entity).unwrap() {
                    let food_position = closest_position.get(&entity).unwrap();
                    let food_entity = closest_entity.get(&entity).unwrap();
                    let vec_to_food = (food_position.x - position.x, food_position.y - position.y);
                    let norm = (vec_to_food.0.powi(2) + vec_to_food.1.powi(2)).sqrt();
                    if norm < (CREATURE_PIXEL_SIZE as f64 / 2.0 + FOOD_PIXEL_SIZE as f64 / 2.0) {
                        // Food reached -> will go to eating state
                        creature_to_food.insert(entity, *food_entity);
                    } else {
                        // Get closer to the food
                        position.x += vec_to_food.0 / norm * CREATURE_SPEED;
                        position.y += vec_to_food.1 / norm * CREATURE_SPEED;
                    }
                }
            }
        }

        // If food reached, go to eating state
        for (entity, food_entity) in creature_to_food {
            manager.add_component(entity, &EatingFoodComponent::new(food_entity));
        }
    }
}
