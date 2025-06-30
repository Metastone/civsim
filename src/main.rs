mod ecs;

use ecs::{ArchetypeManager, Component, ComponentType, EntityId, System};
use pixels::{Pixels, SurfaceTexture};
use std::{cmp, collections::HashMap, sync::Arc, thread, time};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const SCREEN_WIDTH: u32 = 2880;
const SCREEN_HEIGHT: u32 = 1620;

const HUNGER_RATE: f32 = 0.2;
const EXHAUSTION_RATE: f32 = 1.0;

const FOOD_ENERGY: f32 = 20.0;
const CORPSE_ENERGY: f32 = 20.0;
const REPROD_ENERGY_THRESHOLD: f32 = 90.0;
const REPROD_ENERGY_COST: f32 = 50.0;
const START_ENERGY: f32 = 40.0;
const MAX_ENERGY: f32 = 100.0;
const MAX_HEALTH: f32 = 100.0;
const CORPSE_MASS: f32 = 100.0;

const CREATURE_PIXEL_SIZE: u32 = 60;
const FOOD_PIXEL_SIZE: u32 = 20;

const BAR_WIDTH: u32 = 60;
const BAR_HEIGHT: u32 = 10;

const HERBIVOROUS_COLOR: &[u8] = &[0xff, 0x99, 0x11, 0xff];
const CARNIVOROUS_COLOR: &[u8] = &[0xff, 0x22, 0x11, 0xff];
const FOOD_COLOR: &[u8] = &[0x22, 0xbb, 0x11, 0xff];
const ENERGY_COLOR: &[u8] = &[0x11, 0xff, 0x88, 0xff];
const HEALTH_COLOR: &[u8] = &[0xff, 0x11, 0x11, 0xff];
const CORPSE_COLOR: &[u8] = &[0x44, 0x11, 0x11, 0xff];

const CREATURE_SPEED: f64 = 3.0; // Pixels per iteration
const MS_PER_ITERATION: u64 = 16;

const HERBIVOROUS_NB: usize = 10;
const CARNIVOROUS_NB: usize = 1;
const FOOD_NB: usize = 100;
const CORPSE_NB: usize = 30;
const NEW_FOOD_PER_TICK: usize = 1;

#[derive(Clone, Copy)]
struct CreatureComponent {
    energy: f32,
    health: f32,
}
impl Component for CreatureComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Creature
    }
}
impl CreatureComponent {
    fn new() -> Self {
        Self {
            energy: START_ENERGY,
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
        let x = rand::random_range((SCREEN_WIDTH as f64 / -2.0)..(SCREEN_WIDTH as f64 / 2.0));
        let y = rand::random_range((SCREEN_HEIGHT as f64 / -2.0)..(SCREEN_HEIGHT as f64 / 2.0));
        Self { x, y }
    }

    fn from(x: f64, y: f64) -> Self {
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
struct EatingCorpseComponent {
    corpse_entity: EntityId,
}
impl Component for EatingCorpseComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::EatingCorpse
    }
}
impl EatingCorpseComponent {
    fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone, Copy)]
struct CorpseComponent {
    mass: f32,
}
impl Component for CorpseComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Corpse
    }
}
impl CorpseComponent {
    fn new() -> Self {
        Self { mass: CORPSE_MASS }
    }
}

#[derive(Clone, Copy)]
struct HerbivorousComponent {}
impl Component for HerbivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Herbivorous
    }
}
impl HerbivorousComponent {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
struct CarnivorousComponent {}
impl Component for CarnivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Carnivorous
    }
}
impl CarnivorousComponent {
    fn new() -> Self {
        Self {}
    }
}

struct HungerSystem;
impl System for HungerSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in manager.iter_entities(ComponentType::Creature) {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                creature.energy -= HUNGER_RATE;
                if creature.energy <= 0.0 {
                    creature.energy = 0.0;
                }
            }
        }
    }
}

struct ExhaustionSystem;
impl System for ExhaustionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for (arch_index, entity_index, _) in manager.iter_entities(ComponentType::Creature) {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if creature.energy <= 0.0 {
                    creature.health -= EXHAUSTION_RATE;
                }
                if creature.health <= 0.0 {
                    creature.health = 0.0;
                }
            }
        }
    }
}

