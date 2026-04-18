use crate::components::agent_component::{ActionResult, AgentComponent, Goap, WorldState};
use crate::configuration::Config;
use crate::ecs::{Ecs, EntityInfo, System, Update};
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
    entity_info: EntityInfo,
    has_plan: bool,
    goal: Option<usize>,
    goal_set: usize,
    action: Option<usize>,
    action_set: usize,
    world_state: WorldState,
}
impl AgentInfo {
    fn new(agent: &AgentComponent, entity_info: &EntityInfo) -> Self {
        AgentInfo {
            entity_info: *entity_info,
            has_plan: agent.has_plan(),
            goal: agent.goal(),
            goal_set: agent.goal_set(),
            action: agent.action(),
            action_set: agent.action_set(),
            world_state: agent.world_state().clone(),
        }
    }
}

impl System for AgentSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        // This is just stupid, a recuring problem, I have to copy all relevant info.
        // I can't work directly in the loop because that would required borrowing ECS twice
        // (to pass a reference to the GOAP)
        // TODO is there a better way ?
        self.agents.clear();
        self.agents = iter_components!(ecs, (), (AgentComponent))
            .map(|(agent, info)| AgentInfo::new(agent, &info))
            .collect();

        for agent in self.agents.iter_mut() {
            // Find a goal if necessary
            if agent.goal.is_none() {
                if let Some(goal) =
                    self.goap
                        .find_goal(&*ecs, config, &agent.entity_info, agent.goal_set)
                {
                    let agent_component = ecs
                        .component_mut::<AgentComponent>(&agent.entity_info)
                        .unwrap();
                    agent.goal = Some(goal);
                    agent_component.set_goal(goal);
                } else {
                    // If no goal was found, skip the agent
                    continue;
                }
            }

            // Compute a plan if necessary
            if !agent.has_plan {
                if let Some(plan) = self.goap.compute_plan(
                    &*ecs,
                    &mut updates,
                    config,
                    agent.goal.unwrap(),
                    agent.goal_set,
                    agent.action_set,
                ) {
                    let agent_component = ecs
                        .component_mut::<AgentComponent>(&agent.entity_info)
                        .unwrap();
                    agent.action = agent_component.action();
                    agent_component.set_plan(plan);
                } else {
                    // If no plan was found, skip the agent
                    continue;
                }
            }

            // Perform the current action, and if its completed advance to next action in the plan
            match self.goap.perform_action(
                &*ecs,
                &mut updates,
                &agent.entity_info,
                config,
                &mut agent.world_state,
                agent.action.unwrap(),
                agent.action_set,
            ) {
                ActionResult::Success => {
                    ecs.component_mut::<AgentComponent>(&agent.entity_info)
                        .unwrap()
                        .advance_to_next_action();
                }
                ActionResult::Failure => {
                    ecs.component_mut::<AgentComponent>(&agent.entity_info)
                        .unwrap()
                        .reset_plan();
                }
                ActionResult::OnGoing => {
                    ecs.component_mut::<AgentComponent>(&agent.entity_info)
                        .unwrap()
                        .set_world_state(&agent.world_state);
                }
            }
        }

        ecs.apply(updates);
    }
}
