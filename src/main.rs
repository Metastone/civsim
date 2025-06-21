mod ecs;

use ecs::{ArchetypeManager, Component, ComponentType, EntityId, EntityIdAllocator, System};
use pixels::{Pixels, SurfaceTexture};
use std::{collections::HashMap, sync::Arc, thread, time, usize};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const HUNGER_RATE: f32 = 1.0;
const EXHAUSTION_RATE: f32 = 1.0;
const FOOD_ENERGY: f32 = 20.0;
const MAX_ENERGY: f32 = 100.0;
const MAX_HEALTH: f32 = 100.0;
const PERSON_PLACEHOLDER_PIXEL_SIZE: u32 = 30;
const FOOD_PLACEHOLDER_PIXEL_SIZE: u32 = 10;
const PERSON_COLOR: &[u8] = &[0xff, 0x55, 0x11, 0xff];
const FOOD_COLOR: &[u8] = &[0x22, 0xbb, 0x11, 0xff];
const PERSON_SPEED: f64 = 3.0; // Pixels per iteration
const MS_PER_ITERATION: u64 = 16;
const PERSON_NB: usize = 1000;
const FOOD_NB: usize = 20000;

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
            energy: MAX_ENERGY,
            health: MAX_HEALTH,
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
        let x = rand::random_range((-1.0 * WIDTH as f64 / 2.0)..(WIDTH as f64 / 2.0));
        let y = rand::random_range((-1.0 * HEIGHT as f64 / 2.0)..(HEIGHT as f64 / 2.0));
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
struct FoodComponent {}
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
struct EatingFoodComponent {
    food_entity: EntityId,
}
impl Component for EatingFoodComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::EatingFood
    }
}
impl EatingFoodComponent {
    fn new(food_entity: EntityId) -> Self {
        Self { food_entity }
    }
}

#[derive(Clone, Copy)]
struct BehaviorComponent {}
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
        for (arch_index, entity_index, _) in manager.iter_entities(ComponentType::Person) {
            if let Some(person) = manager.get_component::<PersonComponent>(
                arch_index,
                entity_index,
                &ComponentType::Person,
            ) {
                person.energy -= HUNGER_RATE;
                if person.energy <= 0.0 {
                    person.energy = 0.0;
                }
            }
        }
    }
}

struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in manager.iter_entities(ComponentType::Person) {
            if let Some(person) = manager.get_component::<PersonComponent>(
                arch_index,
                entity_index,
                &ComponentType::Person,
            ) {
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

struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        let mut to_remove = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::Person) {
            if let Some(person) = manager.get_component::<PersonComponent>(
                arch_index,
                entity_index,
                &ComponentType::Person,
            ) {
                if person.health <= 0.0 {
                    to_remove.push(entity);
                }
            }
        }
        to_remove
            .iter()
            .for_each(|entity| manager.remove_entity(*entity));
    }
}

struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all entities with a behavior
        let mut behavior_positions = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Behavior, ComponentType::Position])
        {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                behavior_positions.insert(entity.clone(), position.clone());
            }
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &behavior_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);
            for (arch_index, entity_index, food_entity) in
                manager.iter_entities_with(&[ComponentType::Food, ComponentType::Position])
            {
                if let Some(food_position) = manager.get_component::<PositionComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::Position,
                ) {
                    let distance_squared = (food_position.x - position.x).powi(2)
                        + (food_position.y - position.y).powi(2);
                    if distance_squared < closest_distance_squared {
                        closest_distance_squared = distance_squared;
                        found.insert(*entity, true);
                        closest_position.insert(*entity, food_position.clone());
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Move all entities with a behavior in direction of the closest food
        let mut person_to_food: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Behavior, ComponentType::Position])
        {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                if *found.get(&entity).unwrap() {
                    let food_position = closest_position.get(&entity).unwrap();
                    let food_entity = closest_entity.get(&entity).unwrap();
                    let vec_to_food = (food_position.x - position.x, food_position.y - position.y);
                    let norm = (vec_to_food.0.powi(2) + vec_to_food.1.powi(2)).sqrt();
                    if norm
                        < (PERSON_PLACEHOLDER_PIXEL_SIZE as f64 / 2.0
                            + FOOD_PLACEHOLDER_PIXEL_SIZE as f64 / 2.0)
                    {
                        // Food reached -> will go to eating state
                        person_to_food.insert(entity, *food_entity);
                    } else {
                        // Get closer to the food
                        position.x += vec_to_food.0 / norm * PERSON_SPEED;
                        position.y += vec_to_food.1 / norm * PERSON_SPEED;
                    }
                }
            }
        }

        // If food reached, go to eating state
        for (entity, food_entity) in person_to_food {
            manager.add_component(entity, &EatingFoodComponent::new(food_entity));
        }
    }
}

