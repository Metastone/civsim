use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Ecs, System, Update};
use crate::systems::utils;
use std::any::TypeId;

pub struct MoveToCorpseSystem;
impl System for MoveToCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Move all carnivorous in the direction of their corpse target (if they have one)
        for info in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            BodyComponent,
            MoveToCorpseComponent
        ) {
            // Get the target corpse info
            let MoveToCorpseComponent { corpse_entity } =
                ecs.get_component::<MoveToCorpseComponent>(&info).unwrap();
            let c_entity = *corpse_entity;

            // Get the corpse position
            let corpse_position;
            if let Some(pos) = ecs.get_component_from_entity::<BodyComponent>(c_entity) {
                corpse_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found for some reason
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToCorpseComponent),
                });
                updates.push(Update::Add {
                    info,
                    comp: Box::new(InactiveComponent::new()),
                });
                continue;
            }

            // Move the carnivorous towards the corpse target
            if utils::move_towards_position(
                ecs,
                &info,
                &corpse_position,
                CREATURE_PIXEL_SIZE as f64,
                CREATURE_PIXEL_SIZE as f64,
                CARNIVOROUS_SPEED,
            ) {
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToCorpseComponent),
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
