pub const RNG_SEED: u64 = 1; // 0 means not seeded

pub const SCREEN_WIDTH: u32 = 2560;
pub const SCREEN_HEIGHT: u32 = 1440;

pub const INITIAL_ZOOM: f64 = 0.5;
pub const ZOOM_FACTOR: f64 = 1.2;
pub const MOVE_CAMERA_OFFSET: isize = 100;

pub const BODY_DOMAIN_INITIAL_WIDTH: f64 = 5000.0;
pub const BODY_DOMAIN_INITIAL_HEIGHT: f64 = 3000.0;

pub const HUNGER_RATE: f32 = 0.1;
pub const EXHAUSTION_RATE: f32 = 1.0;

pub const PLANT_ENERGY: f32 = 10.0;
pub const CORPSE_ENERGY: f32 = 20.0;
pub const CARNIVOROUS_ATTACK: f32 = 20.0;
pub const REPROD_ENERGY_THRESHOLD: f32 = 90.0;
pub const REPROD_ENERGY_COST: f32 = 50.0;
pub const START_ENERGY: f32 = 40.0;
pub const MAX_ENERGY: f32 = 100.0;
pub const MAX_HEALTH: f32 = 100.0;

pub const CREATURE_SIZE: f64 = 30.0;
pub const OBSTACLE_SIZE: f64 = 85.0;
pub const CELL_SIZE_FACTOR: f64 = 3.0;
pub const CONTACT_CENTER_2_CENTER_FACTOR: f64 = 1.01;
pub const MAX_SEARCH_DISTANCE: f64 = 500.0;

pub const PLANT_INITIAL_SIZE: f64 = 1.0;
pub const PLANT_MAX_SIZE: f64 = 100.0;
pub const PLANT_SIZE_GROWTH_PER_TICK: f64 = 0.1;
pub const REPROD_X_OFFSET: f64 = 10.0;

pub const BAR_WIDTH: f64 = 30.0;
pub const BAR_HEIGHT: f64 = 5.0;
pub const GRID_LINE_WIDENESS: f64 = 5.0;
pub const GRAPH_EDGE_THICKNESS: f64 = 5.0;

pub const HERBIVOROUS_COLOR: &[u8] = &[0xff, 0x99, 0x11, 0xff];
pub const CARNIVOROUS_COLOR: &[u8] = &[0xff, 0x22, 0x11, 0xff];
pub const PLANT_COLOR: &[u8] = &[0x22, 0xbb, 0x11, 0xff];
pub const ENERGY_COLOR: &[u8] = &[0x11, 0xff, 0x88, 0xff];
pub const HEALTH_COLOR: &[u8] = &[0xff, 0x11, 0x11, 0xff];
pub const CORPSE_COLOR: &[u8] = &[0x44, 0x11, 0x11, 0xff];
pub const OBSTACLE_COLOR: &[u8] = &[0x77, 0x33, 0x33, 0xff];
pub const WAYPOINT_COLOR: &[u8] = &[0x22, 0x33, 0xff, 0xff];
pub const WAYPOINT_REACHED_COLOR: &[u8] = &[0x55, 0xaa, 0xff, 0xff];
pub const GRID_COLOR: &[u8] = &[0xff, 0x00, 0x00, 0xff];
pub const GRAPH_COLOR: &[u8] = &[0xc9, 0x00, 0xff, 0xff];

pub const HERBIVOROUS_SPEED: f64 = 1.5; // Pixels per iteration
pub const CARNIVOROUS_SPEED: f64 = 2.0; // Pixels per iteration

pub const MS_PER_ITERATION: u64 = 16;

pub const HERBIVOROUS_NB: usize = 0;
pub const CARNIVOROUS_NB: usize = 0;
pub const PLANT_NB: usize = 1000;
pub const CORPSE_NB: usize = 0;
pub const OBSTACLES_NB: usize = 0;

pub const NEW_PLANT_PER_TICK: usize = 0;

pub const NB_PRM_POSITIONS_GENERATED: usize = 100;
