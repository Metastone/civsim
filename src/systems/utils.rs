use crate::algorithms::path_finding::WayPoint;
use crate::algorithms::path_finding::compute_path;
use crate::components::all::PlantComponent;
use crate::components::body_component::BodyComponent;
use crate::configuration::Config;
use crate::ecs::{Component, Ecs, EntityId};
use crate::shared_data::body_grid;
use std::any::TypeId;
use std::collections::HashSet;

// If an empty path is returned, it means that the target is already reached
pub fn find_closest_reachable<C>(
    ecs: &mut Ecs,
    config: &Config,
    entity: EntityId,
    body: &BodyComponent,
) -> Option<(f64, EntityId, BodyComponent, Vec<WayPoint>)>
where
    C: Component,
{
    for (target_entity, distance_squared) in
        body_grid::iter_closest(entity, body, config.path.max_search_distance)
    {
        if let Some(info) = ecs.get_entity_info(target_entity)
            && ecs.has_components(
                info.arch_index,
                &HashSet::from([to_ctype!(C), to_ctype!(BodyComponent)]),
            )
        {
            // If the target is a plant, check if it is eatable
            if let Some(target_plant) = ecs.component::<PlantComponent>(&info)
                && !target_plant.is_eatable()
            {
                continue;
            }

            let target_body = ecs.component::<BodyComponent>(&info).unwrap();

            // Check if the target is already reached
            if body.almost_collides(target_body, config.collision.contact_center_2_center_factor) {
                return Some((distance_squared, target_entity, *target_body, Vec::new()));
            }
            // If not, try to find a path to the target
            else if let Some((path, _)) =
                compute_path(config, entity, body, target_entity, target_body)
            {
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
