use crate::algorithms::path_finding::{find_reverse_path, Graph, Node};
use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, EntityId};

#[derive(Clone)]
pub struct WayPoint {
    x: f64,
    y: f64,
    reached: bool,
}

impl WayPoint {
    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_reached(&self) -> bool {
        self.reached
    }
}

#[derive(Clone)]
pub struct MoveToTargetComponent {
    pub target_entity: EntityId,
    pub target_body: BodyComponent,
    path: Vec<WayPoint>,
    graph: Graph,
    speed: f64,
    on_target_reached: Box<dyn Component>,
    on_failure: Box<dyn Component>,
}

impl Component for MoveToTargetComponent {}

impl MoveToTargetComponent {
    pub fn new(
        target_entity: EntityId,
        target_body: BodyComponent,
        speed: f64,
        on_target_reached: Box<dyn Component>,
        on_failure: Box<dyn Component>,
    ) -> Self {
        Self {
            target_entity,
            target_body,
            path: Vec::new(),
            graph: Graph::new(),
            speed,
            on_target_reached,
            on_failure,
        }
    }

    pub fn compute_path(&mut self, entity: EntityId, body: &BodyComponent) -> bool {
        // Construct a graph (graph B) consisting of the centers of the cells of the body grid.
        // Only cells up, down, left, right are considered adjacent for the graph construction.
        // The cells considered traversable, taken into account for the graph, are the ones that
        // are empty AND not colliding with anything in the 8 adjacent cells.
        // Restrict it to an area containing the the creature and the target, not the whole map
        // (for performances)
        //
        // Use PRM-like algorithm to construct a graph (graph A) of all safely reachable places
        // around the current creature position. Connect nodes that are near each other (if the
        // edge does not collide, taking the creature size into account).
        // Connect this graph to the center of the 8 cells adjacent to the creature position
        // (based on the body collision grid).
        //
        // Construct a graph (graph C) around the target using a PRM-like algorithm, same
        // logic as before.
        //
        // Finally, use A* algorithm to find a path to the target, on the full graph made of graphs A, B, C.

        self.graph.clear();

        self.graph.add_body_grid_nodes(
            entity,
            body.get_x(),
            body.get_y(),
            self.target_body.get_x(),
            self.target_body.get_y(),
        );

        if !self
            .graph
            .add_prm_nodes(entity, body, body.get_x(), body.get_y())
        {
            return false;
        }

        if !self.graph.add_prm_nodes(
            entity,
            body,
            self.target_body.get_x(),
            self.target_body.get_y(),
        ) {
            return false;
        }

        if !self.graph.connect_nodes(entity, body) {
            return false;
        }

        let start_node = Node::new(body.get_x(), body.get_y());
        let end_node = Node::new(self.target_body.get_x(), self.target_body.get_y());

        if let Some(reverse_path) = find_reverse_path(&self.graph, start_node, end_node) {
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

    pub fn get_speed(&self) -> f64 {
        self.speed
    }

    pub fn get_on_target_reached(&self) -> Box<dyn Component> {
        self.on_target_reached.clone()
    }

    pub fn get_on_failure(&self) -> Box<dyn Component> {
        self.on_failure.clone()
    }

    // For display
    pub fn get_path(&self) -> &Vec<WayPoint> {
        &self.path
    }

    // For display
    pub fn get_graph(&self) -> &Graph {
        &self.graph
    }
}