struct DeathSystem;
impl System for DeathSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        let mut to_remove = Vec::new();
        let mut positions = Vec::new();

        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Creature, ComponentType::Position])
        {
            // Check if the creature should die
            if let Some(creature) = manager.get_component::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if creature.health <= 0.0 {
                    to_remove.push(entity);
                } else {
                    continue;
                }
            }

            // Store the creature's position
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                positions.push(*position);
            }
        }

        // Delete dead creature entities
        to_remove
            .iter()
            .for_each(|entity| manager.remove_entity(*entity));

        // Create a corpse
        for position in positions {
            manager.create_entity_with(&[&CorpseComponent::new(), &position]);
        }
    }
}

struct MoveToFoodSystem;
impl System for MoveToFoodSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all herbivorous entities
        let mut herbivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Herbivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                herbivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest food
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &herbivorous_positions {
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
                        closest_position.insert(*entity, *food_position);
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Move all herbivorous entities in direction of the closest food
        let mut creature_to_food: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Herbivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component_mut::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                if *found.get(&entity).unwrap() {
                    let food_position = closest_position.get(&entity).unwrap();
                    let food_entity = closest_entity.get(&entity).unwrap();
                    let vec_to_food = (food_position.x - position.x, food_position.y - position.y);
                    let norm = (vec_to_food.0.powi(2) + vec_to_food.1.powi(2)).sqrt();
                    if norm < (CREATURE_PIXEL_SIZE as f64 / 2.0 + FOOD_PIXEL_SIZE as f64 / 2.0) {
                        // Food reached -> will go to eating state
                        creature_to_food.insert(entity, *food_entity);
                    } else {
                        // Get closer to the food
                        position.x += vec_to_food.0 / norm * CREATURE_SPEED;
                        position.y += vec_to_food.1 / norm * CREATURE_SPEED;
                    }
                }
            }
        }

        // If food reached, go to eating state
        for (entity, food_entity) in creature_to_food {
            manager.add_component(entity, &EatingFoodComponent::new(food_entity));
        }
    }
}

struct MoveToCorpseSystem;
impl System for MoveToCorpseSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Get the positions of all carnivorous entities
        let mut carnivorous_positions = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Carnivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                carnivorous_positions.insert(entity, *position);
            }
        }

        // For each position, find the closest corpse
        let mut found: HashMap<EntityId, bool> = HashMap::new();
        let mut closest_position: HashMap<EntityId, PositionComponent> = HashMap::new();
        let mut closest_entity: HashMap<EntityId, EntityId> = HashMap::new();
        for (entity, position) in &carnivorous_positions {
            let mut closest_distance_squared = f64::MAX;
            found.insert(*entity, false);
            for (arch_index, entity_index, food_entity) in
                manager.iter_entities_with(&[ComponentType::Corpse, ComponentType::Position])
            {
                if let Some(corpse_position) = manager.get_component::<PositionComponent>(
                    arch_index,
                    entity_index,
                    &ComponentType::Position,
                ) {
                    let distance_squared = (corpse_position.x - position.x).powi(2)
                        + (corpse_position.y - position.y).powi(2);
                    if distance_squared < closest_distance_squared {
                        closest_distance_squared = distance_squared;
                        found.insert(*entity, true);
                        closest_position.insert(*entity, *corpse_position);
                        closest_entity.insert(*entity, food_entity);
                    }
                }
            }
        }

        // Move all carnivorous entities in direction of the closest corpse
        let mut creature_to_corpse: HashMap<EntityId, EntityId> = HashMap::new();
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::Carnivorous, ComponentType::Position])
        {
            if let Some(position) = manager.get_component_mut::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                if *found.get(&entity).unwrap() {
                    let corpse_position = closest_position.get(&entity).unwrap();
                    let corpse_entity = closest_entity.get(&entity).unwrap();
                    let vec_to_corpse = (
                        corpse_position.x - position.x,
                        corpse_position.y - position.y,
                    );
                    let norm = (vec_to_corpse.0.powi(2) + vec_to_corpse.1.powi(2)).sqrt();
                    if norm < (CREATURE_PIXEL_SIZE as f64 / 2.0 + CREATURE_PIXEL_SIZE as f64 / 2.0)
                    {
                        // Corpse reached -> will go to eating state
                        creature_to_corpse.insert(entity, *corpse_entity);
                    } else {
                        // Get closer to the corpse
                        position.x += vec_to_corpse.0 / norm * CREATURE_SPEED;
                        position.y += vec_to_corpse.1 / norm * CREATURE_SPEED;
                    }
                }
            }
        }

        // If corpse reached, go to eating state
        for (entity, corpse_entity) in creature_to_corpse {
            manager.add_component(entity, &EatingCorpseComponent::new(corpse_entity));
        }
    }
}

