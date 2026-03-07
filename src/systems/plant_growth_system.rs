use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct PlantGrowthSystem;
impl System for PlantGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (plant, body, info) in iter_components!(ecs, (), (PlantComponent, BodyComponent)) {
            // Grow plant, if there is enough space
            let new_size = (plant.size + plant.growth_per_tick).min(plant.max_size);
            if new_size != plant.size && body.try_update_size(info.entity, new_size, new_size) {
                plant.size = new_size;
            }

            // Grow new seeds
            if plant.count_ticks_to_seed >= plant.ticks_per_seed {
                plant.count_ticks_to_seed = 0;
                plant.nb_seeds = (plant.nb_seeds + 1).min(plant.max_nb_seeds);
            }
            plant.count_ticks_to_seed += 1;
        }
    }
}
