use crate::components::*;
use crate::ecs::{ArchetypeManager, ComponentType};

pub fn move_towards_position(
    manager: &mut ArchetypeManager,
    arch_index: usize,
    entity_index: usize,
    target_position: &PositionComponent,
    target_size: f64,
    entity_size: f64,
    speed: f64,
) -> bool {
    if let Some(position) = manager.get_component_mut::<PositionComponent>(
        arch_index,
        entity_index,
        &ComponentType::Position,
    ) {
        let vec_to_target = (
            target_position.x - position.x,
            target_position.y - position.y,
        );
        let norm = (vec_to_target.0.powi(2) + vec_to_target.1.powi(2)).sqrt();
        if norm < (target_size / 2.0 + entity_size / 2.0) {
            // Target reached
            return true;
        } else {
            // Get closer to the target
            position.x += vec_to_target.0 / norm * speed;
            position.y += vec_to_target.1 / norm * speed;
        }
    }
    false
}
