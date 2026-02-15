use crate::components::all::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use std::any::TypeId;
use std::collections::HashMap;

pub struct EatCorpseSystem;
impl System for EatCorpseSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Make sure that a corpse is not eaten by more than one creature
        let mut corpse_to_creature: HashMap<EntityId, EntityInfo> = HashMap::new();
        for info in iter_entities!(ecs, EatingCorpseComponent) {
            if let Some(eating_corpse) = ecs.component::<EatingCorpseComponent>(&info) {
                corpse_to_creature.insert(eating_corpse.corpse_entity, info);
            }
            updates.push(Update::Delete {
                info,
                c_type: to_ctype!(EatingCorpseComponent),
            });
            updates.push(Update::Add {
                info,
                comp: Box::new(InactiveComponent::new()),
            });
        }

        // Increase energy of creatures that ate a corpse
        for info in corpse_to_creature.values() {
            if let Some(creature) = ecs.component_mut::<CreatureComponent>(info) {
                creature.energy += CORPSE_ENERGY;
                if creature.energy > MAX_ENERGY {
                    creature.energy = MAX_ENERGY;
                }
            }
        }

        // Remove eaten corpse entities
        for corpse_entity in corpse_to_creature.keys() {
            if let Some(info) = ecs.entity_info(*corpse_entity) {
                updates.push(Update::DeleteEntity(info));
            }
        }

        ecs.apply(updates);
    }
}
