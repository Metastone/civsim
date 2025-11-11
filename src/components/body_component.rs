use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::cell::RefCell;

use crate::constants::*;
use crate::ecs::Component;
use crate::ecs::EntityId;
use crate::shared_data::body_grid::BODY_GRID;

#[derive(Clone, Copy)]
pub struct BodyComponent {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Component for BodyComponent {
    fn on_delete(&self, _entity: EntityId) {
        BODY_GRID.with_borrow_mut(|grid| grid.delete(self));
    }
}

impl BodyComponent {
    pub fn new_rand_pos_with_collision(w: f64, h: f64) -> Self {
        thread_local! {
            static RNG: RefCell<SmallRng> = if RNG_SEED != 0 {
                RefCell::new(SmallRng::seed_from_u64(RNG_SEED))
            } else {
                RefCell::new(SmallRng::from_rng(&mut rand::rng()))
            }
        };

        // Generate a random position that does not collides with any already existing body
        let body = loop {
            let x = RNG.with_borrow_mut(|rng| {
                rng.random_range((SCREEN_WIDTH as f64 / -2.0)..(SCREEN_WIDTH as f64 / 2.0))
            });
            let y = RNG.with_borrow_mut(|rng| {
                rng.random_range((SCREEN_HEIGHT as f64 / -2.0)..(SCREEN_HEIGHT as f64 / 2.0))
            });

            let b = Self::from(x, y, w, h);
            if !BODY_GRID.with_borrow_mut(|grid| grid.collides_in_surronding_cells(&b, &b)) {
                break b;
            }
        };
        body.add_to_grid();
        body
    }

    pub fn new_with_collision(x: f64, y: f64, w: f64, h: f64) -> Self {
        let body = Self::from(x, y, w, h);
        body.add_to_grid();
        body
    }

    pub fn from(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
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

    fn add_to_grid(&self) {
        BODY_GRID.with_borrow_mut(|grid| grid.add(self));
    }

    pub fn try_translate(&mut self, offset_x: f64, offset_y: f64) {
        if BODY_GRID.with_borrow_mut(|grid| grid.try_translate(self, offset_x, offset_y)) {
            self.x += offset_x;
            self.y += offset_y;
        }
    }

    pub fn collides(&self, other: &BodyComponent) -> bool {
        (self.x - self.w / 2.0) < (other.x + other.w / 2.0)
            && (self.x + self.w / 2.0) > (other.x - other.w / 2.0)
            && (self.y - self.h / 2.0) < (other.y + other.h / 2.0)
            && (self.y + self.h / 2.0) > (other.y - other.h / 2.0)
    }

    pub fn almost_collides(&self, other: &BodyComponent, factor: f64) -> bool {
        (self.x - (self.w / 2.0) * factor) < (other.x + (other.w / 2.0) * factor)
            && (self.x + (self.w / 2.0) * factor) > (other.x - (other.w / 2.0) * factor)
            && (self.y - (self.h / 2.0) * factor) < (other.y + (other.h / 2.0) * factor)
            && (self.y + (self.h / 2.0) * factor) > (other.y - (other.h / 2.0) * factor)
    }
}
