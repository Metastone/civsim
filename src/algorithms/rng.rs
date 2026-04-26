use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::cell::RefCell;

use crate::configuration::Config;

// Initialize RNG
thread_local! {
    static RNG: RefCell<Option<SmallRng>> = RefCell::new(None);
}

pub fn init(config: &Config) {
    RNG.with_borrow_mut(|rng| {
        *rng = Some(if config.rng_seed != 0 {
            SmallRng::seed_from_u64(config.rng_seed)
        } else {
            SmallRng::from_rng(&mut rand::rng())
        })
    });
}

pub fn random_range(lower_bound: f64, upper_bound: f64) -> f64 {
    RNG.with_borrow_mut(|rng| rng.as_mut().unwrap().random_range(lower_bound..upper_bound))
}
