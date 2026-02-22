use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{Ecs, System, Update};

pub struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
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
    }
}
