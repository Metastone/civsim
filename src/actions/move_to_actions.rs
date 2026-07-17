use crate::{
    actions::all::get_comp_or_error,
    components::{
        agent_component::AgentComponent,
        all::{
            CorpseComponent, HerbivorousComponent, MoveToTargetResultComponent,
            PlantWithFruitComponent,
        },
        body_component::BodyComponent,
        move_to_target_component::MoveToTargetComponent,
    },
    configuration::Config,
    ecs::{Component, Ecs, EntityInfo, Update, to_ctype},
    goap::{Action, ActionResult, Condition, Effect, Modifier, Symbol, Value},
    systems::utils,
};
use std::any::TypeId;

/// Create a MoveToTarget component and let the corresponding system handle the move.
/// Consider that the move is finished when the entity has a TargetReached component.
fn perform_move_to_target_action<A, C, F>(
    ecs: &mut Ecs,
    info: &EntityInfo,
    config: &Config,
    speed: f64,
    condition: F,
) -> Result<ActionResult, String>
where
    A: Action,
    C: Component,
    F: Fn(&C) -> bool,
{
    // If the move is over, return the result
    if let Some(result) = ecs.component::<MoveToTargetResultComponent>(info).cloned() {
        ecs.apply(vec![Update::Delete {
            info: *info,
            c_type: to_ctype!(MoveToTargetResultComponent),
        }]);
        return if result.success {
            Ok(ActionResult::Success)
        } else {
            Ok(ActionResult::Failure)
        };
    }

    // If the move is on-going, do nothing
    if ecs.has_component(info.arch_index, &to_ctype!(MoveToTargetComponent)) {
        return Ok(ActionResult::OnGoing);
    }

    // If we reach this point, the move must be initiated

    // Get the body component
    let body = *get_comp_or_error::<A, BodyComponent>(ecs, info)?;

    // Find the closest reachable entity (if there is one)
    // with the given component, and which satisfies the condition
    if let Some((_, closest_entity, closest_body, closest_path)) =
        utils::find_closest_reachable::<C, F>(ecs, config, info.entity, &body, condition)
    {
        let agent = get_comp_or_error::<A, AgentComponent>(ecs, info)?;
        agent.target_entity = closest_entity;

        // Target found, initiate the move
        ecs.apply(vec![Update::Add {
            info: *info,
            comp: Box::new(MoveToTargetComponent::new(
                closest_entity,
                closest_body,
                closest_path,
                speed,
            )),
        }]);
        Ok(ActionResult::OnGoing)
    } else {
        // No target found, go into idle state to lower cpu load
        let agent = get_comp_or_error::<A, AgentComponent>(ecs, info)?;
        agent.go_idle();
        Ok(ActionResult::Failure)
    }
}

pub struct MoveToNearestPlantWithFruitAction {
    effects: [Effect; 1],
}
impl MoveToNearestPlantWithFruitAction {
    pub fn new() -> Self {
        Self {
            effects: [Effect::new(
                Symbol::IsNearPlantWithFruit,
                Modifier::SetValue,
                Value::Bool(true),
            )],
        }
    }
}
impl Action for MoveToNearestPlantWithFruitAction {
    fn preconditions(&self) -> &[Condition] {
        &[]
    }

    fn effects(&self) -> &[Effect] {
        &self.effects
    }

    fn perform(
        &self,
        ecs: &mut Ecs,
        info: &EntityInfo,
        config: &Config,
    ) -> Result<ActionResult, String> {
        perform_move_to_target_action::<MoveToNearestPlantWithFruitAction, PlantWithFruitComponent, _>(
            ecs,
            info,
            config,
            config.creature.herbivorous_speed,
            |plant| plant.has_fruits(),
        )
    }

    fn description(&self) -> String {
        String::from("move to nearest plant")
    }
}

pub struct MoveToNearestCorpseAction {
    effects: [Effect; 1],
}
impl MoveToNearestCorpseAction {
    pub fn new() -> Self {
        Self {
            effects: [Effect::new(
                Symbol::IsNearCorpse,
                Modifier::SetValue,
                Value::Bool(true),
            )],
        }
    }
}
impl Action for MoveToNearestCorpseAction {
    fn preconditions(&self) -> &[Condition] {
        &[]
    }

    fn effects(&self) -> &[Effect] {
        &self.effects
    }

    fn perform(
        &self,
        ecs: &mut Ecs,
        info: &EntityInfo,
        config: &Config,
    ) -> Result<ActionResult, String> {
        perform_move_to_target_action::<MoveToNearestCorpseAction, CorpseComponent, _>(
            ecs,
            info,
            config,
            config.creature.carnivorous_speed,
            |_| true,
        )
    }

    fn description(&self) -> String {
        String::from("move to nearest corpse")
    }
}

pub struct MoveToNearestHerbivorousAction {
    effects: [Effect; 1],
}
impl MoveToNearestHerbivorousAction {
    pub fn new() -> Self {
        Self {
            effects: [Effect::new(
                Symbol::IsNearHerbivorous,
                Modifier::SetValue,
                Value::Bool(true),
            )],
        }
    }
}
impl Action for MoveToNearestHerbivorousAction {
    fn preconditions(&self) -> &[Condition] {
        &[]
    }

    fn effects(&self) -> &[Effect] {
        &self.effects
    }

    fn perform(
        &self,
        ecs: &mut Ecs,
        info: &EntityInfo,
        config: &Config,
    ) -> Result<ActionResult, String> {
        perform_move_to_target_action::<MoveToNearestHerbivorousAction, HerbivorousComponent, _>(
            ecs,
            info,
            config,
            // TODO speed should depend on the running agent ideally (so that the action may be
            // attributed to herbivorous agents also, not just carninvorous)
            config.creature.carnivorous_speed,
            |_| true,
        )
    }

    fn description(&self) -> String {
        String::from("move to nearest herbivorous")
    }
}
