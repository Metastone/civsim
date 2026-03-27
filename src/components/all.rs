use crate::configuration::{Config, CreatureConfig};
use crate::ecs::{Component, EntityId};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct CreatureComponent {
    pub energy: f32,
    pub health: f32,
}
impl Component for CreatureComponent {}
impl CreatureComponent {
    pub fn new(config: &CreatureConfig) -> Self {
        Self {
            energy: config.start_energy,
            health: config.max_health,
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
    pub fn new(config: &Config) -> Self {
        // Temporary values. The plant will be properly initialiazed taking into account the
        // humidity level, when the plant position is known (body component added to the ECS)
        Self {
            is_seed_initialized: false,
            is_seed: true,
            countdown_ticks_as_seed: config.plant.ticks_as_seed,
            growth_per_tick: 1.0,
            size: config.seed.size,
            max_size: config.plant.max_size,
            nb_seeds: 0,
            max_nb_seeds: config.plant.max_seeds,
            count_ticks_to_seed: 0,
            ticks_per_seed: config.plant.ticks_per_seed,
        }
    }

    pub fn is_eatable(&self) -> bool {
        !self.is_seed
    }

    pub fn init_seed(&mut self, config: &Config, h: f64) {
        // Low humidity makes growing from seed to plant longer
        self.countdown_ticks_as_seed = (config.plant.ticks_as_seed as f64 * (1.0 / h)) as usize;

        self.is_seed_initialized = true;
    }

    pub fn become_plant(&mut self, config: &Config, h: f64) {
        // humidity is in [0; 1]

        self.is_seed = false;

        // Low humidity makes growing from seed to plant longer
        self.countdown_ticks_as_seed = (config.plant.ticks_as_seed as f64 * (1.0 / h)) as usize;

        self.size = config.plant.initial_size;

        let h_2 = h.powi(2);
        self.growth_per_tick = config.plant.size_growth_per_tick * h_2;
        self.max_size = config.plant.max_size * h_2;

        // Minimum 1 seed to allow reproduction even in deserts
        self.max_nb_seeds = ((config.plant.max_seeds as f64 * h_2) as usize).max(1);

        // Low humidity makes generating new seeds longer
        self.ticks_per_seed = (config.plant.ticks_per_seed as f64 * (1.0 / h)) as usize;
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
pub struct InactiveComponent {
    pub idle: bool,
    pub idle_ticks_count: usize,
}
impl Component for InactiveComponent {}
impl InactiveComponent {
    pub fn new() -> Self {
        Self {
            idle: false,
            idle_ticks_count: 0,
        }
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
