use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::constants::CONTACT_CENTER_2_CENTER_FACTOR;
use crate::ecs::{Ecs, EntityId, EntityInfo, System, Update};
use std::any::TypeId;
use std::collections::HashMap;

enum MoveToTargetResult {
    Moved,
    Stopped,
    Reached,
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
                .map(|(move_to_target, _)| (move_to_target.target_entity(), None))
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
            match try_move(body, move_to_target, &info, &target_bodies) {
                MoveToTargetResult::Stopped => {
                    // Go into motionless state
                    updates.push(Update::Delete {
                        info,
                        c_type: to_ctype!(MoveToTargetComponent),
                    });
                    updates.push(Update::Add {
                        info,
                        comp: move_to_target.on_failure().clone_box(),
                    });
                }
                MoveToTargetResult::Reached => {
                    // Go into motionless state
                    updates.push(Update::Delete {
                        info,
                        c_type: to_ctype!(MoveToTargetComponent),
                    });
                    updates.push(Update::Add {
                        info,
                        comp: move_to_target.on_target_reached().clone_box(),
                    });
                }
                _ => {}
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
    if let Some(b) = target_bodies[&move_to_target.target_entity()] {
        target_body = b;
    } else {
        return MoveToTargetResult::Stopped;
    }
    *move_to_target.target_body_mut() = target_body;

    /* Idea for performance :
     *
     * If on a previous run of this system for the same creature entity, no path was found,
     * (so Stopped -> Component MoveToTarget deleted)
     * AND in this new run, the target is the same as before AND has not moved
     *
     * Then its probable that the path resolution will still fail this time
     * (not certain because maybe some obstacles moved).
     * Maybe I can do something to try avoid infinite loop of useless trying of path computation ?
     *
     * Not sure if this case will be frequent and / or an issue.
     */

    // Compute a new path if necessary
    if move_to_target.next_waypoint().is_none() {
        if !move_to_target.compute_path(info.entity, body) {
            return MoveToTargetResult::Stopped;
        }
    }

    let (waypoint_x, waypoint_y) = move_to_target.next_waypoint().unwrap();

    // Check if the target is reached (no need to go though all waypoints if it's already reached)
    if body.almost_collides(move_to_target.target_body(), CONTACT_CENTER_2_CENTER_FACTOR) {
        move_to_target.waypoint_reached();
        return MoveToTargetResult::Reached;
    }
    // Check if the next waypoint is reached
    else if body.almost_at_position(waypoint_x, waypoint_y, move_to_target.speed()) {
        move_to_target.waypoint_reached();
        if move_to_target.is_last_waypoint_reached() {
            return MoveToTargetResult::Stopped;
        }
    }
    // Try to move closer to the next waypoint
    else {
        let vec_to_target = (waypoint_x - body.get_x(), waypoint_y - body.get_y());
        let norm = (vec_to_target.0.powi(2) + vec_to_target.1.powi(2)).sqrt();
        let offset_x = vec_to_target.0 / norm * move_to_target.speed();
        let offset_y = vec_to_target.1 / norm * move_to_target.speed();
        if body.try_translate(
            info.entity,
            move_to_target.target_entity(),
            offset_x,
            offset_y,
        ) {
            return MoveToTargetResult::Moved;
        } else {
            // Move failed: try to re-compute a new path
            if !move_to_target.compute_path(info.entity, body) {
                return MoveToTargetResult::Stopped;
            }
        }
    }
    MoveToTargetResult::Moved
}
