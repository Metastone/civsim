use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Component, Ecs, System, Update};
use std::any::TypeId;

pub struct ReproductionSystem;
impl System for ReproductionSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Find creatures that can reproduce
        for info in iter_entities_with!(ecs, CreatureComponent, BodyComponent) {
            // If the creature can reproduce, reset its energy to start value
            let creature = ecs.get_component_mut::<CreatureComponent>(&info).unwrap();
            if creature.energy >= REPROD_ENERGY_THRESHOLD {
                creature.energy -= REPROD_ENERGY_COST;
            } else {
                continue;
            }

            let position = ecs.get_component::<BodyComponent>(&info).unwrap();
            let mut comps: Vec<Box<dyn Component>> = vec![
                Box::new(CreatureComponent::new()),
                Box::new(BodyComponent::new_with_collision(
                    position.get_x() + CREATURE_PIXEL_SIZE as f64,
                    position.get_y(),
                    CREATURE_PIXEL_SIZE.into(),
                    CREATURE_PIXEL_SIZE.into(),
                )),
                Box::new(InactiveComponent::new()),
            ];

            // Check if herbivorous or carnivorous
            let is_herbivorous =
                ecs.has_component(info.arch_index, &to_ctype!(HerbivorousComponent));
            if is_herbivorous {
                comps.push(Box::new(HerbivorousComponent::new()));
            } else {
                comps.push(Box::new(CarnivorousComponent::new()));
            }

            // Create a new creature
            updates.push(Update::Create(comps));
        }

        ecs.apply(updates);
    }
}
