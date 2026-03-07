use crate::components::all::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use std::any::TypeId;
use std::collections::HashMap;

pub struct EatPlantSystem;
impl System for EatPlantSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Make sure that a plant is not eaten by more than one creature
        let mut plant_to_creature: HashMap<EntityId, EntityInfo> = HashMap::new();
        for info in iter_entities!(ecs, EatingPlantComponent) {
            if let Some(eating_plant) = ecs.component::<EatingPlantComponent>(&info) {
                plant_to_creature.insert(eating_plant.plant_entity, info);
            }
            updates.push(Update::Delete {
                info,
                c_type: to_ctype!(EatingPlantComponent),
            });
            updates.push(Update::Add {
                info,
                comp: Box::new(InactiveComponent::new()),
            });
        }

        // Increase energy of creatures that ate a plant
        for info in plant_to_creature.values() {
            if let Some(creature) = ecs.component_mut::<CreatureComponent>(info) {
                creature.energy += PLANT_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten plant entities
        for plant_entity in plant_to_creature.keys() {
            if let Some(info) = ecs.get_entity_info(*plant_entity) {
                updates.push(Update::DeleteEntity(info));
            }
        }

        ecs.apply(updates);
    }
}