struct EatSystem;
impl System for EatSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Make sure that a food is not eaten by more than one person
        let mut food_to_person: HashMap<EntityId, EntityId> = HashMap::new();
        let mut persons_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::EatingFood) {
            if let Some(eating_food) = manager.get_component::<EatingFoodComponent>(
                arch_index,
                entity_index,
                &ComponentType::EatingFood,
            ) {
                food_to_person.insert(eating_food.food_entity, entity);
            }
            persons_trying_to_eat.push(entity);
        }

        // Increase energy of persons that ate a food
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::EatingFood, ComponentType::Person])
        {
            if let Some(person) = manager.get_component::<PersonComponent>(
                arch_index,
                entity_index,
                &ComponentType::Person,
            ) {
                if food_to_person
                    .values()
                    .any(|&person_entity| person_entity == entity)
                {
                    person.energy += FOOD_ENERGY;
                    if person.energy > MAX_ENERGY {
                        person.energy = MAX_ENERGY;
                    }
                }
            }
        }

        // Remove eaten food entities
        for food_entity in food_to_person.keys() {
            manager.remove_entity(*food_entity);
        }

        // Remove all "eating food" components
        for entity in persons_trying_to_eat.iter() {
            manager.remove_component(*entity, &ComponentType::EatingFood);
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
            systems: Vec::new(),
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
        for (arch_index, entity_index, _) in self
            .archetype_manager
            .iter_entities(ComponentType::Position)
        {
            let (color, placeholder_pixel_size) = if let Some(_) =
                self.archetype_manager.get_component::<PersonComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::Person,
                ) {
                (PERSON_COLOR, PERSON_PLACEHOLDER_PIXEL_SIZE)
            } else {
                (FOOD_COLOR, FOOD_PLACEHOLDER_PIXEL_SIZE)
            };

            if let Some(position) = self.archetype_manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                let pos_in_window = (
                    position.x + (window_width as f64) / 2.0,
                    position.y * -1.0 + (window_height as f64) / 2.0,
                );
                let r = placeholder_pixel_size as i64 / 2;
                for i in (-1 * r)..r {
                    for j in (-1 * r)..r {
                        let pixel_pos = (pos_in_window.0 as i64 + i, pos_in_window.1 as i64 + j);
                        if pixel_pos.0 >= 0
                            && pixel_pos.0 < window_width as i64
                            && pixel_pos.1 >= 0
                            && pixel_pos.1 < window_height as i64
                        {
                            let index = ((pixel_pos.1 as usize) * (window_width as usize)
                                + (pixel_pos.0 as usize))
                                * 4;
                            pixels[index..(index + 4)].copy_from_slice(color);
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

        for _ in 0..PERSON_NB {
            let id = ids.get_next_id();
            world.add_component(id, &PersonComponent::new());
            world.add_component(id, &PositionComponent::new());
            world.add_component(id, &BehaviorComponent::new());
        }

        for _ in 0..FOOD_NB {
            let id = ids.get_next_id();
            world.add_component(id, &FoodComponent::new());
            world.add_component(id, &PositionComponent::new());
        }

        world.add_system(Box::new(HungerSystem));
        world.add_system(Box::new(ExhaustionSystem));
        world.add_system(Box::new(DeathSystem));
        world.add_system(Box::new(MoveToFoodSystem));
        world.add_system(Box::new(EatSystem));

        Self {
            window: Default::default(),
            pixels: Default::default(),
            world,
        }
    }
}
impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Civsim")
                        .with_inner_size(LogicalSize::new(WIDTH as f64, HEIGHT as f64)),
                )
                .unwrap(),
        );
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            Pixels::new(WIDTH, HEIGHT, surface_texture).unwrap()
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

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut App::default()).unwrap();
}
