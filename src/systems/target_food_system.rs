use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct TargetFoodSystem;
impl System for TargetFoodSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        for info in iter_entities!(
            ecs,
            HerbivorousComponent,
            BodyComponent,
            TargetFoodComponent
        ) {
            // Get the target food info
            let TargetFoodComponent { food_entity } =
                ecs.get_component::<TargetFoodComponent>(&info).unwrap();
            let f_entity = *food_entity;

            // Get the food body
            let food_body;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(f_entity) {
                food_body = *pos;
            } else {
                // Go to inactive state if the target body can't be found for some reason
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetFoodComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            let body = ecs.get_component::<BodyComponent>(&info).unwrap();

            if utils::is_target_reached(body, &food_body) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetFoodComponent),
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
