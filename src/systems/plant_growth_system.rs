use crate::components::all::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct PlantGrowthSystem;
impl System for PlantGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (plant, _) in iter_components!(ecs, (), (PlantComponent)) {
            plant.size += plant.growth_per_tick;
            if plant.size > plant.max_size {
                plant.size = plant.max_size;
            }
        }

        /*
        let mut updates: Vec<Update> = Vec::new();
        for _ in 0..NEW_plant_PER_TICK {
            updates.push(Update::Create(vec![
                Box::new(plantComponent::new()),
                Box::new(BodyComponent::new_rand_pos_traversable(
                    plant_SIZE.into(),
                    plant_SIZE.into(),
                )),
            ]));
        }
        ecs.apply(updates);
        */
    }
}
