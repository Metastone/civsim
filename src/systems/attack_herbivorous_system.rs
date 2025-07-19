use crate::components::*;
use crate::constants::*;
use crate::ecs::EntityInfo;
use crate::ecs::{Ecs, EntityId, System};
use std::any::TypeId;
use std::collections::HashSet;

pub struct AttackHerbivorousSystem;
impl System for AttackHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        // Make sure that a herbivorous is not attacked by more than one creature
        let mut attacked_herbivorous: HashSet<EntityId> = HashSet::new();
        let mut creatures_trying_to_attack: Vec<EntityInfo> = Vec::new();
        for (attacking_herbivorous, info) in iter_components!(ecs, AttackingHerbivorousComponent) {
            attacked_herbivorous.insert(attacking_herbivorous.herbivorous_entity);
            creatures_trying_to_attack.push(info);
        }

        // Decrease life of attacked herbivorous entities
        for h_entity in attacked_herbivorous.iter() {
            if let Some(creature) =
                ecs.get_component_mut_from_entity::<CreatureComponent>(*h_entity)
            {
                creature.health -= CARNIVOROUS_ATTACK;
                if creature.health < 0.0 {
                    creature.health = 0.0;
                }
            }
        }

        // Go into inactive state
        for info in creatures_trying_to_attack.iter() {
            ecs.remove_component(info.entity, to_ctype!(AttackingHerbivorousComponent));
            ecs.add_component(info.entity, &InactiveComponent::new());
        }
    }
}
