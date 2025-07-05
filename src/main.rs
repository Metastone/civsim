mod components;
mod constants;
#[macro_use]
mod ecs;
mod display;
mod shared_data;
mod systems;

use ecs::{Component, Ecs, System, Update};
use pixels::{Pixels, SurfaceTexture};
use std::any::TypeId;
use std::{
    sync::Arc,
    thread,
    time::{self, Instant},
};

use components::all::*;
use components::body_component::BodyComponent;
use constants::*;
use shared_data::body_grid;
use systems::attack_herbivorous_system::AttackHerbivorousSystem;
use systems::carnivorous_mind_system::CarnivorousMindSystem;
use systems::death_system::DeathSystem;
use systems::eat_corpse_system::EatCorpseSystem;
use systems::eat_food_system::EatFoodSystem;
use systems::exhaustion_system::ExhaustionSystem;
use systems::food_growth_system::FoodGrowthSystem;
use systems::herbivorous_mind_system::HerbivorousMindSystem;
use systems::hunger_system::HungerSystem;
use systems::move_to_corpse_system::MoveToCorpseSystem;
use systems::move_to_food_system::MoveToFoodSystem;
use systems::move_to_herbivorous_system::MoveToHerbivorousSystem;
use systems::reproduction_system::ReproductionSystem;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct World {
    ecs: Ecs,
    systems: Vec<Box<dyn System>>,
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

    pub fn iterate(&mut self) {
        for s in &self.systems {
            s.run(&mut self.ecs);
        }
        body_grid::purge_deleted_bodies();
    }

    fn draw(&mut self, pixels: &mut [u8], window_width: u32, window_height: u32) {
        display::draw(&mut self.ecs, pixels, window_width, window_height);
    }

    pub fn print_summary(&self) {
        let food = iter_entities!(self.ecs, FoodComponent).count();
        let herbivorous = iter_entities!(self.ecs, HerbivorousComponent).count();
        let carnivorous = iter_entities!(self.ecs, CarnivorousComponent).count();
        let corpses = iter_entities!(self.ecs, CorpseComponent).count();
        println!("\tfood = {food}\n\therbivorous = {herbivorous}\n\tcarnivorous = {carnivorous}\n\tcorpses = {corpses}\n");
    }
}

fn create_world() -> World {
    let mut world = World::new();

    for _ in 0..FOOD_NB {
        world.create_entity_with(&[
            &FoodComponent::new(),
            &BodyComponent::new_rand_pos_traversable(
                FOOD_PIXEL_SIZE.into(),
                FOOD_PIXEL_SIZE.into(),
            ),
        ]);
    }

    for _ in 0..HERBIVOROUS_NB {
        world.create_entity_with(&[
            &CreatureComponent::new(),
            &BodyComponent::new_rand_pos_not_traversable(
                CREATURE_PIXEL_SIZE.into(),
                CREATURE_PIXEL_SIZE.into(),
            ),
            &HerbivorousComponent::new(),
            &InactiveComponent::new(),
        ]);
    }

    for _ in 0..CARNIVOROUS_NB {
        world.create_entity_with(&[
            &CreatureComponent::new(),
            &BodyComponent::new_rand_pos_not_traversable(
                CREATURE_PIXEL_SIZE.into(),
                CREATURE_PIXEL_SIZE.into(),
            ),
            &CarnivorousComponent::new(),
            &InactiveComponent::new(),
        ]);
    }

    #[allow(clippy::reversed_empty_ranges)]
    for _ in 0..CORPSE_NB {
        world.create_entity_with(&[
            &CorpseComponent::new(),
            &BodyComponent::new_rand_pos_not_traversable(
                CREATURE_PIXEL_SIZE.into(),
                CREATURE_PIXEL_SIZE.into(),
            ),
        ]);
    }

    world.add_system(Box::new(FoodGrowthSystem));
    world.add_system(Box::new(ReproductionSystem));
    world.add_system(Box::new(HerbivorousMindSystem));
    world.add_system(Box::new(MoveToFoodSystem));
    world.add_system(Box::new(EatFoodSystem));
    world.add_system(Box::new(CarnivorousMindSystem));
    world.add_system(Box::new(MoveToCorpseSystem));
    world.add_system(Box::new(MoveToHerbivorousSystem));
    world.add_system(Box::new(EatCorpseSystem));
    world.add_system(Box::new(AttackHerbivorousSystem));
    world.add_system(Box::new(HungerSystem));
    world.add_system(Box::new(ExhaustionSystem));
    world.add_system(Box::new(DeathSystem));

    world
}

struct App<'window> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'window>>,
    world: World,
}
impl Default for App<'_> {
    fn default() -> Self {
        Self {
            window: Default::default(),
            pixels: Default::default(),
            world: create_world(),
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
                            SCREEN_WIDTH as f64,
                            SCREEN_HEIGHT as f64,
                        )),
                )
                .unwrap(),
        );
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture).unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(pixels);
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
            }
            WindowEvent::RedrawRequested => {
                self.world.iterate();

                let window_size = self.window.as_ref().unwrap().inner_size();
                self.world.draw(
                    self.pixels.as_mut().unwrap().frame_mut(),
                    window_size.width,
                    window_size.height,
                );
                self.pixels.as_mut().unwrap().render().unwrap();

                thread::sleep(time::Duration::from_millis(MS_PER_ITERATION));
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let mut world = create_world();

    let old = Instant::now();
    let mut i: usize = 0;
    let nb_iter_step = 10;
    loop {
        world.iterate();

        if i % nb_iter_step == 0 {
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
