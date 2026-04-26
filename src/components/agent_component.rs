use crate::{ecs::Component, goap::WorldState};

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
        }
    }

    pub fn reset_plan(&mut self) {
        self.current_action_index_in_plan = 0;
        self.plan.clear();
        self.goal = None;
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
            return Some(self.plan[self.current_action_index_in_plan]);
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
