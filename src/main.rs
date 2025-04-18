use std::collections::HashSet;

use ecs::{ArchetypeManager, Component, ComponentType, System, World};

mod ecs;

#[derive(Clone, Copy)]
struct HelloComponent;
impl Component for HelloComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Hello
    }
}
impl HelloComponent {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
struct ValueComponent {
    value: i32
}
impl Component for ValueComponent {
    fn get_type(&self) -> ComponentType {
        ComponentType::Value
    }
}
impl ValueComponent {
    fn new(value: i32) -> Self {
        Self {
            value
        }
    }
}

struct IncrementSytem;
impl System for IncrementSytem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with(HashSet::from([ComponentType::Value])) {
            for component in manager.get_components(arch_index, &ComponentType::Value).iter_mut() {
                if let Some(value_comp) = component.as_any_mut().downcast_mut::<ValueComponent>() {
                    value_comp.value += 1;
                    println!("Value {}", value_comp.value);
                }
            }
        }
    }
}

struct HelloSystem;
impl System for HelloSystem {
    fn run(&self, manager: &mut ArchetypeManager) {
        for arch_index in manager.with(HashSet::from([ComponentType::Hello])) {
            for _ in 0..manager.get_components(arch_index, &ComponentType::Hello).len() {
                println!("Hello");
            }
        }
    }
}

fn main() {
    let mut world = World::new();
    for i in 0..3 {
        world.add_component(i, &ValueComponent::new((i as i32) * 100));
    }
    for i in 0..3 {
        world.add_component(i + 3, &HelloComponent::new());
    }
    for i in 0..3 {
        world.add_component(i + 6, &ValueComponent::new((i as i32) * 100));
        world.add_component(i + 6, &ValueComponent::new((i as i32) * 100 + 50));
        world.add_component(i + 6, &HelloComponent::new());
        world.add_component(i + 6, &HelloComponent::new());
    }
    world.add_system(Box::new(HelloSystem));
    world.add_system(Box::new(IncrementSytem));
    world.run();
}
