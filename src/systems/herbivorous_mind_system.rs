use crate::algorithms::path_finding::WayPoint;
use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, System, Update, RESERVED_ENTITY_ID};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct HerbivorousMindSystem;
impl System for HerbivorousMindSystem {
    fn run(&self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the bodies of all inactive herbivorous entities
        let mut herbivorous_bodies = HashMap::new();
        for (inactive, body, info) in iter_components!(
            ecs,
            (HerbivorousComponent, BodyComponent),
            (InactiveComponent, BodyComponent)
        ) {
            if !inactive.idle {
                herbivorous_bodies.insert(info, *body);
            } else {
                inactive.idle_ticks_count += 1;
                if inactive.idle_ticks_count >= config.creature.total_ticks_idle {
                    inactive.idle = false;
                    inactive.idle_ticks_count = 0;
                }
            }
        }

        // For each body, find the closest plant
        for (info, body) in &herbivorous_bodies {
            let mut target_entity = RESERVED_ENTITY_ID;
            let mut target_body: Option<BodyComponent> = None;
            let mut path_to_target: Option<Vec<WayPoint>> = None;
            let mut target_found = false;

            if let Some((_, closest_entity, closest_body, closest_path)) =
                utils::find_closest_reachable::<PlantComponent>(ecs, config, info.entity, body)
            {
                target_found = true;
                target_entity = closest_entity;
                target_body = Some(closest_body);
                path_to_target = Some(closest_path);
            }

            // If a reachable target is found, move to target
            if target_found {
                let on_target_reached = Box::new(EatingPlantComponent::new(target_entity));
                let on_failure = Box::new(InactiveComponent::new());
                updates.push(Update::Add {
                    info: *info,
                    comp: Box::new(MoveToTargetComponent::new(
                        target_entity,
                        target_body.unwrap(),
                        path_to_target.unwrap(),
                        config.creature.herbivorous_speed,
                        on_target_reached,
                        on_failure,
                    )),
                });
                updates.push(Update::Delete {
                    info: *info,
                    c_type: to_ctype!(InactiveComponent),
                });
            }
            // If no reachable target is found, go into idle state for a while to avoid doing a
            // heavy path computation at each iteration
            else {
                ecs.component_mut::<InactiveComponent>(info).unwrap().idle = true;
            }
        }

        ecs.apply(updates);
    }
}
