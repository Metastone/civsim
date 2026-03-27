use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, System};
use crate::humidity;
use crate::shared_data::body_grid;
use std::any::TypeId;

pub struct PlantGrowthSystem;
impl System for PlantGrowthSystem {
    fn run(&self, ecs: &mut Ecs, config: &Config) {
        for (plant, body, info) in iter_components!(ecs, (), (PlantComponent, BodyComponent)) {
            // Initialize seeds with humidity level
            if !plant.is_seed_initialized {
                plant.init_seed(config, humidity(body.x(), body.y()));
            }

            // Grow from seed to plant
            if plant.is_seed {
                if plant.countdown_ticks_as_seed == 0 {
                    if body_grid::try_update_size(
                        info.entity,
                        body,
                        config.plant.initial_size,
                        config.plant.initial_size,
                    ) {
                        plant.become_plant(config, humidity(body.x(), body.y()));

                        // Add collision to the plant
                        // TODO not great to have to do this both in ECS and in body grid...
                        body_grid::set_traversable(info.entity, body, false);
                        body.set_traversable(false);
                    }
                } else {
                    plant.countdown_ticks_as_seed -= 1;
                    continue;
                }
            }

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
