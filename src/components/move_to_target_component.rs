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

    pub fn compute_path(&mut self) -> bool {
        self.path = vec![WayPoint {
            x: self.target_body.get_x(),
            y: self.target_body.get_y(),
            reached: false,
        }];
        true
    }

    pub fn waypoint_reached(&mut self) {
        for waypoint in self.path.iter_mut() {
            if !waypoint.reached {
                waypoint.reached = true;
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
