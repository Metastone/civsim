use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use std::collections::HashMap;

pub struct MoveToCorpseSystem;
impl System for MoveToCorpseSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all carnivorous entities
        let mut carnivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Carnivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                carnivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest corpse
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &carnivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);
            for (arch_index, entity_index, food_entity) in
                manager.iter_entities_with(&[ComponentType::Corpse, ComponentType::Position])
            {
                if let Some(corpse_position) = manager.get_component::<PositionComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::Position,
                ) {
                    let distance_squared = (corpse_position.x - position.x).powi(2)
                        + (corpse_position.y - position.y).powi(2);
                    if distance_squared < closest_distance_squared {
                        closest_distance_squared = distance_squared;
                        found.insert(*entity, true);
                        closest_position.insert(*entity, *corpse_position);
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Move all carnivorous entities in direction of the closest corpse
        let mut creature_to_corpse: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Carnivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component_mut::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                if *found.get(&entity).unwrap() {
                    let corpse_position = closest_position.get(&entity).unwrap();
                    let corpse_entity = closest_entity.get(&entity).unwrap();
                    let vec_to_corpse = (
                        corpse_position.x - position.x,
                        corpse_position.y - position.y,
                    );
                    let norm = (vec_to_corpse.0.powi(2) + vec_to_corpse.1.powi(2)).sqrt();
                    if norm < (CREATURE_PIXEL_SIZE as f64 / 2.0 + CREATURE_PIXEL_SIZE as f64 / 2.0)
                    {
                        // Corpse reached -> will go to eating state
                        creature_to_corpse.insert(entity, *corpse_entity);
                    } else {
                        // Get closer to the corpse
                        position.x += vec_to_corpse.0 / norm * CREATURE_SPEED;
                        position.y += vec_to_corpse.1 / norm * CREATURE_SPEED;
                    }
                }
            }
        }

        // If corpse reached, go to eating state
        for (entity, corpse_entity) in creature_to_corpse {
            manager.add_component(entity, &EatingCorpseComponent::new(corpse_entity));
        }
    }
}
