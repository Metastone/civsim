use crate::components::all::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use std::any::TypeId;
use std::collections::HashMap;

pub struct EatFoodSystem;
impl System for EatFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Make sure that a food is not eaten by more than one creature
        let mut food_to_creature: HashMap<EntityId, EntityInfo> = HashMap::new();
        for info in iter_entities!(ecs, EatingFoodComponent) {
            if let Some(eating_food) = ecs.component::<EatingFoodComponent>(&info) {
                food_to_creature.insert(eating_food.food_entity, info);
            }
            updates.push(Update::Delete {
                info,
                c_type: to_ctype!(EatingFoodComponent),
            });
            updates.push(Update::Add {
                info,
                comp: Box::new(InactiveComponent::new()),
            });
        }

        // Increase energy of creatures that ate a food
        for info in food_to_creature.values() {
            if let Some(creature) = ecs.component_mut::<CreatureComponent>(info) {
                creature.energy += FOOD_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten food entities
        for food_entity in food_to_creature.keys() {
            if let Some(info) = ecs.entity_info(*food_entity) {
                updates.push(Update::DeleteEntity(info));
            }
        }

        ecs.apply(updates);
    }
}
