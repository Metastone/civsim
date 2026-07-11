use crate::{
    configuration::Config,
    ecs::{Component, EntityId, RESERVED_ENTITY_ID},
    goap::{Goap, WorldState},
};
use log::error;

#[derive(Clone)]
pub struct AgentComponent {
    pub goal: Option<usize>,
    goal_set: usize,
    action_set: usize,
    action_costs: Vec<f64>,
    plan: Vec<usize>,
    current_action_index_in_plan: usize,
    pub world_state: WorldState,
    idle: bool,
    idle_ticks_count: usize,
    pub target_entity: EntityId,
}

impl Component for AgentComponent {}

impl AgentComponent {
    pub fn new(goal_set: usize, action_set: usize, action_set_len: usize) -> Self {
        AgentComponent {
            goal: None,
            goal_set,
            action_set,
            action_costs: vec![1.0; action_set_len],
            plan: Vec::new(),
            current_action_index_in_plan: 0,
            world_state: WorldState::new(),
            idle: false,
            idle_ticks_count: 0,
            target_entity: RESERVED_ENTITY_ID,
        }
    }

    pub fn next_action(&mut self) {
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

    pub fn increase_action_cost(&mut self, action: usize) {
        if action < self.action_costs.len() {
            self.action_costs[action] += 1.0;
        } else {
            error!("No action with index {action}");
        }
    }

    pub fn reset_action_cost(&mut self, action: usize) {
        if action < self.action_costs.len() {
            self.action_costs[action] = 1.0;
        } else {
            error!("No action with index {action}");
        }
    }

    pub fn get_action_cost(&self, action: usize) -> f64 {
        if action < self.action_costs.len() {
            self.action_costs[action]
        } else {
            1.0
        }
    }

    pub fn idle(&self) -> bool {
        self.idle
    }

    pub fn go_idle(&mut self) {
        self.idle = true;
        self.idle_ticks_count = 0;
    }

    pub fn tick_idle(&mut self, config: &Config) {
        self.idle_ticks_count += 1;
        if self.idle_ticks_count >= config.agent.total_ticks_idle {
            self.idle = false;
            self.idle_ticks_count = 0;
        }
    }

    pub fn description(&self, goap: &Goap) -> String {
        let mut desc = String::new();
        desc.push_str("PLAN: ");
        if self.has_plan() {
            for (i, action) in self.plan.iter().enumerate() {
                if i != 0 {
                    desc.push_str(", ");
                }
                if let Some(description) = goap.get_description(self.action_set(), *action) {
                    desc.push_str(description.as_str());
                } else {
                    desc.push_str(format!("unknown action {action}").as_str());
                }
            }
        } else {
            desc.push_str("none");
        }
        desc
    }
}
