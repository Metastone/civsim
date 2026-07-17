use crate::components::all::*;
use crate::configuration::Config;
use crate::ecs::{Ecs, System, iter_components};
use std::any::TypeId;

pub struct HealthSystem;
impl System for HealthSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        for (creature, _) in iter_components!(ecs, (), (CreatureComponent)) {
            if creature.energy() > 0.0 {
                creature.increment_health(config.creature.recovery_rate);
            } else {
                creature.increment_health(-config.creature.exhaustion_rate);
            };
        }
    }
}
