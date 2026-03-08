use crate::components::all::{HerbivorousComponent, PlantComponent};
use crate::components::body_component::BodyComponent;
use crate::constants::SEED_SIZE;
use crate::ecs::{Ecs, System, Update, RESERVED_ENTITY_ID};
use crate::shared_data::body_grid;
use crate::TypeId;
use std::f64::consts::PI;

pub struct DigestionSystem;
impl System for DigestionSystem {
    fn run(&self, ecs: &mut Ecs) {
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
                    SEED_SIZE,
                    SEED_SIZE,
                );

                if !body_grid::collides(RESERVED_ENTITY_ID, &seed_body) {
                    updates.push(Update::Create(vec![
                        Box::new(seed_body),
                        Box::new(PlantComponent::new()),
                    ]));
                }
            }

            herbivorous.seeds.pop_front();
        }

        ecs.apply(updates);
    }
}
