use crate::components::body_component::BodyComponent;
use crate::ecs::{ComponentType, Ecs, EntityId, EntityInfo};
use std::any::TypeId;

pub fn move_towards_position(
    ecs: &mut Ecs,
    info: &EntityInfo,
    target_position: &BodyComponent,
    target_size: f64,
    entity_size: f64,
    speed: f64,
) -> bool {
    if let Some(position) = ecs.get_component_mut::<BodyComponent>(info) {
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

pub fn find_closest(
    ecs: &Ecs,
    position: &BodyComponent,
    c_type: ComponentType,
) -> Option<(f64, EntityId)> {
    let mut closest_distance_squared = f64::MAX;
    let mut opt_entity = None;
    for info in ecs.iter_entities_with(&[c_type, to_ctype!(BodyComponent)]) {
        if let Some(o_position) = ecs.get_component::<BodyComponent>(&info) {
            let distance_squared =
                (o_position.x - position.x).powi(2) + (o_position.y - position.y).powi(2);
            if distance_squared < closest_distance_squared {
                closest_distance_squared = distance_squared;
                opt_entity = Some(info.entity);
            }
        }
    }
    opt_entity.map(|entity| (closest_distance_squared, entity))
}
