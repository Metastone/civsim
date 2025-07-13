use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, System};
use std::any::TypeId;

pub struct ReproductionSystem;
impl System for ReproductionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Find creatures that can reproduce
        let mut positions = Vec::new();
        let mut is_herbivorous = Vec::new();
        for (arch_index, entity_index, _entity) in
            iter_entities_with!(manager, CreatureComponent, PositionComponent)
        {
            // If the creature can reproduce, reset its energy to start value
            if let Some(creature) =
                manager.get_component_mut::<CreatureComponent>(arch_index, entity_index)
            {
                if creature.energy >= REPROD_ENERGY_THRESHOLD {
                    creature.energy -= REPROD_ENERGY_COST;
                } else {
                    continue;
                }
            }

            // Store the creature's position
            if let Some(position) =
                manager.get_component::<PositionComponent>(arch_index, entity_index)
            {
                positions.push(*position);
            } else {
                continue;
            }

            // Check if herbivorous or carnivorous
            is_herbivorous
                .push(manager.has_component(arch_index, &to_ctype!(HerbivorousComponent)));
        }

        // Create one new creature next to each reproducing create
        for (position, is_h) in positions.iter().zip(is_herbivorous) {
            if is_h {
                manager.create_entity_with(&[
                    &CreatureComponent::new(),
                    &HerbivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                    &InactiveComponent::new(),
                ]);
            } else {
                manager.create_entity_with(&[
                    &CreatureComponent::new(),
                    &CarnivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                    &InactiveComponent::new(),
                ]);
            }
        }
    }
}
