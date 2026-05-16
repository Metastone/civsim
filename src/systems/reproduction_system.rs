use crate::components::agent_component::AgentComponent;
use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::configuration::Config;
use crate::ecs::{Component, Ecs, RESERVED_ENTITY_ID, System, Update};
use crate::shared_data::body_grid;
use std::any::TypeId;

pub struct ReproductionSystem {
    herbivorous_goal_set: usize,
    herbivorous_action_set: usize,
    herbivorous_action_set_len: usize,
    carnivorous_goal_set: usize,
    carnivorous_action_set: usize,
    carnivorous_action_set_len: usize,
}
impl ReproductionSystem {
    pub fn new(
        herbivorous_goal_set: usize,
        herbivorous_action_set: usize,
        herbivorous_action_set_len: usize,
        carnivorous_goal_set: usize,
        carnivorous_action_set: usize,
        carnivorous_action_set_len: usize,
    ) -> Self {
        Self {
            herbivorous_goal_set,
            herbivorous_action_set,
            herbivorous_action_set_len,
            carnivorous_goal_set,
            carnivorous_action_set,
            carnivorous_action_set_len,
        }
    }
}
impl System for ReproductionSystem {
    fn run(&mut self, ecs: &mut Ecs, config: &Config) {
        let mut updates: Vec<Update> = Vec::new();

        // Find creatures that can reproduce
        for info in iter_entities!(ecs, CreatureComponent, BodyComponent) {
            // Check if the creature has enough energy to reproduce
            {
                let creature = ecs.component_mut::<CreatureComponent>(&info).unwrap();
                if creature.energy < config.creature.reprod_energy_threshold {
                    continue;
                }
            }

            let body = ecs.component::<BodyComponent>(&info).unwrap();
            let new_body = BodyComponent::new_not_traversable(
                body.x() + config.creature.size + config.creature.reprod_x_offset,
                body.y(),
                config.creature.size,
                config.creature.size,
            );

            // Reproduce only if there is a free space for the new creature
            if body_grid::collides(RESERVED_ENTITY_ID, &new_body) {
                continue;
            }

            let mut comps: Vec<Box<dyn Component>> = vec![
                Box::new(CreatureComponent::new(&config.creature)),
                Box::new(new_body),
            ];

            // Check if herbivorous or carnivorous
            let is_herbivorous =
                ecs.has_component(info.arch_index, &to_ctype!(HerbivorousComponent));
            if is_herbivorous {
                comps.push(Box::new(HerbivorousComponent::new()));
                comps.push(Box::new(AgentComponent::new(
                    self.herbivorous_goal_set,
                    self.herbivorous_action_set,
                    self.herbivorous_action_set_len,
                )));
            } else {
                comps.push(Box::new(CarnivorousComponent::new()));
                comps.push(Box::new(AgentComponent::new(
                    self.carnivorous_goal_set,
                    self.carnivorous_action_set,
                    self.carnivorous_action_set_len,
                )));
            }

            // Create a new creature
            updates.push(Update::Create(comps));

            // Apply reproduction energy cost to parent creature
            {
                let creature = ecs.component_mut::<CreatureComponent>(&info).unwrap();
                creature.energy -= config.creature.reprod_energy_cost;
            }
        }

        ecs.apply(updates);
    }
}
