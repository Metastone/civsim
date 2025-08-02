use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct MoveToHerbivorousSystem;
impl System for MoveToHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Move all carnivorous in the direction of their herbivorous target (if they have one)
        let mut creature_to_herbivorous: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creature_to_inactive: HashSet<EntityId> = HashSet::new();
        for info in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            PositionComponent,
            MoveToHerbivorousComponent
        ) {
            // Get the target herbivorous info
            let MoveToHerbivorousComponent { herbivorous_entity } = ecs
                .get_component::<MoveToHerbivorousComponent>(&info)
                .unwrap();
            let herb_entity = *herbivorous_entity;

            // Get the herbivorous position
            let herbivorous_position;
            if let Some(pos) = ecs.get_component_from_entity::<PositionComponent>(herb_entity) {
                herbivorous_position = *pos;
            } else {
                // Go to inactive state if the target position can't be found
                creature_to_inactive.insert(info.entity);
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

        for entity in creature_to_inactive {
            ecs.remove_component(entity, to_ctype!(MoveToHerbivorousComponent));
            ecs.add_component(entity, &InactiveComponent::new());
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
