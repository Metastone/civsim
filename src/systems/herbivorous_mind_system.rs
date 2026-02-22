use crate::algorithms::path_finding::WayPoint;
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
        let mut closest_target_of: HashMap<EntityInfo, (EntityId, BodyComponent, Vec<WayPoint>)> =
            HashMap::new();
        for (info, body) in &herbivorous_bodies {
            found.insert(*info, false);

            if let Some((_, closest_entity, closest_body, closest_path)) =
                utils::find_closest_reachable::<FoodComponent>(ecs, info.entity, body)
            {
                found.insert(*info, true);
                closest_target_of.insert(*info, (closest_entity, closest_body, closest_path));
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_info, found) in found {
            if found {
                let (found_entity, found_body, found_path) =
                    closest_target_of.get(&herbivorous_info).unwrap();
                let on_target_reached = Box::new(EatingFoodComponent::new(*found_entity));
                let on_failure = Box::new(InactiveComponent::new());
                updates.push(Update::Add {
                    info: herbivorous_info,
                    comp: Box::new(MoveToTargetComponent::new(
                        *found_entity,
                        *found_body,
                        found_path.clone(),
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
