use crate::components::all::*;
use crate::components::body_component::BodyComponent;
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
            BodyComponent
        ) {
            herbivorous_bodies.insert(info, *body);
        }

        // For each body, find the closest food
        let mut found: HashMap<EntityInfo, bool> = HashMap::new();
        let mut closest_entity_of: HashMap<EntityInfo, EntityId> = HashMap::new();
        for (info, body) in &herbivorous_bodies {
            found.insert(*info, false);

            // Check all foods
            if let Some((_, closest_entity)) =
                utils::find_closest(ecs, body, to_ctype!(FoodComponent))
            {
                found.insert(*info, true);
                closest_entity_of.insert(*info, closest_entity);
            }
        }

        // Assign action component to herbivorous that found a target
        for (herbivorous_info, found) in found {
            if found {
                let found_entity = closest_entity_of.get(&herbivorous_info).unwrap();
                updates.push(Update::Add {
                    info: herbivorous_info,
                    comp: Box::new(MoveToFoodComponent::new(*found_entity)),
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
