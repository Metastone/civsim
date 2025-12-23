use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct TargetCorpseSystem;
impl System for TargetCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Move all carnivorous in the direction of their corpse target (if they have one)
        for info in iter_entities!(
            ecs,
            CarnivorousComponent,
            BodyComponent,
            TargetCorpseComponent
        ) {
            // Get the target corpse info
            let TargetCorpseComponent { corpse_entity } =
                ecs.get_component::<TargetCorpseComponent>(&info).unwrap();
            let c_entity = *corpse_entity;

            // Get the corpse body
            let corpse_body;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(c_entity) {
                corpse_body = *pos;
            } else {
                // Go to inactive state if the target body can't be found for some reason
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetCorpseComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            let body = ecs.get_component::<BodyComponent>(&info).unwrap();

            if utils::is_target_reached(body, &corpse_body) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(TargetCorpseComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(EatingCorpseComponent::new(c_entity)),
                });
            }
        }

        ecs.apply(updates);
    }
}
