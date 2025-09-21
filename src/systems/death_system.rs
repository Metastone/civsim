use crate::components::*;
use crate::ecs::{Ecs, System, Update};
use std::any::TypeId;

pub struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        for info in iter_entities_with!(ecs, CreatureComponent, PositionComponent) {
            // Check if the creature should die
            if let Some(creature) = ecs.get_component::<CreatureComponent>(&info) {
                if creature.health <= 0.0 {
                    // Create a corpse
                    if let Some(position) = ecs.get_component::<PositionComponent>(&info) {
                        updates.push(Update::Create(vec![
                            Box::new(CorpseComponent),
                            Box::new(*position),
                        ]));
                    }

                    // Remove the entity
                    updates.push(Update::DeleteEntity(info));
                }
            }
        }

        ecs.apply(updates);
    }
}
