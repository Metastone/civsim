use crate::constants::*;
use crate::ecs::{Component, ComponentType, EntityId};

#[derive(Clone, Copy)]
pub struct CreatureComponent {
    pub energy: f32,
    pub health: f32,
}
impl Component for CreatureComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Creature
    }
}
impl CreatureComponent {
    pub fn new() -> Self {
        Self {
            energy: START_ENERGY,
            health: MAX_HEALTH,
        }
    }
}

#[derive(Clone, Copy)]
pub struct PositionComponent {
    pub x: f64,
    pub y: f64,
}
impl Component for PositionComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Position
    }
}
impl PositionComponent {
    pub fn new() -> Self {
        let x = rand::random_range((SCREEN_WIDTH as f64 / -2.0)..(SCREEN_WIDTH as f64 / 2.0));
        let y = rand::random_range((SCREEN_HEIGHT as f64 / -2.0)..(SCREEN_HEIGHT as f64 / 2.0));
        Self { x, y }
    }

    pub fn from(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct FoodComponent {}
impl Component for FoodComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Food
    }
}
impl FoodComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct EatingFoodComponent {
    pub food_entity: EntityId,
}
impl Component for EatingFoodComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::EatingFood
    }
}
impl EatingFoodComponent {
    pub fn new(food_entity: EntityId) -> Self {
        Self { food_entity }
    }
}

#[derive(Clone, Copy)]
pub struct EatingCorpseComponent {
    pub corpse_entity: EntityId,
}
impl Component for EatingCorpseComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::EatingCorpse
    }
}
impl EatingCorpseComponent {
    pub fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone, Copy)]
pub struct AttackingHerbivorousComponent {
    pub herbivorous_entity: EntityId,
}
impl Component for AttackingHerbivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::AttackingHerbivorous
    }
}
impl AttackingHerbivorousComponent {
    pub fn new(herbivorous_entity: EntityId) -> Self {
        Self { herbivorous_entity }
    }
}

#[derive(Clone, Copy)]
pub struct CorpseComponent;
impl Component for CorpseComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Corpse
    }
}
impl CorpseComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct HerbivorousComponent {}
impl Component for HerbivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Herbivorous
    }
}
impl HerbivorousComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct CarnivorousComponent {}
impl Component for CarnivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Carnivorous
    }
}
impl CarnivorousComponent {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
pub struct MoveToCorpseComponent {
    pub corpse_entity: EntityId,
}
impl Component for MoveToCorpseComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::MoveToCorpse
    }
}
impl MoveToCorpseComponent {
    pub fn new(corpse_entity: EntityId) -> Self {
        Self { corpse_entity }
    }
}

#[derive(Clone, Copy)]
pub struct MoveToFoodComponent {
    pub food_entity: EntityId,
}
impl Component for MoveToFoodComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::MoveToFood
    }
}
impl MoveToFoodComponent {
    pub fn new(food_entity: EntityId) -> Self {
        Self { food_entity }
    }
}

#[derive(Clone, Copy)]
pub struct MoveToHerbivorousComponent {
    pub herbivorous_entity: EntityId,
}
impl Component for MoveToHerbivorousComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::MoveToHerbivorous
    }
}
impl MoveToHerbivorousComponent {
    pub fn new(herbivorous_entity: EntityId) -> Self {
        Self { herbivorous_entity }
    }
}

#[derive(Clone, Copy)]
pub struct InactiveComponent {}
impl Component for InactiveComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Inactive
    }
}
impl InactiveComponent {
    pub fn new() -> Self {
        Self {}
    }
}
