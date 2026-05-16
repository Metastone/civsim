use crate::components::agent_component::AgentComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, EntityInfo, System};
use crate::goap::{ActionResult, Goap, WorldState};
use std::any::TypeId;

pub struct AgentSystem {
    goap: Goap,
    agents: Vec<AgentInfo>,
}

impl AgentSystem {
    pub fn new(goap: Goap) -> Self {
        Self {
            goap,
            agents: Vec::new(),
        }
    }
}

struct AgentInfo {
    info: EntityInfo,
    has_plan: bool,
    goal: Option<usize>,
    goal_set: usize,
    action: Option<usize>,
    action_set: usize,
    world_state: WorldState,
    idle: bool,
}
impl AgentInfo {
    fn new(agent: &AgentComponent, info: &EntityInfo) -> Self {
        AgentInfo {
            info: *info,
            has_plan: agent.has_plan(),
            goal: agent.goal,
            goal_set: agent.goal_set(),
            action: agent.action(),
            action_set: agent.action_set(),
            world_state: agent.world_state.clone(),
            idle: agent.idle(),
        }
    }
}

impl System for AgentSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        // TODO This is just stupid, a recuring problem, I have to copy all relevant info.
        // I can't work directly in the loop because that would required borrowing ECS twice
        // (to pass a reference to the GOAP)
        // Is there a better way to do this ?
        self.agents.clear();
        self.agents = iter_components!(ecs, (), (AgentComponent))
            .map(|(agent, info)| AgentInfo::new(agent, &info))
            .collect();

        for agent in self.agents.iter_mut() {
            // Check that the agent still exists (it may have been deleted by the other agents
            // performing their actions)
            let info_opt = ecs.get_entity_info(agent.info.entity);
            if let Some(info) = info_opt
                && ecs.has_component(info.arch_index, &to_ctype!(AgentComponent))
            {
                agent.info = info;
            } else {
                continue;
            }

            // If the agent is in idle state, do nothing
            if agent.idle {
                let agent_component = ecs.component_mut::<AgentComponent>(&agent.info).unwrap();
                agent_component.tick_idle(config);
                continue;
            }

            // Find a goal if necessary
            if agent.goal.is_none() {
                if let Some(goal) = self.goap.find_goal(&*ecs, &agent.info, agent.goal_set) {
                    let agent_component = ecs.component_mut::<AgentComponent>(&agent.info).unwrap();
                    agent.goal = Some(goal);
                    // TODO goal should be reset after plan execution
                    agent_component.goal = Some(goal);
                } else {
                    // If no goal was found, skip the agent
                    continue;
                }
            }

            // Compute a plan if necessary
            let agent_component = ecs.component_mut::<AgentComponent>(&agent.info).unwrap();
            if !agent.has_plan {
                if let Some(plan) = self.goap.compute_plan(
                    &*agent_component,
                    &agent.world_state,
                    agent.goal.unwrap(),
                    agent.goal_set,
                    agent.action_set,
                ) && !plan.is_empty()
                {
                    agent.action = Some(plan[0]);
                    agent.has_plan = true;
                    agent_component.set_plan(plan);
                } else {
                    // If no plan was found, skip the agent
                    continue;
                }
            }

            // Perform the current action, and if its completed advance to next action in the plan
            let result = self.goap.perform_action(
                ecs,
                &agent.info,
                config,
                &mut agent.world_state,
                agent.action.unwrap(),
                agent.action_set,
            );

            // After performing the action, check that the agent still exists
            let info_opt = ecs.get_entity_info(agent.info.entity);
            if let Some(info) = info_opt
                && ecs.has_component(info.arch_index, &to_ctype!(AgentComponent))
            {
                agent.info = info;
            } else {
                continue;
            }

            let agent_component = ecs.component_mut::<AgentComponent>(&agent.info).unwrap();
            match result {
                ActionResult::Success => {
                    agent_component.advance_to_next_action();
                    agent_component.reset_action_cost(agent.action.unwrap());
                }
                ActionResult::Failure => {
                    agent_component.reset_plan();
                    agent_component.increase_action_cost(agent.action.unwrap());
                }
                ActionResult::OnGoing => {
                    agent_component.world_state = agent.world_state.clone();
                }
            }
        }
    }
}
