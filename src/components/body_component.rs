use crate::algorithms::rng;
use crate::constants::*;
use crate::ecs::Component;
use crate::ecs::EntityId;
use crate::shared_data::body_grid;

#[derive(Clone, Copy)]
pub struct BodyComponent {
    // Position of the center of the body
    x: f64,
    y: f64,

    // Size of the body on x and y axis respectively
    w: f64,
    h: f64,

    // Cannot collide with anything, not added to body grid
    is_traversable: bool,

    init_with_random_pos: bool,
}

impl Component for BodyComponent {
    fn on_create(&mut self, entity: EntityId) {
        // Generate a random position that does not collides with any already existing body
        if self.init_with_random_pos {
            loop {
                let x = rng::random_range(SCREEN_WIDTH as f64 / -2.0, SCREEN_WIDTH as f64 / 2.0);
                let y = rng::random_range(SCREEN_HEIGHT as f64 / -2.0, SCREEN_HEIGHT as f64 / 2.0);
                self.x = x;
                self.y = y;
                if self.is_traversable || !body_grid::collides(entity, self) {
                    break;
                }
            }
        }

        if !self.is_traversable {
            body_grid::add(entity, self);
        }
    }

    fn on_delete(&mut self, entity: EntityId) {
        body_grid::delete(entity, self);
    }
}

impl BodyComponent {
    pub fn new_rand_pos_not_traversable(w: f64, h: f64) -> Self {
        Self {
            x: 0.0, // Temp x,y -> will be updated in on_create
            y: 0.0,
            w,
            h,
            is_traversable: false,
            init_with_random_pos: true,
        }
    }

    pub fn new_rand_pos_traversable(w: f64, h: f64) -> Self {
        Self {
            x: 0.0, // Temp x,y -> will be updated in on_create
            y: 0.0,
            w,
            h,
            is_traversable: true,
            init_with_random_pos: true,
        }
    }

    pub fn new_not_traversable(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            w,
            h,
            is_traversable: false,
            init_with_random_pos: false,
        }
    }

    pub fn new_traversable(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            w,
            h,
            is_traversable: true,
            init_with_random_pos: false,
        }
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }

    pub fn get_w(&self) -> f64 {
        self.w
    }

    pub fn get_h(&self) -> f64 {
        self.h
    }

    pub fn try_translate(&mut self, entity: EntityId, offset_x: f64, offset_y: f64) -> bool {
        if body_grid::try_translate(entity, self, offset_x, offset_y) {
            self.x += offset_x;
            self.y += offset_y;
            return true;
        }
        false
    }

    pub fn collides(&self, other: &BodyComponent) -> bool {
        (self.x - self.w / 2.0) < (other.x + other.w / 2.0)
            && (self.x + self.w / 2.0) > (other.x - other.w / 2.0)
            && (self.y - self.h / 2.0) < (other.y + other.h / 2.0)
            && (self.y + self.h / 2.0) > (other.y - other.h / 2.0)
    }

    // TODO remove if I really don't use it
    #[allow(unused)]
    pub fn almost_collides(&self, other: &BodyComponent, factor: f64) -> bool {
        (self.x - (self.w / 2.0) * factor) < (other.x + (other.w / 2.0) * factor)
            && (self.x + (self.w / 2.0) * factor) > (other.x - (other.w / 2.0) * factor)
            && (self.y - (self.h / 2.0) * factor) < (other.y + (other.h / 2.0) * factor)
            && (self.y + (self.h / 2.0) * factor) > (other.y - (other.h / 2.0) * factor)
    }

    pub fn almost_at_position(&self, x: f64, y: f64, margin: f64) -> bool {
        (self.x - x).abs() < margin && (self.y - y).abs() < margin
    }
}
