use crate::{
    configuration::Config,
    ecs::{Ecs, EntityInfo, Update},
    goap::{Action, ActionResult, Condition, Effect, Modifier, Symbol, Value, WorldState},
};

pub struct MoveToNearestPlantAction {
    effects: [Effect; 1],
}

impl MoveToNearestPlantAction {
    pub fn new() -> Self {
        Self {
            effects: [Effect::new(
                Symbol::IsNearPlant,
                Modifier::SetValue,
                Value::Bool(true),
            )],
        }
    }
}

impl Action for MoveToNearestPlantAction {
    fn preconditions(&self) -> &[Condition] {
        &[]
    }

    fn effects(&self) -> &[Effect] {
        &self.effects
    }

    fn perform(
        &self,
        ecs: &Ecs,
        updates: &mut Vec<Update>,
        info: &EntityInfo,
        config: &Config,
        world_state: &mut WorldState,
    ) -> ActionResult {
        ActionResult::Success
    }
}