struct EatFoodSystem;
impl System for EatFoodSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Make sure that a food is not eaten by more than one creature
        let mut food_to_creature: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::EatingFood) {
            if let Some(eating_food) = manager.get_component::<EatingFoodComponent>(
                arch_index,
                entity_index,
                &ComponentType::EatingFood,
            ) {
                food_to_creature.insert(eating_food.food_entity, entity);
            }
            creatures_trying_to_eat.push(entity);
        }

        // Increase energy of creatures that ate a food
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::EatingFood, ComponentType::Creature])
        {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if food_to_creature
                    .values()
                    .any(|&creature_entity| creature_entity == entity)
                {
                    creature.energy += FOOD_ENERGY;
                    if creature.energy > MAX_ENERGY {
                        creature.energy = MAX_ENERGY;
                    }
                }
            }
        }

        // Remove eaten food entities
        for food_entity in food_to_creature.keys() {
            manager.remove_entity(*food_entity);
        }

        // Remove all "eating food" components
        for entity in creatures_trying_to_eat.iter() {
            manager.remove_component(*entity, &ComponentType::EatingFood);
        }
    }
}

struct EatCorpseSystem;
impl System for EatCorpseSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Make sure that a corpse is not eaten by more than one creature
        let mut corpse_to_creature: HashMap<EntityId, EntityId> = HashMap::new();
        let mut creatures_trying_to_eat: Vec<EntityId> = Vec::new();
        for (arch_index, entity_index, entity) in manager.iter_entities(ComponentType::EatingCorpse)
        {
            if let Some(eating_corpse) = manager.get_component::<EatingCorpseComponent>(
                arch_index,
                entity_index,
                &ComponentType::EatingCorpse,
            ) {
                corpse_to_creature.insert(eating_corpse.corpse_entity, entity);
            }
            creatures_trying_to_eat.push(entity);
        }

        // Increase energy of creatures that ate a corpse
        for (arch_index, entity_index, entity) in
            manager.iter_entities_with(&[ComponentType::EatingCorpse, ComponentType::Creature])
        {
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if corpse_to_creature
                    .values()
                    .any(|&creature_entity| creature_entity == entity)
                {
                    creature.energy += CORPSE_ENERGY;
                    if creature.energy > MAX_ENERGY {
                        creature.energy = MAX_ENERGY;
                    }
                }
            }
        }

        // Remove eaten corpse entities
        for corpse_entity in corpse_to_creature.keys() {
            manager.remove_entity(*corpse_entity);
        }

        // Remove all "eating corpse" components
        for entity in creatures_trying_to_eat.iter() {
            manager.remove_component(*entity, &ComponentType::EatingCorpse);
        }
    }
}

struct ReproductionSystem;
impl System for ReproductionSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        // Find creatures that can reproduce
        let mut positions = Vec::new();
        let mut is_herbivorous = Vec::new();
        for (arch_index, entity_index, _entity) in
            manager.iter_entities_with(&[ComponentType::Creature, ComponentType::Position])
        {
            // If the creature can reproduce, reset its energy to start value
            if let Some(creature) = manager.get_component_mut::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                if creature.energy >= REPROD_ENERGY_THRESHOLD {
                    creature.energy -= REPROD_ENERGY_COST;
                } else {
                    continue;
                }
            }

            // Store the creature's position
            if let Some(position) = manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                positions.push(*position);
            } else {
                continue;
            }

            // Check if herbivorous or carnivorous
            is_herbivorous.push(manager.has_component(arch_index, &ComponentType::Herbivorous));
        }

        // Create one new creature next to each reproducing create
        for (position, is_h) in positions.iter().zip(is_herbivorous) {
            if is_h {
                manager.create_entity_with(&[
                    &CreatureComponent::new(),
                    &HerbivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                ]);
            } else {
                manager.create_entity_with(&[
                    &CreatureComponent::new(),
                    &CarnivorousComponent::new(),
                    &PositionComponent::from(position.x + CREATURE_PIXEL_SIZE as f64, position.y),
                ]);
            }
        }
    }
}

struct FoodGrowthSystem;
impl System for FoodGrowthSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for _ in 0..NEW_FOOD_PER_TICK {
            manager.create_entity_with(&[&FoodComponent::new(), &PositionComponent::new()]);
        }
    }
}

pub struct World {
    archetype_manager: ArchetypeManager,
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

    pub fn create_entity_with(&mut self, components: &[&dyn Component]) {
        self.archetype_manager.create_entity_with(components);
    }

    pub fn iterate(&mut self) {
        for s in &self.systems {
            s.run(&mut self.archetype_manager);
        }
    }

