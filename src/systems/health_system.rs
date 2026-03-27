use crate::components::all::*;
use crate::configuration::Config;
use crate::ecs::{Ecs, System};
use std::any::TypeId;

pub struct HealthSystem;
impl System for HealthSystem {
    fn run(&self, ecs: &mut Ecs, config: &Config) {
        for (creature, _) in iter_components!(ecs, (), (CreatureComponent)) {
            creature.health += if creature.energy > 0.0 {
                config.creature.recovery_rate
            } else {
                -config.creature.exhaustion_rate
            };
            creature.health = creature.health.clamp(0.0, config.creature.max_health);
        }
    }
}
