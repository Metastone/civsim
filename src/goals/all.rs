use ordered_float::OrderedFloat;

use crate::{
    components::all::CreatureComponent,
    configuration::Config,
    ecs::{Ecs, EntityInfo},
    goap::{Condition, Goal, Operator, Symbol, Value},
};

pub struct ReplenishEnergyGoal {
    max_energy: f32,
    preconditions: [Condition; 1],
}
impl ReplenishEnergyGoal {
    pub fn new(config: &Config) -> Self {
        let max_energy = config.creature.max_energy;
        Self {
            max_energy,
            preconditions: [Condition::new(
                Symbol::Energy,
                Operator::GreaterOrEqual,
                Value::F32(OrderedFloat(max_energy)),
            )],
        }
    }
}
impl Goal for ReplenishEnergyGoal {
    fn conditions(&self) -> &[Condition] {
        &self.preconditions
    }
    fn utility(&self, ecs: &Ecs, info: &EntityInfo) -> f32 {
        let creature = ecs.component::<CreatureComponent>(info).unwrap();
        f32::max(self.max_energy - creature.energy(), 0.0)
    }
}
