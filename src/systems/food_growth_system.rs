use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, System, Update};

pub struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();
        for _ in 0..NEW_FOOD_PER_TICK {
            updates.push(Update::Create(vec![
                Box::new(FoodComponent::new()),
                Box::new(PositionComponent::new()),
            ]));
        }
        ecs.apply(updates);
    }
}