    pub fn draw(&mut self, pixels: &mut [u8], window_width: u32, window_height: u32) {
        // Background
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0xcc, 0xcc, 0xcc, 0xff]);
        }

        // Draw corpses
        for (arch_index, entity_index, _) in self
            .archetype_manager
            .iter_entities_with(&[ComponentType::Corpse, ComponentType::Position])
        {
            if let Some(position) = self.archetype_manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                self.draw_square(
                    position,
                    CORPSE_COLOR,
                    CREATURE_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }

        // Draw creatures
        for (arch_index, entity_index, _) in self
            .archetype_manager
            .iter_entities_with(&[ComponentType::Creature, ComponentType::Position])
        {
            // Check what kind of creature this is
            let color = if self
                .archetype_manager
                .has_component(arch_index, &ComponentType::Herbivorous)
            {
                HERBIVOROUS_COLOR
            } else {
                CARNIVOROUS_COLOR
            };
            let pos;
            if let Some(position) = self.archetype_manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                pos = *position;
                self.draw_square(
                    position,
                    color,
                    CREATURE_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            } else {
                continue;
            }

            if let Some(creature) = self.archetype_manager.get_component::<CreatureComponent>(
                arch_index,
                entity_index,
                &ComponentType::Creature,
            ) {
                // Draw health bar
                self.draw_rec(
                    (
                        pos.x,
                        pos.y + CREATURE_PIXEL_SIZE as f64 / 2.0 + BAR_HEIGHT as f64 / 2.0 + 5.0,
                    ),
                    HEALTH_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.health / MAX_HEALTH) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                    window_width,
                    window_height,
                );

                // Draw energy bar
                self.draw_rec(
                    (
                        pos.x,
                        pos.y
                            + CREATURE_PIXEL_SIZE as f64 / 2.0
                            + BAR_HEIGHT as f64 * 1.5
                            + 5.0 * 2.0,
                    ),
                    ENERGY_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.energy / MAX_ENERGY) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }

        // Draw food
        for (arch_index, entity_index, _) in self
            .archetype_manager
            .iter_entities_with(&[ComponentType::Food, ComponentType::Position])
        {
            if let Some(position) = self.archetype_manager.get_component::<PositionComponent>(
                arch_index,
                entity_index,
                &ComponentType::Position,
            ) {
                self.draw_square(
                    position,
                    FOOD_COLOR,
                    FOOD_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }
    }

    fn draw_square(
        &self,
        position: &PositionComponent,
        color: &[u8],
        size: u32,
        pixels: &mut [u8],
        window_width: u32,
        window_height: u32,
    ) {
        self.draw_rec(
            (position.x, position.y),
            color,
            (size, size),
            pixels,
            window_width,
            window_height,
        );
    }

    fn draw_rec(
        &self,
        (x, y): (f64, f64),
        color: &[u8],
        (width, height): (u32, u32),
        pixels: &mut [u8],
        window_width: u32,
        window_height: u32,
    ) {
        let pos_in_window = (
            x + (window_width as f64) / 2.0,
            -y + (window_height as f64) / 2.0,
        );
        let w = width as i64 / 2;
        let h = height as i64 / 2;
        for i in -w..w {
            for j in -h..h {
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

struct App<'window> {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'window>>,
    world: World,
}
impl<'window> Default for App<'window> {
    fn default() -> Self {
        let mut world = World::new();

        for _ in 0..FOOD_NB {
            world.create_entity_with(&[&FoodComponent::new(), &PositionComponent::new()]);
        }

        for _ in 0..HERBIVOROUS_NB {
            world.create_entity_with(&[
                &CreatureComponent::new(),
                &PositionComponent::new(),
                &HerbivorousComponent::new(),
            ]);
        }

        for _ in 0..CARNIVOROUS_NB {
            world.create_entity_with(&[
                &CreatureComponent::new(),
                &PositionComponent::new(),
                &CarnivorousComponent::new(),
            ]);
        }

        for _ in 0..CORPSE_NB {
            world.create_entity_with(&[&CorpseComponent::new(), &PositionComponent::new()]);
        }

        world.add_system(Box::new(HungerSystem));
        world.add_system(Box::new(ExhaustionSystem));
        world.add_system(Box::new(DeathSystem));
        world.add_system(Box::new(MoveToFoodSystem));
        world.add_system(Box::new(EatFoodSystem));
        world.add_system(Box::new(MoveToCorpseSystem));
        world.add_system(Box::new(EatCorpseSystem));
        world.add_system(Box::new(ReproductionSystem));
        world.add_system(Box::new(FoodGrowthSystem));

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

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut App::default()).unwrap();
}
