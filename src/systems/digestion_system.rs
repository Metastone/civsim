use crate::TypeId;
use crate::components::all::{HerbivorousComponent, PlantComponent};
use crate::components::body_component::BodyComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, RESERVED_ENTITY_ID, System, Update};
use crate::shared_data::body_grid;
use std::f64::consts::PI;

pub struct DigestionSystem;
impl System for DigestionSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        for (herbivorous, body, _) in
            iter_components!(ecs, (), (HerbivorousComponent, BodyComponent))
        {
            // Get the seeds closest to being excreted
            if herbivorous.seeds.is_empty() {
                continue;
            }
            let (nb_seeds, ref mut coutdown_to_excretion) = herbivorous.seeds[0];

            // Check if seeds ready to be excreted
            if *coutdown_to_excretion > 0 {
                *coutdown_to_excretion -= 1;
                continue;
            }

            // Create plants by shitting the seeds in a circle around the herbivorous
            let arc = 2.0 * PI / (nb_seeds as f64);
            let mut a: f64 = 0.0;
            let body_size = body.w(); // Assume the herbivorous is squared
            for _ in 0..nb_seeds {
                let x = a.cos() * body_size;
                let y = a.sin() * body_size;
                a += arc;

                // Plants start as seed, which have no collision. They gain collision later on.
                let seed_body = BodyComponent::new_traversable(
                    body.x() + x,
                    body.y() + y,
                    config.seed.size,
                    config.seed.size,
                );

                if !body_grid::collides(RESERVED_ENTITY_ID, &seed_body) {
                    updates.push(Update::Create(vec![
                        Box::new(seed_body),
                        Box::new(PlantComponent::new(config)),
                    ]));
                }
            }

            herbivorous.seeds.pop_front();
        }

        ecs.apply(updates);
    }
}
