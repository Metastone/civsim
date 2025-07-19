use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, EntityInfo, System};
use std::any::TypeId;
use std::collections::HashMap;

pub struct EatFoodSystem;
impl System for EatFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Make sure that a food is not eaten by more than one creature
        let mut food_to_creature: HashMap<EntityId, EntityInfo> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for info in iter_entities!(ecs, EatingFoodComponent) {
            if let Some(eating_food) = ecs.get_component::<EatingFoodComponent>(&info) {
                food_to_creature.insert(eating_food.food_entity, info);
            }
            creatures_trying_to_eat.push(info.entity);
        }

        // Increase energy of creatures that ate a food
        for info in food_to_creature.values() {
            if let Some(creature) = ecs.get_component_mut::<CreatureComponent>(info) {
                creature.energy += FOOD_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten food entities
        for food_entity in food_to_creature.keys() {
            ecs.remove_entity(*food_entity);
        }

        // Remove all "eating food" components and go into inactive state
        for entity in creatures_trying_to_eat.iter() {
            ecs.remove_component(*entity, to_ctype!(EatingFoodComponent));
            ecs.add_component(*entity, &InactiveComponent::new());
        }
    }
}
