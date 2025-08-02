use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all herbivorous in the direction of their food target (if they have one)
        let mut creature_to_food: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creature_to_inactive: HashSet<EntityId> = HashSet::new();
        for info in iter_entities_with!(
            ecs,
            HerbivorousComponent,
            PositionComponent,
            MoveToFoodComponent
        ) {
            // Get the target food info
            let MoveToFoodComponent { food_entity } =
                ecs.get_component::<MoveToFoodComponent>(&info).unwrap();
            let c_entity = *food_entity;

            // Get the food position
            let food_position;
            if let Some(pos) = ecs.get_component_from_entity::<PositionComponent>(c_entity) {
                food_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found for some reason
                creature_to_inactive.insert(info.entity);
                continue;
            }

            // Move the herbivorous towards the food target
            if utils::move_towards_position(
                ecs,
                &info,
                &food_position,
                FOOD_PIXEL_SIZE as f64,
                CREATURE_PIXEL_SIZE as f64,
                HERBIVOROUS_SPEED,
            ) {
                creature_to_food.insert(info.entity, c_entity);
            }
        }

        for entity in creature_to_inactive {
            ecs.remove_component(entity, to_ctype!(MoveToFoodComponent));
            ecs.add_component(entity, &InactiveComponent::new());
        }

        // If food reached, go to eating state
        for (entity, food_entity) in creature_to_food {
            ecs.remove_component(entity, to_ctype!(MoveToFoodComponent));
            ecs.add_component(entity, &EatingFoodComponent::new(food_entity));
        }
    }
}
