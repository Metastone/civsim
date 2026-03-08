use crate::constants::*;
use crate::ecs::{Component, EntityId};
use std::collections::VecDeque;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct PlantComponent {
    pub is_seed_initialized: bool,
    pub is_seed: bool,
    pub countdown_ticks_as_seed: usize,

    pub growth_per_tick: f64,
    pub size: f64,
    pub max_size: f64,

    pub nb_seeds: usize,
    pub max_nb_seeds: usize,

    pub count_ticks_to_seed: usize,
    pub ticks_per_seed: usize,
}
impl Component for PlantComponent {}
impl PlantComponent {
    pub fn new() -> Self {
        // Temporary values. The plant will be properly initialiazed taking into account the
        // humidity level, when the plant position is known (body component added to the ECS)
        Self {
            is_seed_initialized: false,
            is_seed: true,
            countdown_ticks_as_seed: PLANT_TICKS_AS_SEED,
            growth_per_tick: 1.0,
            size: SEED_SIZE,
            max_size: PLANT_MAX_SIZE,
            nb_seeds: 0,
            max_nb_seeds: PLANT_MAX_SEEDS,
            count_ticks_to_seed: 0,
            ticks_per_seed: PLANT_TICKS_PER_SEED,
        }
    }

    pub fn is_eatable(&self) -> bool {
        !self.is_seed
    }

    pub fn init_seed(&mut self, h: f64) {
        // Low humidity makes growing from seed to plant longer
        self.countdown_ticks_as_seed = (PLANT_TICKS_AS_SEED as f64 * (1.0 / h)) as usize;

        self.is_seed_initialized = true;
    }

    pub fn become_plant(&mut self, h: f64) {
        // humidity is in [0; 1]

        self.is_seed = false;

        // Low humidity makes growing from seed to plant longer
        self.countdown_ticks_as_seed = (PLANT_TICKS_AS_SEED as f64 * (1.0 / h)) as usize;

        self.size = PLANT_INITIAL_SIZE;

        let h_2 = h.powi(2);
        self.growth_per_tick = PLANT_SIZE_GROWTH_PER_TICK * h_2;
        self.max_size = PLANT_MAX_SIZE * h_2;

        // Minimum 1 seed to allow reproduction even in deserts
        self.max_nb_seeds = ((PLANT_MAX_SEEDS as f64 * h_2) as usize).max(1);

        // Low humidity makes generating new seeds longer
        self.ticks_per_seed = (PLANT_TICKS_PER_SEED as f64 * (1.0 / h)) as usize;
    }
}

#[derive(Clone)]
pub struct EatingPlantComponent {
    pub plant_entity: EntityId,
}
impl Component for EatingPlantComponent {}
impl EatingPlantComponent {
    pub fn new(plant_entity: EntityId) -> Self {
        Self { plant_entity }
    }
}

#[derive(Clone)]
pub struct EatingCorpseComponent {
    pub corpse_entity: EntityId,
}
impl Component for EatingCorpseComponent {}
impl EatingCorpseComponent {
    pub fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone)]
pub struct AttackingHerbivorousComponent {
    pub herbivorous_entity: EntityId,
}
impl Component for AttackingHerbivorousComponent {}
impl AttackingHerbivorousComponent {
    pub fn new(herbivorous_entity: EntityId) -> Self {
        Self { herbivorous_entity }
    }
}

#[derive(Clone)]
pub struct CorpseComponent;
impl Component for CorpseComponent {}
impl CorpseComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct HerbivorousComponent {
    // Queue of (number of seeds, countdown to excretion)
    pub seeds: VecDeque<(usize, usize)>,
}
impl Component for HerbivorousComponent {}
impl HerbivorousComponent {
    pub fn new() -> Self {
        Self {
            seeds: VecDeque::new(),
        }
    }
}

#[derive(Clone)]
pub struct CarnivorousComponent {}
impl Component for CarnivorousComponent {}
impl CarnivorousComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct InactiveComponent {}
impl Component for InactiveComponent {}
impl InactiveComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct ObstacleComponent {}
impl Component for ObstacleComponent {}
impl ObstacleComponent {
    pub fn new() -> Self {
        Self {}
    }
}
