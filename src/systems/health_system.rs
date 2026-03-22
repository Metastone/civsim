use crate::components::all::*;
use crate::constants::*;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct HealthSystem;
impl System for HealthSystem {
    fn run(&self, ecs: &mut Ecs) {
        for (creature, _) in iter_components!(ecs, (), (CreatureComponent)) {
            creature.health += if creature.energy > 0.0 {
                RECOVERY_RATE
            } else {
                -EXHAUSTION_RATE
            };
            creature.health = creature.health.clamp(0.0, MAX_HEALTH);
        }
    }
}
