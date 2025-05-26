mod ecs;

use std::{collections::HashMap, iter::zip, sync::Arc, thread, time, usize};
use ecs::{ArchetypeManager, Component, ComponentType, EntityId, EntityIdAllocator, System};
use pixels::{Pixels, SurfaceTexture};
use winit::{application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;
const HUNGER_RATE: f32 = 1.0;
const EXHAUSTION_RATE: f32 = 1.0;
const PERSON_PLACEHOLDER_PIXEL_SIZE: u32 = 30;
const FOOD_PLACEHOLDER_PIXEL_SIZE: u32 = 10;
const PERSON_COLOR: &[u8] = &[0xff, 0x55, 0x11, 0xff];
const FOOD_COLOR: &[u8] = &[0x22, 0xbb, 0x11, 0xff];

#[derive(Clone, Copy)]
struct PersonComponent {
    energy: f32,
    health: f32,
}
impl Component for PersonComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Person
    }
}
impl PersonComponent {
    fn new() -> Self {
        Self {
            energy: 100.0,
            health: 100.0,
        }
    }
}

#[derive(Clone, Copy)]
struct PositionComponent {
    x: f64,
    y: f64,
}
impl Component for PositionComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Position
    }
}
impl PositionComponent {
    fn new() -> Self {
        let x = rand::random_range(-300.0..300.0);
        let y = rand::random_range(-200.0..200.0);
        Self {
            x,
            y,
        }
    }
}

#[derive(Clone, Copy)]
struct FoodComponent {
}
impl Component for FoodComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Food
    }
}
impl FoodComponent {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
struct BehaviorComponent {
}
impl Component for BehaviorComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Behavior
    }
}
impl BehaviorComponent {
    fn new() -> Self {
        Self {}
    }
}

struct HungerSystem;
impl System for HungerSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with_comp(&ComponentType::Person) {
            for component in manager.get_components(arch_index, &ComponentType::Person).iter_mut() {
                if let Some(person) = component.as_any_mut().downcast_mut::<PersonComponent>() {
                    person.energy -= HUNGER_RATE;
                    if person.energy <= 0.0 {
                        person.energy = 0.0;
                    }
                }
            }
        }
    }
}

struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with_comp(&ComponentType::Person) {
            for component in manager.get_components(arch_index, &ComponentType::Person).iter_mut() {
                if let Some(person) = component.as_any_mut().downcast_mut::<PersonComponent>() {
                    if person.energy <= 0.0 {
                        person.health -= EXHAUSTION_RATE;
                    }
                    if person.health <= 0.0 {
                        person.health = 0.0;
                    }
                }
            }
        }
    }
}

struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
         for arch_index in manager.with_comp(&ComponentType::Person) {
            let mut entities_to_remove: Vec<usize> = manager.get_components(arch_index, &ComponentType::Person).iter().enumerate()
                .filter(|(_, component)| {
                    if let Some(person) = component.as_any().downcast_ref::<PersonComponent>() {
                        person.health <= 0.0
                    } else {
                        false
                    }
                })
                .map(|(i, _)| i)
                .collect();
            // Ugly: sort in decreased order so that indexes stay valid even while modifying the internal vectors
            entities_to_remove.sort_by(|a, b| b.cmp(a));
            for entity_index in entities_to_remove.iter() {
                manager.remove_entity(arch_index, *entity_index);
            }
        }
    }
}

#[allow(dead_code)]
struct MoveEastSystem;
impl System for MoveEastSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with_comp(&ComponentType::Position) {
            for component in manager.get_components(arch_index, &ComponentType::Position).iter_mut() {
                if let Some(position) = component.as_any_mut().downcast_mut::<PositionComponent>() {
                    position.x += 1.0;
                }
            };
        }
    }
}

struct BehaviorSystem;
impl System for BehaviorSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all entities with a behavior
        let mut behavior_positions = HashMap::new();
        for arch_index in manager.with_comp(&ComponentType::Behavior) {
            let (data, entities) = manager.get_components_with_eids(arch_index, &ComponentType::Position);
            for (component, entity) in zip(data.into_iter(), entities.into_iter()) {
                if let Some(position) = component.as_any_mut().downcast_mut::<PositionComponent>() {
                    behavior_positions.insert(entity.clone(), position.clone());
               }
            };
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        for (entity, position) in &behavior_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);
            closest_position.insert(*entity, PositionComponent { x: 0.0, y: 0.0 });
            for arch_index in manager.with_comp(&ComponentType::Food) {
                let (data, entities) = manager.get_components_with_eids(arch_index, &ComponentType::Position);
                for (component, _) in zip(data.into_iter(), entities.into_iter()) {
                    if let Some(food_position) = component.as_any_mut().downcast_mut::<PositionComponent>() {
                        let distance_squared = (food_position.x - position.x).powi(2) + (food_position.y - position.y).powi(2);
                        if distance_squared < closest_distance_squared {
                            closest_distance_squared = distance_squared;
                            found.insert(*entity, true);
                            closest_position.insert(*entity, food_position.clone());
                        }
                    }
                }
            }
        }

        // Move all entities with a behavior in direction of the closest food
        for arch_index in manager.with_comp(&ComponentType::Behavior) {
            let (data, entities) = manager.get_components_with_eids(arch_index, &ComponentType::Position);
            for (component, entity) in zip(data.into_iter(), entities.into_iter()) {
                if let Some(position) = component.as_any_mut().downcast_mut::<PositionComponent>() {
                    if *found.get(entity).unwrap() {
                        let food_position = closest_position.get(entity).unwrap();
                        let food_direction = (
                            food_position.x - position.x,
                            food_position.y - position.y
                        );
                        let direction_norm = (food_direction.0.powi(2) + food_direction.1.powi(2)).sqrt();
                        position.x += food_direction.0 / direction_norm;
                        position.y += food_direction.1 / direction_norm;
                    }

                }
            }
        }
    }
}


