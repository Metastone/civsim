use crate::components::*;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct CarnivorousMindSystem;
impl System for CarnivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the positions of all inactive carnivorous entities
        let mut carnivorous_positions = HashMap::new();
        for (position, info) in iter_components_with!(
            ecs,
            (CarnivorousComponent, PositionComponent, InactiveComponent),
            PositionComponent
        ) {
            carnivorous_positions.insert(info, *position);
        }

        // For each position, find the closest corpse or herbivorous
        for (info, position) in &carnivorous_positions {
            let mut target_entity = 0;
            let mut closest_distance_squared = f64::MAX;
            let mut found_target = false;
            let mut is_corpse = false;

            // Check corpses
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, position, to_ctype!(CorpseComponent))
            {
                closest_distance_squared = distance_squared;
                found_target = true;
                is_corpse = true;
                target_entity = closest_entity;
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, position, to_ctype!(HerbivorousComponent))
            {
                if distance_squared < closest_distance_squared {
                    found_target = true;
                    target_entity = closest_entity;
                }
            }

            if found_target {
                if is_corpse {
                    updates.push(Update::Add {
                        info: *info,
                        comp: Box::new(MoveToCorpseComponent::new(target_entity)),
                    });
                } else {
                    updates.push(Update::Add {
                        info: *info,
                        comp: Box::new(MoveToHerbivorousComponent::new(target_entity)),
                    });
                }
                updates.push(Update::Delete {
                    info: *info,
                    c_type: to_ctype!(InactiveComponent),
                });
            }
        }

        ecs.apply(updates);
    }
}
