use crate::configuration::Config;
use crate::algorithms::path_finding::WayPoint;
use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::ecs::{Component, Ecs, System, Update, RESERVED_ENTITY_ID};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct CarnivorousMindSystem;
impl System for CarnivorousMindSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the bodies of all inactive carnivorous entities that are ready to act
        let mut carnivorous_body = HashMap::new();
        for (inactive, body, info) in iter_components!(
            ecs,
            (CarnivorousComponent),
            (InactiveComponent, BodyComponent)
        ) {
            if !inactive.idle {
                carnivorous_body.insert(info, *body);
            }
            else {
                inactive.idle_ticks_count += 1;
                if inactive.idle_ticks_count >= config.creature.total_ticks_idle {
                    inactive.idle = false;
                    inactive.idle_ticks_count = 0;

                }
            }
        }

        // For each body, find the closest corpse or herbivorous
        for (info, body) in &carnivorous_body {
            let mut target_entity = RESERVED_ENTITY_ID;
            let mut target_body: Option<BodyComponent> = None;
            let mut path_to_target: Option<Vec<WayPoint>> = None;
            let mut closest_distance_squared = f64::MAX;
            let mut target_found = false;
            let mut is_corpse = false;

            // Check corpses
            if let Some((distance_squared, closest_entity, closest_body, closest_path)) =
                utils::find_closest_reachable::<CorpseComponent>(ecs, config, info.entity, body)
            {
                closest_distance_squared = distance_squared;
                target_found = true;
                is_corpse = true;
                target_entity = closest_entity;
                target_body = Some(closest_body);
                path_to_target = Some(closest_path);
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity, closest_body, closest_path)) =
                utils::find_closest_reachable::<HerbivorousComponent>(ecs, config, info.entity, body)
                && distance_squared < closest_distance_squared
            {
                target_found = true;
                target_entity = closest_entity;
                target_body = Some(closest_body);
                path_to_target = Some(closest_path);
            }
            
            // If a reachable target is found, move to target
            if target_found {
                let on_target_reached: Box<dyn Component> = if is_corpse {
                    Box::new(EatingCorpseComponent::new(target_entity))
                } else {
                    Box::new(AttackingHerbivorousComponent::new(target_entity))
                };
                let on_failure = Box::new(InactiveComponent::new());
                updates.push(Update::Add {
                    info: *info,
                    comp: Box::new(MoveToTargetComponent::new(
                        target_entity,
                        target_body.unwrap(),
                        path_to_target.unwrap(),
                        config.creature.carnivorous_speed,
                        on_target_reached,
                        on_failure,
                    )),
                });
                Ecs::push_delete::<InactiveComponent>(*info, &mut updates);
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
