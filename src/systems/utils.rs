use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, Ecs, EntityId};
use std::any::TypeId;

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
