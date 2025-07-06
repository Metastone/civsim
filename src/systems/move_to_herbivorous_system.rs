use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use crate::systems::utils;
use std::collections::HashMap;

pub struct MoveToHerbivorousSystem;
impl System for MoveToHerbivorousSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Move all carnivorous in the direction of their herbivorous target (if they have one)
        let mut creature_to_herbivorous: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in manager.iter_entities_with(&[
            ComponentType::Carnivorous,
            ComponentType::Position,
            ComponentType::MoveToHerbivorous,
        ]) {
            // Get the target herbivorous info
            let herb_entity;
            if let Some(MoveToHerbivorousComponent { herbivorous_entity }) =
                manager.get_component::<MoveToHerbivorousComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::MoveToHerbivorous,
                )
            {
                herb_entity = *herbivorous_entity;
            } else {
                // Go to inactive state if the herbivorous can't be found
                manager.remove_component(entity, &ComponentType::MoveToHerbivorous);
                manager.add_component(entity, &InactiveComponent::new());
                continue;
            }

            // Get the herbivorous position
            let mut position_exists = false;
            let mut herbivorous_position = PositionComponent::new();
            if let Some((h_arch_index, h_entity_index)) = manager.get_entity_indexes(herb_entity) {
                if let Some(pos) = manager.get_component::<PositionComponent>(
                    h_arch_index,
                    h_entity_index,
                    &ComponentType::Position,
                ) {
                    herbivorous_position = *pos;
                    position_exists = true;
                }
            }

            // Go to inactive state if the target position can't be found
            if !position_exists {
                manager.remove_component(entity, &ComponentType::MoveToHerbivorous);
                manager.add_component(entity, &InactiveComponent::new());
                continue;
            }

            // Move the carnivorous towards the herbivorous target
            if utils::move_towards_position(
                manager,
                arch_index,
                entity_index,
                &herbivorous_position,
                CREATURE_PIXEL_SIZE as f64,
                CREATURE_PIXEL_SIZE as f64,
                CARNIVOROUS_SPEED,
            ) {
                creature_to_herbivorous.insert(entity, herb_entity);
            }
        }

        // If herbivorous reached, go to eating state
        for (entity, herbivorous_entity) in creature_to_herbivorous {
            manager.remove_component(entity, &ComponentType::MoveToHerbivorous);
            manager.add_component(
                entity,
                &AttackingHerbivorousComponent::new(herbivorous_entity),
            );
        }
    }
}
