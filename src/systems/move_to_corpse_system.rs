use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct MoveToCorpseSystem;
impl System for MoveToCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all carnivorous in the direction of their corpse target (if they have one)
        let mut creature_to_corpse: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            PositionComponent,
            MoveToCorpseComponent
        ) {
            // Get the target corpse info
            let c_entity;
            if let Some(MoveToCorpseComponent { corpse_entity }) =
                ecs.get_component::<MoveToCorpseComponent>(arch_index, entity_index)
            {
                c_entity = *corpse_entity;
            } else {
                // Go to inactive state if the corpse can't be found
                ecs.remove_component(entity, to_ctype!(MoveToCorpseComponent));
                ecs.add_component(entity, &InactiveComponent::new());
                continue;
            }

            // Get the corpse position
            let mut position_exists = false;
            let mut corpse_position = PositionComponent::new();
            if let Some((c_arch_index, c_entity_index)) = ecs.get_entity_indexes(c_entity) {
                if let Some(pos) =
                    ecs.get_component::<PositionComponent>(c_arch_index, c_entity_index)
                {
                    corpse_position = *pos;
                    position_exists = true;
                }
            }

            // Go to inactive state if the target position can't be found for some reason
            if !position_exists {
                ecs.remove_component(entity, to_ctype!(MoveToCorpseComponent));
                ecs.add_component(entity, &InactiveComponent::new());
                continue;
            }

            // Move the carnivorous towards the corpse target
            if utils::move_towards_position(
                ecs,
                arch_index,
                entity_index,
                &corpse_position,
                CREATURE_PIXEL_SIZE as f64,
                CREATURE_PIXEL_SIZE as f64,
                CARNIVOROUS_SPEED,
            ) {
                creature_to_corpse.insert(entity, c_entity);
            }
        }

        // If corpse reached, go to eating state
        for (entity, corpse_entity) in creature_to_corpse {
            ecs.remove_component(entity, to_ctype!(MoveToCorpseComponent));
            ecs.add_component(entity, &EatingCorpseComponent::new(corpse_entity));
        }
    }
}
