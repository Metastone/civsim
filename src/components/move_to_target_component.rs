use crate::algorithms::a_star::{find_reverse_path, Graph};
use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, EntityId};

#[derive(Clone)]
pub struct WayPoint {
    x: f64,
    y: f64,
    reached: bool,
}

#[derive(Clone)]
pub struct MoveToTargetComponent {
    pub target_entity: EntityId,
    pub target_body: BodyComponent,
    path: Vec<WayPoint>,
    pub speed: f64,
}
impl Component for MoveToTargetComponent {}
impl MoveToTargetComponent {
    pub fn new(target_entity: EntityId, target_body: BodyComponent, speed: f64) -> Self {
        Self {
            target_entity,
            target_body,
            path: Vec::new(),
            speed,
        }
    }

    pub fn compute_path(&mut self, body: &BodyComponent) -> bool {
        // TODO Use PRM-like algorithm in the starting body grid cell to construct a graph (graph A)
        // of all safely reachable places around the starting point.
        // (Add creature size to the obstacles radius)
        // The goal is to be able to reach the center of one of the adjacent cells (if possible)
        //
        // Then, construct a graph (graph B) consisting of the centers of the cells of the body grid.
        // Only cells up, down, left, right are considered adjacent for the graph construction.
        // The cells considered traversable, taken into account for the graph, are the ones that
        // are empty AND not colliding with anything in the 8 adjacent cells.
        // Restrict it to an area containing the the creature and the target, not the whole map
        // (for performances)
        //
        // TODO Then, construct a graph (graph C) around the target using a PRM-like algorithm, same
        // logic as before.
        //
        // Connect graphs A to the centers of the 4 cells adjacent to the starting cell in graph B.
        // Connect graphs C to the centers of the 4 cells adjacent to the destination cell in graph B.
        //
        // Finally, use A* algorithm to find a path to the target, on the full graph made of graphs A, B, C.
        // If a path is found, it passes through the center of empty cells only, except around the
        // start and the destination.

        let mut graph = Graph::new();
        let (start_node, end_node) = graph.add_body_grid_nodes(
            body.get_x(),
            body.get_y(),
            self.target_body.get_x(),
            self.target_body.get_y(),
        );

        if let Some(reverse_path) = find_reverse_path(&graph, start_node, end_node) {
            self.path.clear();
            for n in reverse_path.iter().rev() {
                self.path.push(WayPoint {
                    x: n.get_x(),
                    y: n.get_y(),
                    reached: false,
                });
            }
            return true;
        }

        false
    }

    pub fn waypoint_reached(&mut self) {
        for waypoint in self.path.iter_mut() {
            if !waypoint.reached {
                waypoint.reached = true;
                break;
            }
        }
    }

    pub fn is_last_waypoint(&self) -> bool {
        match self.path.last() {
            Some(waypoint) => waypoint.reached,
            None => true,
        }
    }

    pub fn get_next_waypoint(&self) -> Option<(f64, f64)> {
        for waypoint in self.path.iter() {
            if !waypoint.reached {
                return Some((waypoint.x, waypoint.y));
            }
        }
        None
    }
}
