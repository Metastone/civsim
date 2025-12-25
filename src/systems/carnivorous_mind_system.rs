use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct CarnivorousMindSystem;
impl System for CarnivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the bodies of all inactive carnivorous entities
        let mut carnivorous_body = HashMap::new();
        for (body, info) in iter_components!(
            ecs,
            (CarnivorousComponent, InactiveComponent),
            (BodyComponent)
        ) {
            carnivorous_body.insert(info, *body);
        }

        // For each body, find the closest corpse or herbivorous
        for (info, body) in &carnivorous_body {
            let mut target_entity = 0;
            let mut closest_distance_squared = f64::MAX;
            let mut found_target = false;
            let mut is_corpse = false;

            // Check corpses
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, body, to_ctype!(CorpseComponent))
            {
                closest_distance_squared = distance_squared;
                found_target = true;
                is_corpse = true;
                target_entity = closest_entity;
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, body, to_ctype!(HerbivorousComponent))
                && distance_squared < closest_distance_squared
            {
                found_target = true;
                target_entity = closest_entity;
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
