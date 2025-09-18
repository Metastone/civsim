use crate::components::*;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the positions of all inactive herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (position, info) in iter_components_with!(
            ecs,
            (HerbivorousComponent, PositionComponent, InactiveComponent),
            PositionComponent
        ) {
            herbivorous_positions.insert(info, *position);
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityInfo, bool> = HashMap::new();
        let mut closest_entity_of: HashMap<EntityInfo, EntityId> = HashMap::new();
        for (info, position) in &herbivorous_positions {
            found.insert(*info, false);

            // Check all foods
            if let Some((_, closest_entity)) =
                utils::find_closest(ecs, position, to_ctype!(FoodComponent))
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
