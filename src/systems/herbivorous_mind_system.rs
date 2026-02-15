use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::constants::HERBIVOROUS_SPEED;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the bodies of all inactive herbivorous entities
        let mut herbivorous_bodies = HashMap::new();
        for (body, info) in iter_components!(
            ecs,
            (HerbivorousComponent, BodyComponent, InactiveComponent),
            (BodyComponent)
        ) {
            herbivorous_bodies.insert(info, *body);
        }

        // For each body, find the closest food
        let mut found: HashMap<EntityInfo, bool> = HashMap::new();
        let mut closest_target_of: HashMap<EntityInfo, (EntityId, BodyComponent)> = HashMap::new();
        for (info, body) in &herbivorous_bodies {
            found.insert(*info, false);

            // Check all foods
            if let Some((_, closest_entity)) =
                utils::find_closest(ecs, body, to_ctype!(FoodComponent))
            {
                found.insert(*info, true);
                closest_target_of.insert(
                    *info,
                    (
                        closest_entity,
                        *ecs.get_component::<BodyComponent>(info).unwrap(),
                    ),
                );
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_info, found) in found {
            if found {
                let (found_entity, found_body) = closest_target_of.get(&herbivorous_info).unwrap();
                let on_target_reached = Box::new(EatingFoodComponent::new(*found_entity));
                let on_failure = Box::new(InactiveComponent::new());
                updates.push(Update::Add {
                    info: herbivorous_info,
                    comp: Box::new(MoveToTargetComponent::new(
                        *found_entity,
                        *found_body,
                        HERBIVOROUS_SPEED,
                        on_target_reached,
                        on_failure,
                    )),
                });
                updates.push(Update::Delete {
                    info: herbivorous_info,
                    c_type: to_ctype!(InactiveComponent),
                });
            }
        }

        ecs.apply(updates);
    }
}
