use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};

pub struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, ecs: &mut Ecs) {
        for _ in 0..NEW_FOOD_PER_TICK {
            ecs.create_entity_with(&[&FoodComponent::new(), &PositionComponent::new()]);
        }
    }
}
