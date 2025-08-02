use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct ReproductionSystem;
impl System for ReproductionSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Find creatures that can reproduce
        let mut positions = Vec::new();
        let mut is_herbivorous = Vec::new();
        for info in iter_entities_with!(ecs, CreatureComponent, PositionComponent) {
            // If the creature can reproduce, reset its energy to start value
            let creature = ecs.get_component_mut::<CreatureComponent>(&info).unwrap();
            if creature.energy >= REPROD_ENERGY_THRESHOLD {
                creature.energy -= REPROD_ENERGY_COST;
            } else {
                continue;
            }

            // Store the creature's position
            let position = ecs.get_component::<PositionComponent>(&info).unwrap();
            positions.push(*position);

            // Check if herbivorous or carnivorous
            is_herbivorous
                .push(ecs.has_component(info.arch_index, &to_ctype!(HerbivorousComponent)));
        }

        // Create one new creature next to each reproducing create
        for (position, is_h) in positions.iter().zip(is_herbivorous) {
            if is_h {
                ecs.create_entity_with(&[
                    &CreatureComponent::new(),
                    &HerbivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                    &InactiveComponent::new(),
                ]);
            } else {
                ecs.create_entity_with(&[
                    &CreatureComponent::new(),
                    &CarnivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                    &InactiveComponent::new(),
                ]);
            }
        }
    }
}
