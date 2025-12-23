use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct TargetHerbivorousSystem;
impl System for TargetHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Move all carnivorous in the direction of their herbivorous target (if they have one)
        for info in iter_entities!(
            ecs,
            CarnivorousComponent,
            BodyComponent,
            TargetHerbivorousComponent
        ) {
            // Get the target herbivorous info
            let TargetHerbivorousComponent { herbivorous_entity } = ecs
                .get_component::<TargetHerbivorousComponent>(&info)
                .unwrap();
            let herb_entity = *herbivorous_entity;

            // Get the herbivorous body
            let herbivorous_body;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(herb_entity) {
                herbivorous_body = *pos;
            } else {
                // Go to inactive state if the target body can't be found
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetHerbivorousComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            let body = ecs.get_component::<BodyComponent>(&info).unwrap();

            // Move the carnivorous towards the herbivorous target
            if utils::is_target_reached(body, &herbivorous_body) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetHerbivorousComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(AttackingHerbivorousComponent::new(herb_entity)),
                });
            }
        }

        ecs.apply(updates);
    }
}
