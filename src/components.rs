use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::constants::*;
use crate::ecs::{Component, EntityId};
use std::cell::RefCell;

#[derive(Clone, Copy)]
pub struct CreatureComponent {
    pub energy: f32,
    pub health: f32,
}
impl Component for CreatureComponent {}
impl CreatureComponent {
    pub fn new() -> Self {
        Self {
            energy: START_ENERGY,
            health: MAX_HEALTH,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PositionComponent {
    pub x: f64,
    pub y: f64,
}
impl Component for PositionComponent {}
impl PositionComponent {
    pub fn new() -> Self {
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
        Self { x, y }
    }

    pub fn from(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct FoodComponent {}
impl Component for FoodComponent {}
impl FoodComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct EatingFoodComponent {
    pub food_entity: EntityId,
}
impl Component for EatingFoodComponent {}
impl EatingFoodComponent {
    pub fn new(food_entity: EntityId) -> Self {
        Self { food_entity }
    }
}

#[derive(Clone, Copy)]
pub struct EatingCorpseComponent {
    pub corpse_entity: EntityId,
}
impl Component for EatingCorpseComponent {}
impl EatingCorpseComponent {
    pub fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone, Copy)]
pub struct AttackingHerbivorousComponent {
    pub herbivorous_entity: EntityId,
}
impl Component for AttackingHerbivorousComponent {}
impl AttackingHerbivorousComponent {
    pub fn new(herbivorous_entity: EntityId) -> Self {
        Self { herbivorous_entity }
    }
}

#[derive(Clone, Copy)]
pub struct CorpseComponent;
impl Component for CorpseComponent {}
impl CorpseComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct HerbivorousComponent {}
impl Component for HerbivorousComponent {}
impl HerbivorousComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct CarnivorousComponent {}
impl Component for CarnivorousComponent {}
impl CarnivorousComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct MoveToCorpseComponent {
    pub corpse_entity: EntityId,
}
impl Component for MoveToCorpseComponent {}
impl MoveToCorpseComponent {
    pub fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone, Copy)]
pub struct MoveToFoodComponent {
    pub food_entity: EntityId,
}
impl Component for MoveToFoodComponent {}
impl MoveToFoodComponent {
    pub fn new(food_entity: EntityId) -> Self {
        Self { food_entity }
    }
}

#[derive(Clone, Copy)]
pub struct MoveToHerbivorousComponent {
    pub herbivorous_entity: EntityId,
}
impl Component for MoveToHerbivorousComponent {}
impl MoveToHerbivorousComponent {
    pub fn new(herbivorous_entity: EntityId) -> Self {
        Self { herbivorous_entity }
    }
}

#[derive(Clone, Copy)]
pub struct InactiveComponent {}
impl Component for InactiveComponent {}
impl InactiveComponent {
    pub fn new() -> Self {
        Self {}
    }
}
