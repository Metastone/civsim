use crate::components::body_component::BodyComponent;
use crate::constants::CONTACT_CENTER_2_CENTER_FACTOR;
use crate::ecs::{ComponentType, Ecs, EntityId, EntityInfo};
use std::any::TypeId;

pub fn move_towards_position(
    ecs: &mut Ecs,
    info: &EntityInfo,
    target_body: &BodyComponent,
    speed: f64,
) -> bool {
    if let Some(body) = ecs.get_component_mut::<BodyComponent>(info) {
        if body.almost_collides(target_body, CONTACT_CENTER_2_CENTER_FACTOR) {
            // Target reached
            return true;
        } else {
            // Get closer to the target
            let vec_to_target = (
                target_body.get_x() - body.get_x(),
                target_body.get_y() - body.get_y(),
            );
            let norm = (vec_to_target.0.powi(2) + vec_to_target.1.powi(2)).sqrt();
            let offset_x = vec_to_target.0 / norm * speed;
            let offset_y = vec_to_target.1 / norm * speed;
            body.try_translate(offset_x, offset_y);
        }
    }
    false
}

pub fn find_closest(
    ecs: &Ecs,
    body: &BodyComponent,
    c_type: ComponentType,
) -> Option<(f64, EntityId)> {
    let mut closest_distance_squared = f64::MAX;
    let mut opt_entity = None;
    for info in ecs.iter_entities_with(&[c_type, to_ctype!(BodyComponent)]) {
        if let Some(o_body) = ecs.get_component::<BodyComponent>(&info) {
            let distance_squared =
                (o_body.get_x() - body.get_x()).powi(2) + (o_body.get_y() - body.get_y()).powi(2);
            if distance_squared < closest_distance_squared {
                closest_distance_squared = distance_squared;
                opt_entity = Some(info.entity);
            }
        }
    }
    opt_entity.map(|entity| (closest_distance_squared, entity))
}
