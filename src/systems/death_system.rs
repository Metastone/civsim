use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Ecs, System, Update};
use std::any::TypeId;

pub struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        for info in iter_entities_with!(ecs, CreatureComponent, BodyComponent) {
            // Check if the creature should die
            if let Some(creature) = ecs.get_component::<CreatureComponent>(&info) {
                if creature.health <= 0.0 {
                    // Create a corpse
                    if let Some(body) = ecs.get_component::<BodyComponent>(&info) {
                        // TODO bug fix new corpses not taken into account for collision ? same for
                        // food logically
                        updates.push(Update::Create(vec![
                            Box::new(CorpseComponent),
                            Box::new(BodyComponent::new_with_collision(
                                body.get_x(),
                                body.get_y(),
                                body.get_w(),
                                body.get_h(),
                            )),
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
