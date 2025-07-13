use crate::components::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut to_remove = Vec::new();
        let mut positions = Vec::new();

        for (arch_index, entity_index, entity) in
            iter_entities_with!(ecs, CreatureComponent, PositionComponent)
        {
            // Check if the creature should die
            if let Some(creature) = ecs.get_component::<CreatureComponent>(arch_index, entity_index)
            {
                if creature.health <= 0.0 {
                    to_remove.push(entity);
                } else {
                    continue;
                }
            }

            // Store the creature's position
            if let Some(position) = ecs.get_component::<PositionComponent>(arch_index, entity_index)
            {
                positions.push(*position);
            }
        }

        // Delete dead creature entities
        to_remove
            .iter()
            .for_each(|entity| ecs.remove_entity(*entity));

        // Create a corpse
        for position in positions {
            ecs.create_entity_with(&[&CorpseComponent::new(), &position]);
        }
    }
}
