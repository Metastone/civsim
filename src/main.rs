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
use pixels::{Pixels, SurfaceTexture};
use shared_data::biome::humidity;
use std::{
    any::TypeId,
    sync::Arc,
    thread,
    time::{self, Instant},
};

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

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, MouseScrollDelta, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::algorithms::rng;
use crate::configuration::Config;

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
        // Probably slow to apply updates one by one, but okay for world initialization ?
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
        for s in &self.systems {
            s.run(&mut self.ecs, config);
        }
        body_grid::purge_deleted_bodies();
    }

    pub fn toogle_pause(&mut self) {
        self.pause = !self.pause;
    }

    pub fn print_summary(&self) {
        let plants = iter_entities!(self.ecs, PlantComponent).count();
        let herbivorous = iter_entities!(self.ecs, HerbivorousComponent).count();
        let carnivorous = iter_entities!(self.ecs, CarnivorousComponent).count();
        let corpses = iter_entities!(self.ecs, CorpseComponent).count();
        println!("\tplants = {plants}\n\therbivorous = {herbivorous}\n\tcarnivorous = {carnivorous}\n\tcorpses = {corpses}\n");
    }
}

fn create_world(config: &Config) -> World {
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

    world
}

struct App<'window> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'window>>,
    world: World,
    display: Display,
    config: Config,
    default_ms_per_iteration: u64,
}
impl Default for App<'_> {
    fn default() -> Self {
        let config = load_config("config.toml");
        rng::init(&config);
        body_grid::init(&config);
        Self {
            window: Default::default(),
            pixels: Default::default(),
            world: create_world(&config),
            display: Display::new(&config),
            config,
            default_ms_per_iteration: config.ms_per_iteration,
        }
    }
}
impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Civsim")
                        .with_inner_size(LogicalSize::new(
                            self.config.display.screen_width as f64,
                            self.config.display.screen_height as f64,
                        )),
                )
                .unwrap(),
        );
        let size = window.inner_size();
        let pixels = {
            let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
            Pixels::new(
                self.config.display.screen_width,
                self.config.display.screen_height,
                surface_texture,
            )
            .unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(pixels);
        self.display.resize(size.width, size.height);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.pixels
                    .as_mut()
                    .unwrap()
                    .resize_buffer(size.width, size.height)
                    .unwrap();
                self.pixels
                    .as_mut()
                    .unwrap()
                    .resize_surface(size.width, size.height)
                    .unwrap();
                self.display.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                self.world.iterate(&self.config);

                self.display.draw(
                    &mut self.world.ecs,
                    self.pixels.as_mut().unwrap().frame_mut(),
                );
                self.pixels.as_mut().unwrap().render().unwrap();

                thread::sleep(time::Duration::from_millis(self.config.ms_per_iteration));
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.logical_key == Key::Character("d".into())
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.toogle_debug_mode();
                } else if event.logical_key == Key::Character("p".into())
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.world.toogle_pause();
                } else if event.logical_key == Key::Character("i".into())
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.world.force_iterate(&self.config);
                } else if event.logical_key == Key::Character("t".into())
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.config.ms_per_iteration = if self.config.ms_per_iteration == 0 {
                        self.default_ms_per_iteration
                    } else {
                        0
                    };
                } else if event.logical_key == Key::Named(NamedKey::PageUp)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.zoom_in();
                } else if event.logical_key == Key::Named(NamedKey::PageDown)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.zoom_out();
                } else if event.logical_key == Key::Named(NamedKey::ArrowUp)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.move_camera_up();
                } else if event.logical_key == Key::Named(NamedKey::ArrowDown)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.move_camera_down();
                } else if event.logical_key == Key::Named(NamedKey::ArrowLeft)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.move_camera_left();
                } else if event.logical_key == Key::Named(NamedKey::ArrowRight)
                    && event.state == ElementState::Pressed
                    && !event.repeat
                {
                    self.display.move_camera_right();
                }
            }
            WindowEvent::MouseWheel {
                device_id: _,
                delta: MouseScrollDelta::LineDelta(_x, y),
                ..
            } => {
                if y > 0.0 {
                    self.display.zoom_in();
                } else {
                    self.display.zoom_out();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let config = load_config("config.toml");
    rng::init(&config);
    body_grid::init(&config);
    let mut world = create_world(&config);

    let old = Instant::now();
    let mut i: usize = 0;
    let nb_iter_step = 10;
    loop {
        world.iterate(&config);

        if i.is_multiple_of(nb_iter_step) {
            println!("{}", i);
            world.print_summary();

            let elapsed_seconds = (Instant::now() - old).as_secs();
            if elapsed_seconds != 0 {
                let iter_per_sec = i as u64 / elapsed_seconds;
                println!("iterations per second: {iter_per_sec:?}");
            }
        }

        i += 1;
    }
}
