use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Component, Ecs, System, Update, RESERVED_ENTITY_ID};
use crate::shared_data::body_grid;
use std::any::TypeId;

pub struct ReproductionSystem;
impl System for ReproductionSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Find creatures that can reproduce
        for info in iter_entities_with!(ecs, CreatureComponent, BodyComponent) {
            // Check if the creature has enough energy to reproduce
            {
                let creature = ecs.get_component_mut::<CreatureComponent>(&info).unwrap();
                if creature.energy < REPROD_ENERGY_THRESHOLD {
                    continue;
                }
            }

            let body = ecs.get_component::<BodyComponent>(&info).unwrap();
            let new_body = BodyComponent::new_not_traversable(
                body.get_x() + CREATURE_PIXEL_SIZE as f64,
                body.get_y(),
                CREATURE_PIXEL_SIZE.into(),
                CREATURE_PIXEL_SIZE.into(),
            );

            // Reproduce only if there is a free space for the new creature
            if body_grid::collides_in_surronding_cells(RESERVED_ENTITY_ID, &new_body) {
                continue;
            }

            let mut comps: Vec<Box<dyn Component>> = vec![
                Box::new(CreatureComponent::new()),
                Box::new(new_body),
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

            // Apply reproduction energy cost to parent creature
            {
                let creature = ecs.get_component_mut::<CreatureComponent>(&info).unwrap();
                creature.energy -= REPROD_ENERGY_COST;
            }
        }

        ecs.apply(updates);
    }
}
