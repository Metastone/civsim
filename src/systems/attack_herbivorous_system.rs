use crate::components::*;
use crate::constants::*;
use crate::ecs::{Ecs, EntityId, System, Update};
use std::any::TypeId;
use std::collections::HashSet;

pub struct AttackHerbivorousSystem;
impl System for AttackHerbivorousSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Make sure that a herbivorous is not attacked by more than one creature
        let mut attacked_herbivorous: HashSet<EntityId> = HashSet::new();

        for (attacking_herbivorous, info) in iter_components!(ecs, AttackingHerbivorousComponent) {
            attacked_herbivorous.insert(attacking_herbivorous.herbivorous_entity);
            updates.push(Update::Delete {
                info,
                c_type: to_ctype!(AttackingHerbivorousComponent),
            });
            updates.push(Update::Add {
                info,
                comp: Box::new(InactiveComponent::new()),
            });
        }

        ecs.apply(updates);

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
    }
}
