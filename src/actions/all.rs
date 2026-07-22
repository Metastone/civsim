use std::any::type_name;

use ordered_float::OrderedFloat;

use crate::{
    components::{
        agent_component::AgentComponent,
        all::{CreatureComponent, HerbivorousComponent, PlantKind, PlantWithFruitComponent},
    },
    configuration::Config,
    ecs::{Component, Ecs, EntityInfo, RESERVED_ENTITY_ID, Update},
    goap::{Action, ActionResult, Condition, Effect, Modifier, Operator, Symbol, Value},
};

pub fn get_comp_or_error<'e, A, C>(ecs: &'e mut Ecs, info: &EntityInfo) -> Result<&'e mut C, String>
where
    A: Action,
    C: Component,
{
    ecs.component_mut::<C>(info).ok_or_else(|| {
        format!(
            "Action {} is missing the required component {}",
            type_name::<A>(),
            type_name::<C>()
        )
    })
}

pub struct EatFruitAction {
    preconditions: [Condition; 1],
    effects: [Effect; 2],
}
impl EatFruitAction {
    pub fn new(config: &Config) -> Self {
        Self {
            preconditions: [Condition::new(
                Symbol::IsNearPlantWithFruit,
                Operator::Equal,
                Value::Bool(true),
            )],
            effects: [
                Effect::new(
                    Symbol::Energy,
                    Modifier::Increment,
                    Value::F32(OrderedFloat(config.plant.fruit_energy)),
                ),
                Effect::new(
                    Symbol::IsNearPlantWithFruit,
                    Modifier::SetValue,
                    Value::Bool(false),
                ),
            ],
        }
    }
}

impl Action for EatFruitAction {
    fn preconditions(&self) -> &[Condition] {
        &self.preconditions
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
        // Get the target plant entity ID
        let agent = get_comp_or_error::<EatFruitAction, AgentComponent>(ecs, info)?;
        let plant_entity = agent.target_entity;
        agent.target_entity = RESERVED_ENTITY_ID;

        // Get the target plant component
        if let Some(p_info) = ecs.get_entity_info(plant_entity) {
            let plant = get_comp_or_error::<EatFruitAction, PlantWithFruitComponent>(ecs, &p_info)?;

            // Eat one of the fruits of the plant and its seeds
            if let Some(fruit) = plant.detach_one_fruit().clone() {
                let herbivorous = ecs.component_mut::<HerbivorousComponent>(info).unwrap();
                herbivorous.seeds.push_back((
                    fruit.nb_seeds,
                    PlantKind::Bush,
                    config.creature.herbivorous_ticks_to_digest,
                ));

                // Increase energy
                let creature = get_comp_or_error::<EatFruitAction, CreatureComponent>(ecs, info)?;
                creature.increment_energy(config.plant.fruit_energy);

                return Ok(ActionResult::Success);
            }
        }

        // Plant or fruit not found, maybe it was already eaten by someone else
        Ok(ActionResult::Failure)
    }

    fn description(&self) -> String {
        String::from("eat fruit")
    }
}

pub struct EatCorpseAction {
    preconditions: [Condition; 1],
    effects: [Effect; 2],
}
impl EatCorpseAction {
    pub fn new(config: &Config) -> Self {
        let estimated_gain = config.creature.corpse_energy;
        Self {
            preconditions: [Condition::new(
                Symbol::IsNearCorpse,
                Operator::Equal,
                Value::Bool(true),
            )],
            effects: [
                Effect::new(
                    Symbol::Energy,
                    Modifier::Increment,
                    Value::F32(OrderedFloat(estimated_gain)),
                ),
                Effect::new(Symbol::IsNearCorpse, Modifier::SetValue, Value::Bool(false)),
            ],
        }
    }
}
impl Action for EatCorpseAction {
    fn preconditions(&self) -> &[Condition] {
        &self.preconditions
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
        // Get the target corpse entity ID
        let agent = get_comp_or_error::<EatCorpseAction, AgentComponent>(ecs, info)?;
        let corpse_entity = agent.target_entity;
        agent.target_entity = RESERVED_ENTITY_ID;

        // Check if the target corpse still exists
        if let Some(c_info) = ecs.get_entity_info(corpse_entity) {
            // Increase energy
            let creature = get_comp_or_error::<EatCorpseAction, CreatureComponent>(ecs, info)?;
            creature.increment_energy(config.creature.corpse_energy);

            // Delete the corpse
            ecs.apply(vec![Update::DeleteEntity(c_info)]);

            Ok(ActionResult::Success)
        } else {
            // Corpse not found, maybe it was already eaten by someone else
            Ok(ActionResult::Failure)
        }
    }

    fn description(&self) -> String {
        String::from("eat corpse")
    }
}

pub struct EatHerbivorousAction {
    preconditions: [Condition; 1],
    effects: [Effect; 2],
}
impl EatHerbivorousAction {
    pub fn new(config: &Config) -> Self {
        let estimated_gain = config.creature.corpse_energy;
        Self {
            preconditions: [Condition::new(
                Symbol::IsNearHerbivorous,
                Operator::Equal,
                Value::Bool(true),
            )],
            effects: [
                Effect::new(
                    Symbol::Energy,
                    Modifier::Increment,
                    Value::F32(OrderedFloat(estimated_gain)),
                ),
                Effect::new(
                    Symbol::IsNearHerbivorous,
                    Modifier::SetValue,
                    Value::Bool(false),
                ),
            ],
        }
    }
}
impl Action for EatHerbivorousAction {
    fn preconditions(&self) -> &[Condition] {
        &self.preconditions
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
        // Get the target herbivorous entity ID
        let agent = get_comp_or_error::<EatHerbivorousAction, AgentComponent>(ecs, info)?;
        let herbivorous_entity = agent.target_entity;
        agent.target_entity = RESERVED_ENTITY_ID;

        // Check if the target herbivorous still exists
        if let Some(h_info) = ecs.get_entity_info(herbivorous_entity) {
            // Increase energy
            let creature = get_comp_or_error::<EatCorpseAction, CreatureComponent>(ecs, info)?;
            creature.increment_health(config.creature.corpse_energy);

            // Delete the herbivorous
            ecs.apply(vec![Update::DeleteEntity(h_info)]);

            Ok(ActionResult::Success)
        } else {
            // Herbivorous not found, maybe it was already eaten by someone else
            Ok(ActionResult::Failure)
        }
    }

    fn description(&self) -> String {
        String::from("eat herbivorous")
    }
}
