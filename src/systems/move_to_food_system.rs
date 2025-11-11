use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Move all herbivorous in the direction of their food target (if they have one)
        for info in iter_entities_with!(
            ecs,
            HerbivorousComponent,
            BodyComponent,
            MoveToFoodComponent
        ) {
            // Get the target food info
            let MoveToFoodComponent { food_entity } =
                ecs.get_component::<MoveToFoodComponent>(&info).unwrap();
            let f_entity = *food_entity;

            // Get the food position
            let food_position;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(f_entity) {
                food_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found for some reason
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToFoodComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            // Move the herbivorous towards the food target
            if utils::move_towards_position(ecs, &info, &food_position, HERBIVOROUS_SPEED) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToFoodComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(EatingFoodComponent::new(f_entity)),
                });
            }
        }

        ecs.apply(updates);
    }
}
