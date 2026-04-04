use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone, Copy)]
pub struct Config {
    pub rng_seed: u64,
    pub body_domain_initial_width: f64,
    pub body_domain_initial_height: f64,
    pub herbivorous_nb: usize,
    pub carnivorous_nb: usize,
    pub obstacle_nb: usize,
    pub corpse_nb: usize,
    pub plant_nb: usize,
    pub ms_per_iteration: u64,
    pub obstacle_size: f64,
    pub seed: SeedConfig,
    pub plant: PlantConfig,
    pub creature: CreatureConfig,
    pub path: PathConfig,
    pub collision: CollisionConfig,
    pub display: DisplayConfig,
}

#[derive(Deserialize, Clone, Copy)]
pub struct SeedConfig {
    // Seed display size different to make it more visible
    pub size: f64,
    pub display_size: f64,
}

#[derive(Deserialize, Clone, Copy)]
pub struct PlantConfig {
    pub ticks_as_seed: usize,
    pub initial_size: f64,
    pub max_size: f64,
    pub size_growth_per_tick: f64,
    pub max_seeds: usize,
    pub ticks_per_seed: usize,
    pub energy_per_size_unit: f32,
}

#[derive(Deserialize, Clone, Copy)]
pub struct CreatureConfig {
    pub size: f64,
    pub hunger_rate: f32,
    pub exhaustion_rate: f32,
    pub recovery_rate: f32,
    pub corpse_energy: f32,
    pub carnivorous_attack: f32,
    pub reprod_energy_threshold: f32,
    pub reprod_energy_cost: f32,
    pub start_energy: f32,
    pub max_energy: f32,
    pub max_health: f32,
    pub herbivorous_ticks_to_digest: usize,
    pub total_ticks_idle: usize,
    pub reprod_x_offset: f64,
    pub herbivorous_speed: f64,
    pub carnivorous_speed: f64,
}

#[derive(Deserialize, Clone, Copy)]
pub struct PathConfig {
    pub max_search_distance: f64,
    pub nb_prm_positions_generated: usize,
}

#[derive(Deserialize, Clone, Copy)]
pub struct CollisionConfig {
    pub cell_size_factor: f64,
    pub contact_center_2_center_factor: f64,
}

#[derive(Deserialize, Clone, Copy)]
pub struct DisplayConfig {
    pub screen_width: u32,
    pub screen_height: u32,
    pub initial_zoom: f64,
    pub zoom_factor: f64,
    pub move_camera_offset: isize,
    pub bar_width: f64,
    pub bar_height: f64,
    pub grid_line_wideness: f64,
    pub graph_edge_thickness: f64,
    pub color: DisplayColorConfig,
}

#[derive(Deserialize, Clone, Copy)]
pub struct DisplayColorConfig {
    pub background_color: [u8; 4],
    pub herbivorous_color: [u8; 4],
    pub carnivorous_color: [u8; 4],
    pub plant_color: [u8; 4],
    pub energy_color: [u8; 4],
    pub health_color: [u8; 4],
    pub corpse_color: [u8; 4],
    pub obstacle_color: [u8; 4],
    pub waypoint_color: [u8; 4],
    pub waypoint_reached_color: [u8; 4],
    pub grid_color: [u8; 4],
    pub graph_color: [u8; 4],
    pub seed_color: [u8; 4],
}

pub fn load_config(file_name: &str) -> Config {
    let content = fs::read_to_string(file_name)
        .unwrap_or_else(|_| panic!("Failed to read configuration file {}", file_name));
    toml::from_str(&content)
        .unwrap_or_else(|_| panic!("Failed to parse configuration file {}", file_name))
}
