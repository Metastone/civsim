use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System};
use std::any::TypeId;
use std::collections::HashMap;

pub struct AttackHerbivorousSystem;
impl System for AttackHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Make sure that a herbivorous is not attacked by more than one creature
        let mut herbivorous_to_creature: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creatures_trying_to_attack: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in iter_entities!(ecs, AttackingHerbivorousComponent)
        {
            if let Some(attacking_herbivorous) =
                ecs.get_component::<AttackingHerbivorousComponent>(arch_index, entity_index)
            {
                herbivorous_to_creature.insert(attacking_herbivorous.herbivorous_entity, entity);
            }
            creatures_trying_to_attack.push(entity);
        }

        // Decrease life of attacked herbivorous entities
        for herbivorous_entity in herbivorous_to_creature.keys() {
            // Make sure that the herbivorous entity exists
            let (arch_index, entity_index): (usize, usize);
            if let Some(idx) = ecs.get_entity_indexes(*herbivorous_entity) {
                arch_index = idx.0;
                entity_index = idx.1;
            } else {
                continue;
            }

            // Decrease the herbivorous entity life
            if let Some(creature) =
                ecs.get_component_mut::<CreatureComponent>(arch_index, entity_index)
            {
                creature.health -= CARNIVOROUS_ATTACK;
                if creature.health < 0.0 {
                    creature.health = 0.0;
                }
            }
        }

        // Go into inactive state
        for entity in creatures_trying_to_attack.iter() {
            ecs.remove_component(*entity, to_ctype!(AttackingHerbivorousComponent));
            ecs.add_component(*entity, &InactiveComponent::new());
        }
    }
}
