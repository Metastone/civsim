use crate::constants::*;
use crate::ecs::{Component, EntityId};

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
pub struct FoodComponent {
    pub growth_per_tick: f64,
    pub size: f64,
    pub max_size: f64,
}
impl Component for FoodComponent {}
impl FoodComponent {
    pub fn new() -> Self {
        Self {
            growth_per_tick: 1.0,
            size: FOOD_INITIAL_SIZE,
            max_size: FOOD_MAX_SIZE,
        }
    }

    pub fn init_growth_factor(&mut self, h: f64) {
        // humidity is in [0; 1]
        self.growth_per_tick = FOOD_SIZE_GROWTH_PER_TICK * h.powi(2);
        self.max_size *= h.powi(2);
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
pub struct InactiveComponent {}
impl Component for InactiveComponent {}
impl InactiveComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct ObstacleComponent {}
impl Component for ObstacleComponent {}
impl ObstacleComponent {
    pub fn new() -> Self {
        Self {}
    }
}
