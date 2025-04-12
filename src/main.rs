use std::{any::{type_name, Any}, collections::HashSet};

trait Component: Any {}
impl dyn Component {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Copy)]
struct HelloComponent;
impl Component for HelloComponent {}
impl HelloComponent {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Copy)]
struct ValueComponent {
    value: i32
}
impl Component for ValueComponent {}
impl ValueComponent {
    fn new(value: i32) -> Self {
        Self {
            value
        }
    }
}

enum ComponentType {
    Hello = 0,
    Value,
    MAX
}

type EntityId = usize;
type ComponentIndex = usize;

#[derive(Clone)]
struct Entity {
    id: EntityId,
    components: Vec<ComponentIndex>
}

impl Entity {
    fn new(id: EntityId) -> Self {
        Self {
            id,
            components: vec![usize::MAX; ComponentType::MAX as usize]
        }
    }

    fn add_component(&mut self, comp_type: ComponentType, comp_index: ComponentIndex) {
        self.components[comp_type as usize] = comp_index;
    }
}

trait SystemRunner {
    fn run(&self, comps: &mut ComponentManager, entities: &Vec<Entity>, subjects: &HashSet<EntityId>);
    fn accepts(&self, entity: &Entity) -> bool;
}

struct IncrementSytemRunner;
impl SystemRunner for IncrementSytemRunner {
    fn run(&self, comps: &mut ComponentManager, entities: &Vec<Entity>, subjects: &HashSet<EntityId>) {
        for id in subjects {
            let comp_index = entities[*id].components[ComponentType::Value as usize];
            comps.value_components[comp_index].value += 1;
            println!("Value {} from {}", comps.value_components[comp_index].value, id);
        }
    }

    fn accepts(&self, entity: &Entity) -> bool {
        entity.components[ComponentType::Value as usize] != usize::MAX
    }
}

struct HelloSystemRunner;
impl SystemRunner for HelloSystemRunner {
    fn run(&self, _comps: &mut ComponentManager, _entities: &Vec<Entity>, subjects: &HashSet<EntityId>) {
        for id in subjects {
            println!("Hello from {}", id);
        }
    }

    fn accepts(&self, entity: &Entity) -> bool { 
        entity.components[ComponentType::Hello as usize] != usize::MAX
    }
}

struct System {
    subjects: HashSet<EntityId>,
    runner: Box<dyn SystemRunner>
}

impl System {
    fn new(runner: Box<dyn SystemRunner>) -> Self {
        Self {
            subjects: HashSet::new(),
            runner
        }
    }

    fn run(&self, comps: &mut ComponentManager, entities: &Vec<Entity>) {
        self.runner.run(comps, entities, &self.subjects);
    }

    fn propose_subject(&mut self, entity: &Entity) {
        if self.runner.accepts(entity) {
            self.subjects.insert(entity.id);
        }
    }
}

struct ComponentManager {
    hello_components: Vec<HelloComponent>,
    value_components: Vec<ValueComponent>,
}

impl ComponentManager {
    fn new() -> Self {
        Self {
            hello_components: Vec::new(),
            value_components: Vec::new()
        }
    }
}

struct World {
    entities: Vec<Entity>,
    component_manager: ComponentManager,
    systems: Vec<System>,
}

impl World {
    fn new() -> Self {
        let systems = vec![
            System::new(Box::new(IncrementSytemRunner)),
            System::new(Box::new(HelloSystemRunner))
        ];
        Self {
            entities: Vec::new(),
            component_manager: ComponentManager::new(),
            systems
        }
    }

    fn add_component(&mut self, entity_id: EntityId, component: &dyn Component) {
        if self.entities.len() <= entity_id {
            self.entities.push(Entity::new(self.entities.len()));
        }
        let entity = &mut self.entities[entity_id];

        if let Some(value_c) = component.as_any().downcast_ref::<ValueComponent>() {
            entity.add_component(ComponentType::Value, self.component_manager.value_components.len());
            self.component_manager.value_components.push(*value_c);
        }
        else if let Some(value_c) = component.as_any().downcast_ref::<HelloComponent>() {
            entity.add_component(ComponentType::Hello, self.component_manager.hello_components.len());
            self.component_manager.hello_components.push(*value_c);
        }
        else {
            panic!("Unknown component type {}", type_name::<&dyn Component>());
        }
        self.systems.iter_mut().for_each(|s| s.propose_subject(&self.entities[entity_id]));
    }

    fn run(&mut self) {
        for s in &self.systems {
            s.run(&mut self.component_manager, &self.entities);
        }
    }
}

fn main() {
    let mut world = World::new();
    for i in 0..10 {
        world.add_component(i, &ValueComponent::new((i as i32) * 100));
    }
    for i in 0..10 {
        world.add_component(i + 10, &HelloComponent::new());
    }
    world.run();
}
