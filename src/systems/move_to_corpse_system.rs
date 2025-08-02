use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct MoveToCorpseSystem;
impl System for MoveToCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all carnivorous in the direction of their corpse target (if they have one)
        let mut creature_to_eat_corpse: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creature_to_inactive: HashSet<EntityId> = HashSet::new();
        for info in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            PositionComponent,
            MoveToCorpseComponent
        ) {
            // Get the target corpse info
            let MoveToCorpseComponent { corpse_entity } =
                ecs.get_component::<MoveToCorpseComponent>(&info).unwrap();
            let c_entity = *corpse_entity;

            // Get the corpse position
            let corpse_position;
            if let Some(pos) = ecs.get_component_from_entity::<PositionComponent>(c_entity) {
                corpse_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found for some reason
                creature_to_inactive.insert(info.entity);
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
                creature_to_eat_corpse.insert(info.entity, c_entity);
            }
        }

        for entity in creature_to_inactive {
            ecs.remove_component(entity, to_ctype!(MoveToCorpseComponent));
            ecs.add_component(entity, &InactiveComponent::new());
            continue;
        }

        // If corpse reached, go to eating state
        for (entity, corpse_entity) in creature_to_eat_corpse {
            ecs.remove_component(entity, to_ctype!(MoveToCorpseComponent));
            ecs.add_component(entity, &EatingCorpseComponent::new(corpse_entity));
        }
    }
}
