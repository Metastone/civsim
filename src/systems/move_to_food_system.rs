use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all herbivorous in the direction of their food target (if they have one)
        let mut creature_to_food: HashMap<EntityId, EntityId> = HashMap::new();
        for info in iter_entities_with!(
            ecs,
            HerbivorousComponent,
            PositionComponent,
            MoveToFoodComponent
        ) {
            // Get the target food info
            let c_entity;
            if let Some(MoveToFoodComponent { food_entity }) =
                ecs.get_component::<MoveToFoodComponent>(&info)
            {
                c_entity = *food_entity;
            } else {
                // Go to inactive state if the food can't be found
                ecs.remove_component(info.entity, to_ctype!(MoveToFoodComponent));
                ecs.add_component(info.entity, &InactiveComponent::new());
                continue;
            }

            // Get the food position
            let mut position_exists = false;
            let mut food_position = PositionComponent::new();
            if let Some(pos) = ecs.get_component_from_entity::<PositionComponent>(c_entity) {
                food_position = *pos;
                position_exists = true;
            }

            // Go to inactive state if the target position can't be found for some reason
            if !position_exists {
                ecs.remove_component(info.entity, to_ctype!(MoveToFoodComponent));
                ecs.add_component(info.entity, &InactiveComponent::new());
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

        // If food reached, go to eating state
        for (entity, food_entity) in creature_to_food {
            ecs.remove_component(entity, to_ctype!(MoveToFoodComponent));
            ecs.add_component(entity, &EatingFoodComponent::new(food_entity));
        }
    }
}
