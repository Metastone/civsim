use crate::constants::*;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;
use std::cell::RefCell;

// Initialize RNG
thread_local! {
    static RNG: RefCell<SmallRng> = if RNG_SEED != 0 {
        RefCell::new(SmallRng::seed_from_u64(RNG_SEED))
    } else {
        RefCell::new(SmallRng::from_rng(&mut rand::rng()))
    }
}

pub fn random_range(lower_bound: f64, upper_bound: f64) -> f64 {
    RNG.with_borrow_mut(|rng| rng.random_range(lower_bound..upper_bound))
}
