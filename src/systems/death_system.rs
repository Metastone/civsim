use crate::components::*;
use crate::ecs::{ArchetypeManager, ComponentType, System};

pub struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        let mut to_remove = Vec::new();
        let mut positions = Vec::new();

        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Creature, ComponentType::Position])
        {
            // Check if the creature should die
            if let Some(creature) = manager.get_component::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if creature.health <= 0.0 {
                    to_remove.push(entity);
                } else {
                    continue;
                }
            }

            // Store the creature's position
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                positions.push(*position);
            }
        }

        // Delete dead creature entities
        to_remove
            .iter()
            .for_each(|entity| manager.remove_entity(*entity));

        // Create a corpse
        for position in positions {
            manager.create_entity_with(&[&CorpseComponent::new(), &position]);
        }
    }
}
