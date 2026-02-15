use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, Ecs, EntityId, EntityInfo};
use std::any::TypeId;

pub enum MoveResult {
    Moved,
    Collision,
    WaypointReached,
}

pub fn move_towards_waypoint(
    info: &EntityInfo,
    body: &mut BodyComponent,
    waypoint_x: f64,
    waypoint_y: f64,
    speed: f64,
) -> MoveResult {
    if body.almost_at_position(waypoint_x, waypoint_y, speed) {
        MoveResult::WaypointReached
    } else {
        // Get closer to the target
        let vec_to_target = (waypoint_x - body.get_x(), waypoint_y - body.get_y());
        let norm = (vec_to_target.0.powi(2) + vec_to_target.1.powi(2)).sqrt();
        let offset_x = vec_to_target.0 / norm * speed;
        let offset_y = vec_to_target.1 / norm * speed;
        if body.try_translate(info.entity, offset_x, offset_y) {
            MoveResult::Moved
        } else {
            MoveResult::Collision
        }
    }
}

pub fn find_closest<C>(
    ecs: &mut Ecs,
    body: &BodyComponent,
) -> Option<(f64, EntityId, BodyComponent)>
where
    C: Component,
{
    let mut closest_distance_squared = f64::MAX;
    let mut opt_entity = None;
    for (t_body, t_info) in iter_components!(ecs, (C), (BodyComponent)) {
        let distance_squared =
            (t_body.get_x() - body.get_x()).powi(2) + (t_body.get_y() - body.get_y()).powi(2);
        if distance_squared < closest_distance_squared {
            closest_distance_squared = distance_squared;
            opt_entity = Some((closest_distance_squared, t_info.entity, *t_body));
        }
    }
    opt_entity
}
