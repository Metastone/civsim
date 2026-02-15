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
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn reached(&self) -> bool {
        self.reached
    }
}

#[derive(Clone)]
pub struct MoveToTargetComponent {
    target_entity: EntityId,
    target_body: BodyComponent,
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
        // Construct a graph consisting of the centers of the cells of the body grid.
        // Only cells up, down, left, right are considered adjacent for the graph construction.
        // The cells considered traversable, taken into account for the graph, are the ones that
        // are empty AND not colliding with anything in the 8 adjacent cells.
        // Restrict it to an area containing the the creature and the target, not the whole map
        // (for performances)
        //
        // Use PRM-like algorithm to add to the graph nodes corresponding to non-colliding places
        // around the current creature position, and around the target.
        //
        // Connect nodes that are near each other (if the edge does not collide, taking the
        // creature size into account).
        //
        // Finally, use A* algorithm on this constructed graph to find a path to the target.

        self.graph.clear();
        self.path.clear();

        self.graph.add_body_grid_nodes(
            entity,
            body.get_x(),
            body.get_y(),
            self.target_entity,
            self.target_body.get_x(),
            self.target_body.get_y(),
        );

        if !self
            .graph
            .add_prm_nodes(entity, self.target_entity, body, body.get_x(), body.get_y())
        {
            return false;
        }

        if !self.graph.add_prm_nodes(
            entity,
            self.target_entity,
            body,
            self.target_body.get_x(),
            self.target_body.get_y(),
        ) {
            return false;
        }

        if !self.graph.connect_nodes(entity, self.target_entity, body) {
            return false;
        }

        let start_node = Node::new(body.get_x(), body.get_y());
        let end_node = Node::new(self.target_body.get_x(), self.target_body.get_y());

        if let Some(reverse_path) = find_reverse_path(&self.graph, start_node, end_node) {
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

    pub fn next_waypoint(&self) -> Option<(f64, f64)> {
        for waypoint in self.path.iter() {
            if !waypoint.reached {
                return Some((waypoint.x, waypoint.y));
            }
        }
        None
    }

    pub fn is_last_waypoint_reached(&self) -> bool {
        self.path.last().is_none_or(|wp| wp.reached())
    }

    pub fn target_entity(&self) -> EntityId {
        self.target_entity
    }

    pub fn target_body(&self) -> &BodyComponent {
        &self.target_body
    }

    pub fn target_body_mut(&mut self) -> &mut BodyComponent {
        &mut self.target_body
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn on_target_reached(&self) -> &dyn Component {
        &*self.on_target_reached
    }

    pub fn on_failure(&self) -> &dyn Component {
        &*self.on_failure
    }

    // For display
    pub fn path(&self) -> &Vec<WayPoint> {
        &self.path
    }

    // For display
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}
