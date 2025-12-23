use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use crate::systems::utils::{move_towards_waypoint, MoveResult};
use std::any::TypeId;
use std::collections::HashMap;

enum MoveToTargetResult {
    Moved,
    Stopped,
}

/* Move all entities towards their target while avoiding collisions.
 * Each entity follows a path composed of a series of waypoint (computed to avoid collisions).
 * If a collision occurs (i.e because other entities moved), a new path is computed.
 */
pub struct MoveToTargetSystem;
impl System for MoveToTargetSystem {
    fn run(&self, ecs: &mut Ecs) {
        let mut updates: Vec<Update> = Vec::new();

        // Get the positions (bodies) of all targets
        let mut target_bodies: HashMap<EntityId, Option<BodyComponent>> =
            iter_components!(ecs, (), (MoveToTargetComponent))
                .map(|(move_to_target, _)| (move_to_target.target_entity, None))
                .collect();
        for (entity, body) in target_bodies.iter_mut() {
            *body = ecs
                .get_component_from_entity::<BodyComponent>(*entity)
                .copied();
        }

        // Iterate over all "move to target" entities
        for (body, move_to_target, info) in
            iter_components!(ecs, (), (BodyComponent, MoveToTargetComponent))
        {
            if let MoveToTargetResult::Stopped =
                try_move(body, move_to_target, &info, &target_bodies)
            {
                // Go into motionless state
                updates.push(Update::Delete {
                    info,
                    c_type: to_ctype!(MoveToTargetComponent),
                });
                continue;
            }
        }

        ecs.apply(updates);
    }
}

fn try_move(
    body: &mut BodyComponent,
    move_to_target: &mut MoveToTargetComponent,
    info: &EntityInfo,
    target_bodies: &HashMap<EntityId, Option<BodyComponent>>,
) -> MoveToTargetResult {
    // Get the target position, if possible (the entity could have been deleted)
    let target_body;
    if let Some(b) = target_bodies[&move_to_target.target_entity] {
        target_body = b;
    } else {
        return MoveToTargetResult::Stopped;
    }

    // Recompute a new path if the target moved
    if target_body.get_x() != move_to_target.target_body.get_x()
        || target_body.get_y() != move_to_target.target_body.get_y()
    {
        move_to_target.target_body = target_body;
        if !move_to_target.compute_path() {
            return MoveToTargetResult::Stopped;
        }
    }

    let (waypoint_x, waypoint_y) = move_to_target.get_next_waypoint().unwrap();

    // Try to move the entity towards the next waypoint
    // TODO Distinguish reaching last waypoint VS reaching target
    match move_towards_waypoint(info, body, waypoint_x, waypoint_y, move_to_target.speed) {
        MoveResult::WaypointReached => {
            move_to_target.waypoint_reached();
            if move_to_target.is_last_waypoint() {
                return MoveToTargetResult::Stopped;
            }
        }
        MoveResult::Collision => {
            // Try to re-compute a new path
            // Will move next iteration
            if !move_to_target.compute_path() {
                return MoveToTargetResult::Stopped;
            }
        }
        _ => {}
    }
    MoveToTargetResult::Moved
}
