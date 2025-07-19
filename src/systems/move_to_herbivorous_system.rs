use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct MoveToHerbivorousSystem;
impl System for MoveToHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all carnivorous in the direction of their herbivorous target (if they have one)
        let mut creature_to_herbivorous: HashMap<EntityId, EntityId> = HashMap::new();
        for info in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            PositionComponent,
            MoveToHerbivorousComponent
        ) {
            // Get the target herbivorous info
            let herb_entity;
            if let Some(MoveToHerbivorousComponent { herbivorous_entity }) =
                ecs.get_component::<MoveToHerbivorousComponent>(&info)
            {
                herb_entity = *herbivorous_entity;
            } else {
                // Go to inactive state if the herbivorous can't be found
                ecs.remove_component(info.entity, to_ctype!(MoveToHerbivorousComponent));
                ecs.add_component(info.entity, &InactiveComponent::new());
                continue;
            }

            // Get the herbivorous position
            let mut position_exists = false;
            let mut herbivorous_position = PositionComponent::new();
            if let Some(pos) = ecs.get_component_from_entity::<PositionComponent>(herb_entity) {
                herbivorous_position = *pos;
                position_exists = true;
            }

            // Go to inactive state if the target position can't be found
            if !position_exists {
                ecs.remove_component(info.entity, to_ctype!(MoveToHerbivorousComponent));
                ecs.add_component(info.entity, &InactiveComponent::new());
                continue;
            }

            // Move the carnivorous towards the herbivorous target
            if utils::move_towards_position(
                ecs,
                &info,
                &herbivorous_position,
                CREATURE_PIXEL_SIZE as f64,
                CREATURE_PIXEL_SIZE as f64,
                CARNIVOROUS_SPEED,
            ) {
                creature_to_herbivorous.insert(info.entity, herb_entity);
            }
        }

        // If herbivorous reached, go to eating state
        for (entity, herbivorous_entity) in creature_to_herbivorous {
            ecs.remove_component(entity, to_ctype!(MoveToHerbivorousComponent));
            ecs.add_component(
                entity,
                &AttackingHerbivorousComponent::new(herbivorous_entity),
            );
        }
    }
}