pub struct World {
    archetype_manager: ArchetypeManager,
    systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            archetype_manager: ArchetypeManager::new(),
            systems: Vec::new()
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn add_component(&mut self, entity_id: EntityId, new_comp: &dyn Component) {
        self.archetype_manager.add_component(entity_id, new_comp);
    }

    pub fn iterate(&mut self) {
        for s in &self.systems {
            s.run(&mut self.archetype_manager);
        }
    }

    pub fn draw(&mut self, pixels: &mut [u8], window_width: u32, window_height: u32) {
        // Background
        for (_i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
            pixel.copy_from_slice(&[0xcc, 0xcc, 0xcc, 0xff]);
        }

        // Draw entities with position
        for arch_index in self.archetype_manager.with_comp(&ComponentType::Position) {
            let (color, placeholder_pixel_size) =
            if self.archetype_manager.get_component_types(arch_index).contains(&ComponentType::Person) {
                (PERSON_COLOR, PERSON_PLACEHOLDER_PIXEL_SIZE)
            } else {
                (FOOD_COLOR, FOOD_PLACEHOLDER_PIXEL_SIZE)
            };

            for component in self.archetype_manager.get_components(arch_index, &ComponentType::Position).iter_mut() {
                if let Some(position) = component.as_any_mut().downcast_mut::<PositionComponent>() {
                    let pos_in_window = (position.x + (window_width as f64) / 2.0, position.y * -1.0 + (window_height as f64) / 2.0);
                    for i in 0..placeholder_pixel_size {
                        for j in 0..placeholder_pixel_size {
                            let pixel_pos = (pos_in_window.0 as i64 + i as i64, pos_in_window.1 as i64 + j as i64);
                            if pixel_pos.0 >= 0 && pixel_pos.0 < window_width as i64 && pixel_pos.1 >= 0 && pixel_pos.1 < window_height as i64 {
                                let index = ((pixel_pos.1 as usize) * (window_width as usize) + (pixel_pos.0 as usize)) * 4;
                                pixels[index..(index + 4)].copy_from_slice(color);
                            }
                        }
                    }
                }
            }
        }
    }
}

struct App<'window> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'window>>,
    world: World,
}
impl<'window> Default for App<'window> {
    fn default() -> Self {
        let mut world = World::new();
        let mut ids = EntityIdAllocator::new();

        for _ in 0..2 {
            let id = ids.get_next_id();
            world.add_component(id, &PersonComponent::new());
            world.add_component(id, &PositionComponent::new());
            world.add_component(id, &BehaviorComponent::new());
        }

        for _ in 0..10 {
            let id = ids.get_next_id();
            world.add_component(id, &FoodComponent::new());
            world.add_component(id, &PositionComponent::new());
        }

        world.add_system(Box::new(HungerSystem));
        world.add_system(Box::new(ExhaustionSystem));
        world.add_system(Box::new(DeathSystem));
        world.add_system(Box::new(BehaviorSystem));
        //world.add_system(Box::new(MoveEastSystem));

        Self {
            window: Default::default(),
            pixels: Default::default(),
            world
        }
    }
}
impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(
            Window::default_attributes()
            .with_title("Civsim")
            .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64))
        ).unwrap());
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
        };

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); },
            WindowEvent::Resized(size) => {
                self.pixels.as_mut().unwrap().resize_buffer(size.width, size.height).unwrap();
                self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).unwrap();
            },
            WindowEvent::RedrawRequested => {
                self.world.iterate();

                let window_size = self.window.as_ref().unwrap().inner_size();
                self.world.draw(self.pixels.as_mut().unwrap().frame_mut(), window_size.width, window_size.height);
                self.pixels.as_mut().unwrap().render().unwrap();

                thread::sleep(time::Duration::from_millis(100));
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut App::default()).unwrap();
}
