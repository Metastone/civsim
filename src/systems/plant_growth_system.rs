use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, System, Update, iter_components};
use crate::humidity;
use std::any::TypeId;

pub struct PlantGrowthSystem;
impl System for PlantGrowthSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        for (seed, body, info) in iter_components!(ecs, (), (SeedComponent, BodyComponent)) {
            let h = humidity(body.x(), body.y());

            // Initialize seed with humidity level
            if !seed.is_seed_initialized {
                seed.init_seed(h);
            }

            if seed.countdown_ticks_as_seed == 0 {
                updates.push(Update::DeleteEntity(info));

                let plant = PlantGrowthComponent::new(config, seed.plant_kind, h);
                match seed.plant_kind {
                    PlantKind::Bush => updates.push(Update::Create(vec![
                        Box::new(BodyComponent::new_not_traversable(
                            body.x(),
                            body.y(),
                            plant.width,
                            plant.width,
                        )),
                        Box::new(BushComponent {}),
                        Box::new(plant),
                        Box::new(PlantWithFruitComponent::new(config, h)),
                        Box::new(PlantWithStolonComponent::new(config, h)),
                    ])),
                    PlantKind::Tree => updates.push(Update::Create(vec![
                        Box::new(BodyComponent::new_not_traversable(
                            body.x(),
                            body.y(),
                            plant.width,
                            plant.width,
                        )),
                        Box::new(TreeComponent {}),
                        Box::new(plant),
                        Box::new(PlantWithFruitComponent::new(config, h)),
                    ])),
                }
            } else {
                seed.countdown_ticks_as_seed -= 1;
                continue;
            }
        }

        // Grow plant, if there is enough space
        for (plant, body, info) in iter_components!(ecs, (), (PlantGrowthComponent, BodyComponent))
        {
            let new_size = (plant.width + plant.growth_per_tick).min(plant.max_width);
            if new_size != plant.width && body.try_update_size(info.entity, new_size, new_size) {
                plant.width = new_size;
            }
        }

        // Grow fruits
        for (plant, _) in iter_components!(ecs, (), (PlantWithFruitComponent)) {
            plant.grow_fruits();
            if plant.count_ticks_to_fruit >= plant.ticks_per_fruit {
                plant.count_ticks_to_fruit = 0;
                plant.add_new_fruit(config);
            }
            plant.count_ticks_to_fruit += 1;
        }

        // Grow stolons on plants
        for (plant, _) in iter_components!(ecs, (), (PlantWithStolonComponent)) {
            plant.stolon_length =
                (plant.stolon_length + plant.stolon_growth_per_tick).min(plant.stolon_max_length);
        }

        ecs.apply(updates);
    }
}
