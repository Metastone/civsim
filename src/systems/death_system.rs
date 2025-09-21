use crate::components::*;
use crate::ecs::{Ecs, System, Update};
use std::any::TypeId;

pub struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();
        let mut to_remove = Vec::new();

        for info in iter_entities_with!(ecs, CreatureComponent, PositionComponent) {
            // Check if the creature should die
            if let Some(creature) = ecs.get_component::<CreatureComponent>(&info) {
                if creature.health <= 0.0 {
                    to_remove.push(info.entity);
                } else {
                    continue;
                }
            }

            // Create a corpse
            if let Some(position) = ecs.get_component::<PositionComponent>(&info) {
                updates.push(Update::Create(vec![
                    Box::new(CorpseComponent),
                    Box::new(*position),
                ]));
            }
        }

        // Delete dead creature entities
        to_remove
            .iter()
            .for_each(|entity| ecs.remove_entity(*entity));

        ecs.apply(updates);
    }
}
