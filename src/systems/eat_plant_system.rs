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
        let mut plant_to_creature: HashMap<EntityId, (f64, usize, EntityInfo)> = HashMap::new();
        for info in iter_entities!(ecs, EatingPlantComponent) {
            if let Some(eating_plant) = ecs.component::<EatingPlantComponent>(&info) && let Some(plant) = ecs.component_from_entity::<PlantComponent>(eating_plant.plant_entity) {
                plant_to_creature.insert(eating_plant.plant_entity, (plant.size, plant.nb_seeds, info));
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
        for (plant_size, plant_nb_seeds, info) in plant_to_creature.values() {
            if let Some(creature) = ecs.component_mut::<CreatureComponent>(info) {
                creature.energy += (*plant_size as f32) * PLANT_ENERGY_PER_SIZE_UNIT;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
            if let Some(herbivorous) = ecs.component_mut::<HerbivorousComponent>(info) {
                herbivorous.seeds.push_back((*plant_nb_seeds, HERBIVOROUS_TICKS_TO_DIGEST));
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
