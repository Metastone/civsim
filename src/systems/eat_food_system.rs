use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, ComponentType, EntityId, System};
use std::collections::HashMap;

pub struct EatFoodSystem;
impl System for EatFoodSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Make sure that a food is not eaten by more than one creature
        let mut food_to_creature: HashMap<EntityId, (EntityId, usize, usize)> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::EatingFood) {
            if let Some(eating_food) = manager.get_component::<EatingFoodComponent>(
                arch_index,
                entity_index,
                &ComponentType::EatingFood,
            ) {
                food_to_creature
                    .insert(eating_food.food_entity, (entity, arch_index, entity_index));
            }
            creatures_trying_to_eat.push(entity);
        }

        // Increase energy of creatures that ate a food
        for (_, arch_index, entity_index) in food_to_creature.values() {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                *arch_index,
                *entity_index,
                &ComponentType::Creature,
            ) {
                creature.energy += FOOD_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten food entities
        for food_entity in food_to_creature.keys() {
            manager.remove_entity(*food_entity);
        }

        // Remove all "eating food" components and go into inactive state
        for entity in creatures_trying_to_eat.iter() {
            manager.remove_component(*entity, &ComponentType::EatingFood);
            manager.add_component(*entity, &InactiveComponent::new());
        }
    }
}
