use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::ecs::{Ecs, System, Update, Component};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;
use crate::constants::CARNIVOROUS_SPEED;

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
            let mut target_body: Option<BodyComponent> = None;
            let mut closest_distance_squared = f64::MAX;
            let mut found_target = false;
            let mut is_corpse = false;

            // Check corpses
            if let Some((distance_squared, closest_entity, closest_body)) =
                utils::find_closest::<CorpseComponent>(ecs, body)
            {
                closest_distance_squared = distance_squared;
                found_target = true;
                is_corpse = true;
                target_entity = closest_entity;
                target_body = Some(closest_body);
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity, closest_body)) =
                utils::find_closest::<HerbivorousComponent>(ecs, body)
                && distance_squared < closest_distance_squared
            {
                found_target = true;
                target_entity = closest_entity;
                target_body = Some(closest_body);
            }

            if found_target {
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
                        CARNIVOROUS_SPEED,
                        on_target_reached,
                        on_failure,
                    )),
                });
                // TODO parametre generique pour le type de composant à supprimer
                updates.push(Update::Delete {
                    info: *info,
                    c_type: to_ctype!(InactiveComponent),
                });
            }
        }

        ecs.apply(updates);
    }
}
