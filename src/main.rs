mod actions;
mod algorithms;
mod components;
mod configuration;
mod ecs;
mod goals;
mod goap;
mod gui;
mod shared_data;
mod systems;

use ecs::{Component, Ecs, System, Update};
use gui::renderer::Renderer;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use shared_data::biome::humidity;
use std::{any::TypeId, thread, time};

use components::agent_component::AgentComponent;
use components::all::*;
use components::body_component::BodyComponent;
use configuration::load_config;
use shared_data::body_grid;
use systems::death_system::DeathSystem;
use systems::digestion_system::DigestionSystem;
use systems::health_system::HealthSystem;
use systems::hunger_system::HungerSystem;
use systems::move_to_target_system::MoveToTargetSystem;
use systems::plant_growth_system::PlantGrowthSystem;
use systems::reproduction_system::ReproductionSystem;

use crate::actions::all::{EatCorpseAction, EatFruitAction, EatHerbivorousAction};
use crate::actions::move_to_actions::{
    MoveToNearestCorpseAction, MoveToNearestHerbivorousAction, MoveToNearestPlantWithFruitAction,
};
use crate::algorithms::rng;
use crate::configuration::Config;
use crate::goals::all::ReplenishEnergyGoal;
use crate::goap::{ActionSet, GoalSet, Goap};
use crate::systems::agent_system::AgentSystem;

pub struct World {
    ecs: Ecs,
    systems: Vec<Box<dyn System>>,
    pause: bool,
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            ecs: Ecs::new(),
            systems: Vec::new(),
            pause: false,
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn create_entity_with(&mut self, components: &[&dyn Component]) {
        self.ecs.apply(vec![Update::Create(
            components.iter().map(|c| c.clone_box()).collect(),
        )]);
    }

    fn iterate(&mut self, config: &Config) {
        if !self.pause {
            self.force_iterate(config);
        }
    }

    fn force_iterate(&mut self, config: &Config) {
        for system in self.systems.iter_mut() {
            system.run(&mut self.ecs, config);
        }
        body_grid::purge_deleted_bodies();
    }

    pub fn toogle_pause(&mut self) {
        self.pause = !self.pause;
    }

    pub fn agent_system(&self) -> Option<&AgentSystem> {
        self.systems
            .iter()
            .find_map(|system| system.as_any().downcast_ref::<AgentSystem>())
    }
}

