use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::cell::RefCell;
use std::cmp::max_by;
use std::cmp::Ordering;

use crate::constants::*;
use crate::ecs::Component;

thread_local! {static MAX_BODY_SIZE: RefCell<f64> = RefCell::new(0.0)}
pub fn get_max_half_body_size() -> f64 {
    MAX_BODY_SIZE.with_borrow(|m| *m / 2.0)
}

#[derive(Clone, Copy)]
pub struct BodyComponent {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
impl Component for BodyComponent {}
impl BodyComponent {
    pub fn new_rand_pos(w: f64, h: f64) -> Self {
        thread_local! {
            static RNG: RefCell<SmallRng> = if RNG_SEED != 0 {
                RefCell::new(SmallRng::seed_from_u64(RNG_SEED))
            } else {
                RefCell::new(SmallRng::from_rng(&mut rand::rng()))
            }
        };

        let x = RNG.with_borrow_mut(|rng| {
            rng.random_range((SCREEN_WIDTH as f64 / -2.0)..(SCREEN_WIDTH as f64 / 2.0))
        });
        let y = RNG.with_borrow_mut(|rng| {
            rng.random_range((SCREEN_HEIGHT as f64 / -2.0)..(SCREEN_HEIGHT as f64 / 2.0))
        });
        Self::from(x, y, w, h)
    }

    pub fn from(x: f64, y: f64, w: f64, h: f64) -> Self {
        let max_w_h = max_by(w, h, |a, b| {
            if a > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        MAX_BODY_SIZE.with_borrow_mut(|m| {
            if max_w_h > *m {
                *m = max_w_h;
            }
        });
        Self { x, y, w, h }
    }

    fn collides(&self, other: &BodyComponent) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    pub fn if_colliding_step_to_the_side(&mut self, other: &BodyComponent) {
        if self.collides(other) {
            let right = (self.x + self.w - other.x, other.x - self.w, true);
            let left = (other.x + other.w - self.x, other.x + other.w, true);
            let down = (self.y + self.h - other.y, other.y - self.h, false);
            let up = (other.y + other.h - self.y, other.y + other.h, false);
            let corrections = [right, left, down, up];
            let (_distance, fixed_coord, is_x_coord) = corrections
                .iter()
                .min_by(|(distance_a, _, _), (distance_b, _, _)| {
                    if distance_a > distance_b {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                })
                .unwrap();

            if *is_x_coord {
                self.x = *fixed_coord;
            } else {
                self.y = *fixed_coord;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::body_component::BodyComponent;

    #[test]
    fn step_ouf_of_right_collision() {
        let mut body = BodyComponent::from(0.0, 0.0, 1.0, 1.2);
        let other = BodyComponent::from(0.9, 0.0, 0.7, 0.6);
        body.if_colliding_step_to_the_side(&other);
        assert_eq!(body.x, 0.9 - 1.0);
        assert_eq!(body.y, 0.0);
    }

    #[test]
    fn step_ouf_of_left_collision() {
        let mut body = BodyComponent::from(0.0, 0.0, 1.0, 1.2);
        let other = BodyComponent::from(-0.5, 0.0, 0.7, 0.6);
        body.if_colliding_step_to_the_side(&other);
        assert_eq!(body.x, -0.5 + 0.7);
        assert_eq!(body.y, 0.0);
    }

    #[test]
    fn step_ouf_of_down_collision() {
        let mut body = BodyComponent::from(0.0, 0.0, 1.0, 1.2);
        let other = BodyComponent::from(0.0, 0.9, 0.7, 0.8);
        body.if_colliding_step_to_the_side(&other);
        assert_eq!(body.x, 0.0);
        assert_eq!(body.y, 0.9 - 1.2);
    }

    #[test]
    fn step_ouf_of_up_collision() {
        let mut body = BodyComponent::from(0.0, 0.0, 1.0, 1.2);
        let other = BodyComponent::from(0.0, -0.5, 0.7, 0.8);
        body.if_colliding_step_to_the_side(&other);
        assert_eq!(body.x, 0.0);
        assert_eq!(body.y, -0.5 + 0.8);
    }
}
