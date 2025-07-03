use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, System};

pub struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for _ in 0..NEW_FOOD_PER_TICK {
            manager.create_entity_with(&[&FoodComponent::new(), &PositionComponent::new()]);
        }
    }
}
