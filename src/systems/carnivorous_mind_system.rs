use crate::components::*;
use crate::ecs::{Ecs, EntityId, System};
use crate::systems::utils;
use std::any::TypeId;
use std::collections::HashMap;

pub struct CarnivorousMindSystem;
impl System for CarnivorousMindSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Get the positions of all inactive carnivorous entities
        let mut carnivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in iter_entities_with!(
            ecs,
            CarnivorousComponent,
            PositionComponent,
            InactiveComponent
        ) {
            if let Some(position) = ecs.get_component::<PositionComponent>(arch_index, entity_index)
            {
                carnivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest corpse or herbivorous
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut is_corpse: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_entity_of: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &carnivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);

            // Check corpses
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, position, to_ctype!(CorpseComponent))
            {
                closest_distance_squared = distance_squared;
                found.insert(*entity, true);
                is_corpse.insert(*entity, true);
                closest_entity_of.insert(*entity, closest_entity);
            }

            // Check herbivorous
            if let Some((distance_squared, closest_entity)) =
                utils::find_closest(ecs, position, to_ctype!(HerbivorousComponent))
            {
                if distance_squared < closest_distance_squared {
                    found.insert(*entity, true);
                    is_corpse.insert(*entity, false);
                    closest_entity_of.insert(*entity, closest_entity);
                }
            }
        }

        // Assign action component to carnivorous that found a target
        for (carnivorous_entity, found) in found {
            if found {
                let found_entity = closest_entity_of.get(&carnivorous_entity).unwrap();
                if *is_corpse.get(&carnivorous_entity).unwrap() {
                    ecs.add_component(
                        carnivorous_entity,
                        &MoveToCorpseComponent::new(*found_entity),
                    );
                } else {
                    ecs.add_component(
                        carnivorous_entity,
                        &MoveToHerbivorousComponent::new(*found_entity),
                    );
                }
                ecs.remove_component(carnivorous_entity, to_ctype!(InactiveComponent));
            }
        }
    }
}
