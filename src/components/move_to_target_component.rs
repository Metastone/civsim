use crate::algorithms::path_finding::{compute_path, Graph, WayPoint};
use crate::components::body_component::BodyComponent;
use crate::ecs::{Component, EntityId};

#[derive(Clone)]
pub struct MoveToTargetComponent {
    target_entity: EntityId,
    target_body: BodyComponent,
    path_to_target: Vec<WayPoint>,
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
        path_to_target: Vec<WayPoint>,
        speed: f64,
        on_target_reached: Box<dyn Component>,
        on_failure: Box<dyn Component>,
    ) -> Self {
        Self {
            target_entity,
            target_body,
            path_to_target,
            graph: Graph::new(),
            speed,
            on_target_reached,
            on_failure,
        }
    }

    // TODO maybe refactor this and call compute_path in caller only ? or maybe not
    pub fn compute_path(&mut self, entity: EntityId, body: &BodyComponent) -> bool {
        self.graph.clear();
        self.path_to_target.clear();

        if let Some((path, graph)) =
            compute_path(entity, body, self.target_entity, &self.target_body)
        {
            self.path_to_target = path;
            self.graph = graph;
            return true;
        }

        false
    }

    pub fn waypoint_reached(&mut self) {
        for waypoint in self.path_to_target.iter_mut() {
            if !waypoint.reached() {
                waypoint.set_reached();
                break;
            }
        }
    }

    pub fn next_waypoint(&self) -> Option<(f64, f64)> {
        for waypoint in self.path_to_target.iter() {
            if !waypoint.reached() {
                return Some((waypoint.x(), waypoint.y()));
            }
        }
        None
    }

    pub fn is_last_waypoint_reached(&self) -> bool {
        self.path_to_target.last().is_none_or(|wp| wp.reached())
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
        &self.path_to_target
    }

    // For display
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}
