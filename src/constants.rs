pub const SCREEN_WIDTH: u32 = 2560;
pub const SCREEN_HEIGHT: u32 = 1440;

pub const HUNGER_RATE: f32 = 0.2;
pub const EXHAUSTION_RATE: f32 = 1.0;

pub const FOOD_ENERGY: f32 = 20.0;
pub const CORPSE_ENERGY: f32 = 20.0;
pub const CARNIVOROUS_ATTACK: f32 = 20.0;
pub const REPROD_ENERGY_THRESHOLD: f32 = 90.0;
pub const REPROD_ENERGY_COST: f32 = 50.0;
pub const START_ENERGY: f32 = 40.0;
pub const MAX_ENERGY: f32 = 100.0;
pub const MAX_HEALTH: f32 = 100.0;

pub const CREATURE_PIXEL_SIZE: u32 = 60;
pub const FOOD_PIXEL_SIZE: u32 = 20;

pub const BAR_WIDTH: u32 = 60;
pub const BAR_HEIGHT: u32 = 10;

pub const HERBIVOROUS_COLOR: &[u8] = &[0xff, 0x99, 0x11, 0xff];
pub const CARNIVOROUS_COLOR: &[u8] = &[0xff, 0x22, 0x11, 0xff];
pub const FOOD_COLOR: &[u8] = &[0x22, 0xbb, 0x11, 0xff];
pub const ENERGY_COLOR: &[u8] = &[0x11, 0xff, 0x88, 0xff];
pub const HEALTH_COLOR: &[u8] = &[0xff, 0x11, 0x11, 0xff];
pub const CORPSE_COLOR: &[u8] = &[0x44, 0x11, 0x11, 0xff];

pub const HERBIVOROUS_SPEED: f64 = 3.0; // Pixels per iteration
pub const CARNIVOROUS_SPEED: f64 = 4.0; // Pixels per iteration

pub const MS_PER_ITERATION: u64 = 16;

pub const HERBIVOROUS_NB: usize = 10;
pub const CARNIVOROUS_NB: usize = 1;
pub const FOOD_NB: usize = 100;
pub const CORPSE_NB: usize = 0;
pub const NEW_FOOD_PER_TICK: usize = 1;
