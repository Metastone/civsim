use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct MoveToHerbivorousSystem;
impl System for MoveToHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Move all carnivorous in the direction of their herbivorous target (if they have one)
        for info in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            BodyComponent,
            MoveToHerbivorousComponent
        ) {
            // Get the target herbivorous info
            let MoveToHerbivorousComponent { herbivorous_entity } = ecs
                .get_component::<MoveToHerbivorousComponent>(&info)
                .unwrap();
            let herb_entity = *herbivorous_entity;

            // Get the herbivorous position
            let herbivorous_position;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(herb_entity) {
                herbivorous_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToHerbivorousComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            // Move the carnivorous towards the herbivorous target
            if utils::move_towards_position(ecs, &info, &herbivorous_position, CARNIVOROUS_SPEED) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToHerbivorousComponent),
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
