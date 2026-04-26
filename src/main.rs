mod algorithms;
mod components;
mod configuration;
#[macro_use]
mod ecs;
mod display;
mod shared_data;
mod systems;

use display::Display;
use ecs::{Component, Ecs, System, Update};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use shared_data::biome::humidity;
use std::any::TypeId;
use std::{thread, time};

use components::agent_component::*;
use components::all::*;
use components::body_component::BodyComponent;
use configuration::load_config;
use shared_data::body_grid;
use systems::attack_herbivorous_system::AttackHerbivorousSystem;
use systems::carnivorous_mind_system::CarnivorousMindSystem;
use systems::death_system::DeathSystem;
use systems::digestion_system::DigestionSystem;
use systems::eat_corpse_system::EatCorpseSystem;
use systems::eat_plant_system::EatPlantSystem;
use systems::health_system::HealthSystem;
use systems::herbivorous_mind_system::HerbivorousMindSystem;
use systems::hunger_system::HungerSystem;
use systems::move_to_target_system::MoveToTargetSystem;
use systems::plant_growth_system::PlantGrowthSystem;
use systems::reproduction_system::ReproductionSystem;

use crate::algorithms::rng;
use crate::configuration::Config;
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
}

fn create_world(config: &Config) -> World {
    let mut goal_set = GoalSet::new();
    goal_set.add(Box::new(ReplenishEnergyGoal::new(config)));

    let mut action_set = ActionSet::new();
    action_set.add(Box::new(MoveToNearestPlantAction {}));

    let mut goap = Goap::new();
    let herbivorous_goal_set = goap.add_goal_set(goal_set);
    let herbivorous_action_set = goap.add_action_set(action_set);
    let carnivorous_goal_set = herbivorous_goal_set;
    let carnivorous_action_set = herbivorous_action_set;

    let mut world = World::new();

    for _ in 0..config.plant_nb {
        // Plants start as seed, which have no collision. They gain collision later on.
        world.create_entity_with(&[
            &PlantComponent::new(config),
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
            &InactiveComponent::new(),
            &AgentComponent::new(herbivorous_goal_set, herbivorous_action_set),
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
            &InactiveComponent::new(),
            &AgentComponent::new(carnivorous_goal_set, carnivorous_action_set),
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
    world.add_system(Box::new(ReproductionSystem));
    world.add_system(Box::new(HerbivorousMindSystem));
    world.add_system(Box::new(EatPlantSystem));
    world.add_system(Box::new(CarnivorousMindSystem));
    world.add_system(Box::new(EatCorpseSystem));
    world.add_system(Box::new(AttackHerbivorousSystem));
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
    let mut display = Display::new(&config);
    let default_ms_per_iteration = config.ms_per_iteration;

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "civsim",
            config.display.screen_width,
            config.display.screen_height,
        )
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    display.resize(w as u32, h as u32);
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
                        display.toogle_debug_mode();
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
                        display.zoom_in();
                    }
                    Keycode::PageDown => {
                        display.zoom_out();
                    }
                    Keycode::Up => {
                        display.move_camera_up();
                    }
                    Keycode::Down => {
                        display.move_camera_down();
                    }
                    Keycode::Left => {
                        display.move_camera_left();
                    }
                    Keycode::Right => {
                        display.move_camera_right();
                    }
                    _ => {}
                },
                Event::MouseWheel { y, .. } => {
                    if y > 0 {
                        display.zoom_in();
                    } else {
                        display.zoom_out();
                    }
                }
                _ => {}
            }
        }
        world.iterate(&config);
        display.draw(&mut world.ecs, &mut canvas);
        canvas.present();
        thread::sleep(time::Duration::from_millis(config.ms_per_iteration));
    }
}
