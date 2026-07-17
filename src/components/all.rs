use crate::configuration::{Config, CreatureConfig};
use crate::ecs::Component;
use crate::shared_data::direction::Direction;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct CreatureComponent {
    energy: f32,
    max_energy: f32,
    health: f32,
    max_health: f32,
}
impl Component for CreatureComponent {}
impl CreatureComponent {
    pub fn new(config: &CreatureConfig) -> Self {
        Self {
            energy: config.start_energy,
            max_energy: config.max_energy,
            health: config.max_health,
            max_health: config.max_health,
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }

    pub fn health(&self) -> f32 {
        self.health
    }

    pub fn increment_energy(&mut self, e: f32) {
        self.energy += e;
        self.energy.clamp(0.0, self.max_energy);
    }

    pub fn increment_health(&mut self, h: f32) {
        self.health += h;
        self.health.clamp(0.0, self.max_health);
    }
}

#[derive(Clone, Copy)]
pub enum PlantKind {
    Bush,
    Tree,
}

// TODO make all plant fields private ?

#[derive(Clone)]
pub struct SeedComponent {
    pub is_seed_initialized: bool,
    pub countdown_ticks_as_seed: usize,
    pub plant_kind: PlantKind,
}

impl Component for SeedComponent {}
impl SeedComponent {
    pub fn new(plant_kind: PlantKind, config: &Config) -> Self {
        // Temporary values. The seed will be properly initialized taking into account the
        // humidity level, when the seed position is known (body component added to the ECS)
        Self {
            is_seed_initialized: false,
            countdown_ticks_as_seed: config.plant.ticks_as_seed,
            plant_kind,
        }
    }

    pub fn init_seed(&mut self, config: &Config, humidity: f64 /* in [0; 1] */) {
        self.countdown_ticks_as_seed =
            (config.plant.ticks_as_seed as f64 * (1.0 / humidity)) as usize;
        self.is_seed_initialized = true;
    }
}

#[derive(Clone)]
pub struct PlantGrowthComponent {
    pub growth_per_tick: f64,
    pub width: f64,
    pub max_width: f64,
}

impl Component for PlantGrowthComponent {}
impl PlantGrowthComponent {
    pub fn new(config: &Config, width: f64, humidity: f64 /* in [0; 1] */) -> Self {
        let h_2 = humidity.powi(2);
        Self {
            growth_per_tick: config.plant.size_growth_per_tick * h_2,
            width,
            max_width: config.plant.max_size * h_2,
        }
    }
}

#[derive(Clone)]
pub struct Fruit {
    pub nb_seeds: usize,
    pub max_nb_seeds: usize,
    pub count_ticks_to_seed: usize,
    pub ticks_per_seed: usize,
}

impl Component for Fruit {}
impl Fruit {
    pub fn new(config: &Config, humidity: f64 /* in [0; 1] */) -> Self {
        let h_2 = humidity.powi(2);

        // Minimum 1 fruit to allow reproduction even in deserts
        Self {
            nb_seeds: 0,
            max_nb_seeds: ((config.plant.max_fruit_seeds as f64 * h_2) as usize).max(1),
            count_ticks_to_seed: 0,
            ticks_per_seed: (config.plant.ticks_per_fruit_seed as f64 * (1.0 / humidity)) as usize,
        }
    }
}

#[derive(Clone)]
pub struct PlantWithFruitComponent {
    humidity: f64,
    humidity_2: f64,
    pub fruits: VecDeque<Fruit>,
    pub max_nb_fruits: usize,
    pub count_ticks_to_fruit: usize,
    pub ticks_per_fruit: usize,
}

impl Component for PlantWithFruitComponent {}
impl PlantWithFruitComponent {
    pub fn new(config: &Config, humidity: f64 /* in [0; 1] */) -> Self {
        let h_2 = humidity.powi(2);

        // Minimum 1 fruit to allow reproduction even in deserts
        Self {
            humidity,
            humidity_2: h_2,
            fruits: VecDeque::new(),
            max_nb_fruits: ((config.plant.max_fruits as f64 * h_2) as usize).max(1),
            count_ticks_to_fruit: 0,
            ticks_per_fruit: (config.plant.ticks_per_fruit as f64 * (1.0 / humidity)) as usize,
        }
    }

    pub fn add_new_fruit(&mut self, config: &Config) {
        if self.fruits.len() < self.max_nb_fruits {
            self.fruits.push_back(Fruit::new(config, self.humidity));
        }
    }

    pub fn grow_fruits(&mut self) {
        for fruit in self.fruits.iter_mut() {
            if fruit.count_ticks_to_seed >= fruit.ticks_per_seed {
                fruit.count_ticks_to_seed = 0;
                fruit.nb_seeds = (fruit.nb_seeds + 1).min(fruit.max_nb_seeds);
            }
            fruit.count_ticks_to_seed += 1;
        }
    }

    pub fn detach_one_fruit(&mut self) -> Option<Fruit> {
        self.fruits.pop_front()
    }

    pub fn has_fruits(&self) -> bool {
        !self.fruits.is_empty()
    }
}

// TODO implement decay system for fruits
#[derive(Clone)]
pub struct FruitComponent {
    pub nb_seeds: usize,
}
impl Component for FruitComponent {}
impl FruitComponent {
    pub fn new(nb_seeds: usize) -> Self {
        FruitComponent { nb_seeds }
    }
}

#[derive(Clone)]
pub struct PlantWithStolonComponent {
    pub stolon_growth_per_tick: f64,
    pub stolon_length: f64,
    pub stolon_max_length: f64,
    pub stolon_direction: Direction,
}

impl Component for PlantWithStolonComponent {}
impl PlantWithStolonComponent {
    pub fn new(config: &Config, humidity: f64 /* in [0; 1] */) -> Self {
        let h_2 = humidity.powi(2);
        Self {
            stolon_growth_per_tick: config.plant.stolon_length_growth_per_tick * h_2,
            stolon_length: 0.0,
            stolon_max_length: config.plant.max_stolon_length * humidity,
            stolon_direction: Direction::random(),
        }
    }
}

#[derive(Clone)]
pub struct BushComponent {}
impl Component for BushComponent {}

#[derive(Clone)]
pub struct TreeComponent {}
impl Component for TreeComponent {}

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
    // Queue of (number of seeds, plant_kind, countdown to excretion)
    pub seeds: VecDeque<(usize, PlantKind, usize)>,
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
pub struct ObstacleComponent {}
impl Component for ObstacleComponent {}
impl ObstacleComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct MoveToTargetResultComponent {
    pub success: bool,
}
impl Component for MoveToTargetResultComponent {}
impl MoveToTargetResultComponent {
    pub fn new(success: bool) -> Self {
        Self { success }
    }
}
