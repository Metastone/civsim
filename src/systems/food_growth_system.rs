use crate::components::all::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (food, _) in iter_components!(ecs, (), (FoodComponent)) {
            food.size += food.growth_per_tick;
            if food.size > food.max_size {
                food.size = food.max_size;
            }
        }

        /*
        let mut updates: Vec<Update> = Vec::new();
        for _ in 0..NEW_FOOD_PER_TICK {
            updates.push(Update::Create(vec![
                Box::new(FoodComponent::new()),
                Box::new(BodyComponent::new_rand_pos_traversable(
                    FOOD_SIZE.into(),
                    FOOD_SIZE.into(),
                )),
            ]));
        }
        ecs.apply(updates);
        */
    }
}
