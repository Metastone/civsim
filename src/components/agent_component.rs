use std::collections::HashMap;

use crate::{configuration::Config, ecs::{Component, Ecs, EntityInfo, Update}};
use log::error;

#[derive(Clone, PartialEq, Eq)]
pub enum Symbol {
    Energy,
    Health,
    IsNearPlant,
}

#[derive(Clone)]
pub enum Value {
    F32(f32),
    Bool(bool),
}

#[derive(PartialEq, Eq)]
pub enum Condition {
    Equal,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Not,
}

#[derive(PartialEq, Eq)]
pub enum Modifier {
    SetValue,
    Decrement,
    Increment,
}

pub struct Precondition {
    symbol: Symbol,
    condition: Condition,
    value: Value,
}

pub struct Effect {
    symbol: Symbol,
    modifier: Modifier,
    value: Value,
}

#[derive(Clone)]
pub struct WorldState {
    facts: HashMap<Symbol, Value>,
}
impl WorldState {
    fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }
}

pub enum ActionResult {
    OnGoing,
    Success,
    Failure,
}

pub trait Action {
    fn get_precondition(&self) -> &'static [Precondition];
    fn get_effects(&self) -> &'static [Effect];
    fn perform(&self, ecs: &Ecs, updates: &mut Vec<Update>, info: &EntityInfo, config: &Config, world_state: &mut WorldState) -> ActionResult;
}

pub struct MoveToNearestPlantAction;
impl Action for MoveToNearestPlantAction {
    fn get_precondition(&self) -> &'static [Precondition] {
        &[]
    }

    fn get_effects(&self) -> &'static [Effect] {
        static EFFECTS: [Effect; 1] = [Effect {
            symbol: Symbol::IsNearPlant,
            modifier: Modifier::SetValue,
            value: Value::Bool(true),
        }];
        &EFFECTS
    }

    fn perform(&self, ecs: &Ecs, updates: &mut Vec<Update>, info: &EntityInfo, config: &Config, world_state: &mut WorldState) -> ActionResult {
        // TODO implement
        ActionResult::Success
    }
}

pub trait Goal {}

pub struct ReplenishEnergyGoal {}
impl Goal for ReplenishEnergyGoal {}

pub struct IdleGoal {}
impl Goal for IdleGoal {}

pub struct GoalSet {
    goals: Vec<Box<dyn Goal>>,
}
impl GoalSet {
    pub fn new() -> Self {
        GoalSet { goals: Vec::new() }
    }
    pub fn add(&mut self, goal: Box<dyn Goal>) {
        self.goals.push(goal);
    }
}

pub struct ActionSet {
    actions: Vec<Box<dyn Action>>,
}

impl ActionSet {
    pub fn new() -> Self {
        ActionSet {
            actions: Vec::new(),
        }
    }
    pub fn add(&mut self, goal: Box<dyn Action>) {
        self.actions.push(goal);
    }
}

/// Goal-Oriented Action Planner
pub struct Goap {
    goal_sets: Vec<GoalSet>,
    action_sets: Vec<ActionSet>,
}
impl Goap {
    pub fn new() -> Self {
        Goap {
            goal_sets: Vec::new(),
            action_sets: Vec::new(),
        }
    }

    pub fn add_goal_set(&mut self, goal_set: GoalSet) -> usize {
        self.goal_sets.push(goal_set);
        self.goal_sets.len() - 1
    }

    pub fn add_action_set(&mut self, action_set: ActionSet) -> usize {
        self.action_sets.push(action_set);
        self.action_sets.len() - 1
    }

    pub fn find_goal(&self, ecs: &Ecs, config: &Config, info: &EntityInfo, goal_set: usize) -> Option<usize> {
        // TODO
        None
    }

    pub fn compute_plan(
        &self,
        ecs: &Ecs,
        ecs_updates: &mut Vec<Update>,
        config: &Config,
        goal: usize,
        goal_set: usize,
        action_set: usize,
    ) -> Option<Vec<usize>> {
        None
    }

    // assume that precondition is not already satisfied
    fn validates_precondition(
        &self,
        action: usize,
        action_set: usize,
        precond: &Precondition,
        world_state: &WorldState,
    ) -> bool {
        // TODO take world_state into account
        // TODO assume indexes are valid or check ?
        for effect in self.action_sets[action_set].actions[action].get_effects() {
            // TODO check other possibilities (increment, etc)
            if effect.symbol == precond.symbol && effect.modifier == Modifier::SetValue {
                match effect.value {
                    Value::Bool(e_b) => {
                        if let Value::Bool(p_b) = precond.value && e_b == p_b {
                            return true;
                        }
                    }
                    Value::F32(e_f) => {
                        if let Value::F32(p_f) = precond.value && e_f == p_f {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn perform_action(
        &self,
        ecs: &Ecs,
        ecs_updates: &mut Vec<Update>,
        info: &EntityInfo,
        config: &Config,
        world_state: &mut WorldState,
        action: usize,
        action_set: usize,
    ) -> ActionResult {
        if action_set >= self.action_sets.len() {
            error!("No action set with index {}", action_set);
            return ActionResult::Failure;
        }
        if action >= self.action_sets[action_set].actions.len() {
            error!(
                "No action with index {} in action set {}",
                action, action_set
            );
            return ActionResult::Failure;
        }
        self.action_sets[action_set].actions[action].perform(ecs, ecs_updates, info, config, world_state)
    }
}

#[derive(Clone)]
pub struct AgentComponent {
    goal: Option<usize>,
    goal_set: usize,
    action_set: usize,
    plan: Vec<usize>,
    current_action_index_in_plan: usize,
    world_state: WorldState,
}

impl Component for AgentComponent {}

impl AgentComponent {
    pub fn new(goal_set: usize, action_set: usize) -> Self {
        AgentComponent {
            goal: None,
            goal_set,
            action_set,
            plan: Vec::new(),
            current_action_index_in_plan: 0,
            world_state: WorldState::new(),
        }
    }

    pub fn advance_to_next_action(&mut self) {
        self.current_action_index_in_plan += 1;
        if self.current_action_index_in_plan >= self.plan.len() {
            self.reset_plan();
            self.goal = None;
        }
    }

    pub fn reset_plan(&mut self) {
        self.current_action_index_in_plan = 0;
        self.plan.clear();
    }

    pub fn set_plan(&mut self, plan: Vec<usize>) {
        self.plan = plan
    }

    pub fn has_plan(&self) -> bool {
        !self.plan.is_empty()
    }

    pub fn set_goal(&mut self, goal: usize) {
        self.goal = Some(goal);
    }

    pub fn goal(&self) -> Option<usize> {
        self.goal
    }

    pub fn goal_set(&self) -> usize {
        self.goal_set
    }

    pub fn action(&self) -> Option<usize> {
        if self.current_action_index_in_plan < self.plan.len() {
            return Some(self.plan[self.current_action_index_in_plan])
        }
        None
    }

    pub fn action_set(&self) -> usize {
        self.action_set
    }

    pub fn set_world_state(&mut self, world_state: &WorldState) {
        self.world_state = world_state.clone()
    }

    pub fn world_state(&self) -> &WorldState {
        &self.world_state
    }
}