fn create_world(config: &Config) -> World {
    let mut goap = Goap::new();

    let mut gs = GoalSet::new();
    gs.add(Box::new(ReplenishEnergyGoal::new(config)));

    let mut h_as = ActionSet::new();
    h_as.add(Box::new(MoveToNearestPlantWithFruitAction::new()));
    h_as.add(Box::new(EatFruitAction::new(config)));

    let herbivorous_goal_set = goap.add_goal_set(gs);
    let herbivorous_action_set_len = h_as.len();
    let herbivorous_action_set = goap.add_action_set(h_as);

    let mut c_as = ActionSet::new();
    c_as.add(Box::new(MoveToNearestCorpseAction::new()));
    c_as.add(Box::new(EatCorpseAction::new(config)));
    c_as.add(Box::new(MoveToNearestHerbivorousAction::new()));
    c_as.add(Box::new(EatHerbivorousAction::new(config)));

    let carnivorous_goal_set = herbivorous_goal_set;
    let carnivorous_action_set_len = c_as.len();
    let carnivorous_action_set = goap.add_action_set(c_as);

    let mut world = World::new();

    // Create seeds that will germinate instantly, so that herbivorous don't die of hunder at the
    // start of the simulation
    for _ in 0..config.bush_nb {
        world.create_entity_with(&[
            &SeedComponent::new_instant_germination(PlantKind::Bush),
            &BodyComponent::new_rand_pos_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.seed.size,
                config.seed.size,
            ),
        ]);
    }

    for _ in 0..config.tree_nb {
        world.create_entity_with(&[
            &SeedComponent::new(PlantKind::Tree, config),
            &BodyComponent::new_rand_pos_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.seed.size,
                config.seed.size,
            ),
        ]);
    }

    for _ in 0..config.herbivorous_nb {
        world.create_entity_with(&[
            &CreatureComponent::new(&config.creature),
            &BodyComponent::new_rand_pos_not_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.creature.size,
                config.creature.size,
            ),
            &HerbivorousComponent::new(),
            &AgentComponent::new(
                herbivorous_goal_set,
                herbivorous_action_set,
                herbivorous_action_set_len,
            ),
        ]);
    }

    for _ in 0..config.carnivorous_nb {
        world.create_entity_with(&[
            &CreatureComponent::new(&config.creature),
            &BodyComponent::new_rand_pos_not_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.creature.size,
                config.creature.size,
            ),
            &CarnivorousComponent::new(),
            &AgentComponent::new(
                carnivorous_goal_set,
                carnivorous_action_set,
                carnivorous_action_set_len,
            ),
        ]);
    }

    #[allow(clippy::reversed_empty_ranges)]
    for _ in 0..config.corpse_nb {
        world.create_entity_with(&[
            &CorpseComponent::new(),
            &BodyComponent::new_rand_pos_not_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.creature.size,
                config.creature.size,
            ),
        ]);
    }

    for _ in 0..config.obstacle_nb {
        world.create_entity_with(&[
            &ObstacleComponent::new(),
            &BodyComponent::new_rand_pos_not_traversable(
                config.body_domain_initial_width,
                config.body_domain_initial_height,
                config.obstacle_size,
                config.obstacle_size,
            ),
        ]);
    }

    world.add_system(Box::new(DeathSystem));
    world.add_system(Box::new(HealthSystem));
    world.add_system(Box::new(PlantGrowthSystem));
    world.add_system(Box::new(ReproductionSystem::new(
        herbivorous_goal_set,
        herbivorous_action_set,
        herbivorous_action_set_len,
        carnivorous_goal_set,
        carnivorous_action_set,
        carnivorous_action_set_len,
    )));
    world.add_system(Box::new(HungerSystem));
    world.add_system(Box::new(MoveToTargetSystem));
    world.add_system(Box::new(DigestionSystem));
    world.add_system(Box::new(AgentSystem::new(goap)));

    world
}

fn main() {
    env_logger::init();

    let mut config = load_config("config.toml");
    rng::init(&config);
    body_grid::init(&config);
    let mut world = create_world(&config);
    let default_ms_per_iteration = config.ms_per_iteration;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "civsim",
            config.renderer.screen_width,
            config.renderer.screen_height,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let canvas = window.into_canvas().build().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut renderer = Renderer::new(&config, canvas, &ttf_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    renderer.resize(w as u32, h as u32);
                }
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => match key {
                    Keycode::D => {
                        renderer.toogle_debug_mode();
                    }
                    Keycode::P => {
                        world.toogle_pause();
                    }
                    Keycode::I => {
                        world.force_iterate(&config);
                    }
                    Keycode::T => {
                        config.ms_per_iteration = if config.ms_per_iteration == 0 {
                            default_ms_per_iteration
                        } else {
                            0
                        };
                    }
                    Keycode::PageUp => {
                        renderer.zoom_in(&config);
                    }
                    Keycode::PageDown => {
                        renderer.zoom_out(&config);
                    }
                    Keycode::Up => {
                        renderer.move_camera_up(&config);
                    }
                    Keycode::Down => {
                        renderer.move_camera_down(&config);
                    }
                    Keycode::Left => {
                        renderer.move_camera_left(&config);
                    }
                    Keycode::Right => {
                        renderer.move_camera_right(&config);
                    }
                    _ => {}
                },
                Event::MouseWheel { y, .. } => {
                    if y > 0 {
                        renderer.zoom_in(&config);
                    } else {
                        renderer.zoom_out(&config);
                    }
                }
                Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks: _,
                    x,
                    y,
                } => {
                    if mouse_btn == MouseButton::Left {
                        renderer.select_agent_by_click(&mut world.ecs, x, y);
                    }
                }
                _ => {}
            }
        }
        world.iterate(&config);
        renderer.draw(&mut world, &config);
        thread::sleep(time::Duration::from_millis(config.ms_per_iteration));
    }
}
