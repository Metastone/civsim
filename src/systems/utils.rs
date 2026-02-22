use crate::algorithms::path_finding::compute_path;
use crate::algorithms::path_finding::WayPoint;
use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, Ecs, EntityId};
use crate::shared_data::body_grid;
use crate::MAX_SEARCH_DISTANCE;
use std::any::TypeId;
use std::collections::HashSet;

pub fn find_closest_reachable<C>(
    ecs: &mut Ecs,
    entity: EntityId,
    body: &BodyComponent,
) -> Option<(f64, EntityId, BodyComponent, Vec<WayPoint>)>
where
    C: Component,
{
    for (target_entity, distance_squared) in
        body_grid::iter_closest(entity, body, MAX_SEARCH_DISTANCE)
    {
        if let Some(info) = ecs.get_entity_info(target_entity) && ecs.has_components(
                info.arch_index,
                &HashSet::from([to_ctype!(C), to_ctype!(BodyComponent)]),
            ) {
                let target_body = ecs.component::<BodyComponent>(&info).unwrap();
                if let Some((path, _)) = compute_path(entity, body, target_entity, target_body) {
                    return Some((distance_squared, target_entity, *target_body, path));
                }
            }
    }
    None
}

#[allow(dead_code)]
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
        let distance_squared = (t_body.x() - body.x()).powi(2) + (t_body.y() - body.y()).powi(2);
        if distance_squared < closest_distance_squared {
            closest_distance_squared = distance_squared;
            opt_entity = Some((closest_distance_squared, t_info.entity, *t_body));
        }
    }
    opt_entity
}
