use std::{any::{type_name, Any}, cell::RefCell, rc::Rc};

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
trait HasHello {
    fn hellos(&mut self) -> &mut [HelloComponent];
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
trait HasValue {
    fn values(&mut self) -> &mut [ValueComponent];
}

type EntityId = usize;

struct ArchetypeHV {
    hellos: Vec<HelloComponent>,
    values: Vec<ValueComponent>,
    entities: Vec<EntityId>
}
impl ArchetypeHV {
    fn new() -> Self {
        Self {
            hellos: Vec::new(),
            values: Vec::new(),
            entities: Vec::new()
        }
    }
}
impl HasHello for ArchetypeHV {
    fn hellos(&mut self) -> &mut [HelloComponent] {
        &mut self.hellos
    }
}
impl HasValue for ArchetypeHV {
    fn values(&mut self) -> &mut [ValueComponent] {
        &mut self.values
    }
}

struct ArchetypeH {
    hellos: Vec<HelloComponent>,
    entities: Vec<EntityId>
}
impl HasHello for ArchetypeH {
    fn hellos(&mut self) -> &mut [HelloComponent] {
        &mut self.hellos
    }
}
impl ArchetypeH {
    fn new() -> Self {
        Self {
            hellos: Vec::new(),
            entities: Vec::new()
        }
    }
}

struct ArchetypeV {
    values: Vec<ValueComponent>,
    entities: Vec<EntityId>
}
impl HasValue for ArchetypeV {
    fn values(&mut self) -> &mut [ValueComponent] {
        &mut self.values
    }
}
impl ArchetypeV {
    fn new() -> Self {
        Self {
            values: Vec::new(),
            entities: Vec::new()
        }
    }
}

trait System {
    fn run(&self, archetypes: &mut ArchetypeManager);
}

struct IncrementSytem;
impl System for IncrementSytem {
    fn run(&self, archetypes: &mut ArchetypeManager) {
        for arch in archetypes.has_value_archs.iter_mut() {
            for value_comp in arch.borrow_mut().values().iter_mut() {
                value_comp.value += 1;
                println!("Value {}", value_comp.value);
            }
        }
    }
}

struct HelloSystem;
impl System for HelloSystem {
    fn run(&self, archetypes: &mut ArchetypeManager) {
        for arch in archetypes.has_hello_archs.iter_mut() {
            for _i in 0..arch.borrow_mut().hellos().len() {
                println!("Hello");
            }
        }
    }
}

struct ArchetypeManager {
    hello: Rc<RefCell<ArchetypeH>>,
    value: Rc<RefCell<ArchetypeV>>,
    hello_value: Rc<RefCell<ArchetypeHV>>,
    has_hello_archs: Vec<Rc<RefCell<dyn HasHello>>>,
    has_value_archs: Vec<Rc<RefCell<dyn HasValue>>>,
}

impl ArchetypeManager {
    fn new() -> Self {
        let hello = Rc::new(RefCell::new(ArchetypeH::new()));
        let value = Rc::new(RefCell::new(ArchetypeV::new()));
        let hello_value = Rc::new(RefCell::new(ArchetypeHV::new()));
        Self {
            hello: Rc::clone(&hello),
            value: Rc::clone(&value),
            hello_value: Rc::clone(&hello_value),
            has_hello_archs: vec![
                hello as Rc<RefCell<dyn HasHello>>,
                Rc::clone(&hello_value) as Rc<RefCell<dyn HasHello>>
            ],
            has_value_archs: vec![
                value as Rc<RefCell<dyn HasValue>>,
                hello_value as Rc<RefCell<dyn HasValue>>
            ]
        }
    }

    fn add_component(&mut self, entity_id: EntityId, component: &dyn Component) {
        if let Some(value_c) = component.as_any().downcast_ref::<ValueComponent>() {
            // Already in good archetype, replace component
            if let Some((i, _)) = self.value.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.value.borrow_mut().values[i] = *value_c;
                return;
            }
            // In archetype without the component, move to right archetype
            if let Some((i, id)) = self.hello.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.hello_value.borrow_mut().hellos.push(self.hello.borrow().hellos[i]);
                self.hello_value.borrow_mut().values.push(*value_c);
                self.hello_value.borrow_mut().entities.push(*id);
                self.hello.borrow_mut().hellos.remove(i);
                self.hello.borrow_mut().entities.remove(i);
                return;
            }
            // In archetype with the component among others, replace component
            if let Some((i, _)) = self.hello_value.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.hello_value.borrow_mut().values[i] = *value_c;
                return;
            }
            // Entity does not exist, create it
            self.value.borrow_mut().values.push(*value_c);
            self.value.borrow_mut().entities.push(entity_id);
        }
        else if let Some(hello_c) = component.as_any().downcast_ref::<HelloComponent>() {
            // Already in good archetype, replace component
            if let Some((i, _)) = self.hello.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.hello.borrow_mut().hellos[i] = *hello_c;
                return;
            }
            // In archetype without the component, move to right archetype
            if let Some((i, id)) = self.value.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.hello_value.borrow_mut().values.push(self.value.borrow().values[i]);
                self.hello_value.borrow_mut().hellos.push(*hello_c);
                self.hello_value.borrow_mut().entities.push(*id);
                self.value.borrow_mut().values.remove(i);
                self.value.borrow_mut().entities.remove(i);
                return;
            }
            // In archetype with the component among others, replace component
            if let Some((i, _)) = self.hello_value.borrow().entities.iter().enumerate().find(|(_, id)| **id == entity_id) {
                self.hello_value.borrow_mut().hellos[i] = *hello_c;
                return;
            }
            // Entity does not exist, create it
            self.hello.borrow_mut().hellos.push(*hello_c);
            self.hello.borrow_mut().entities.push(entity_id);
        }
        else {
            panic!("Unknown component type {}", type_name::<&dyn Component>());
        }
    }
}

struct World {
    archetype_manager: ArchetypeManager,
    systems: Vec<Box<dyn System>>,
}

impl World {
    fn new() -> Self {
        let systems: Vec<Box<dyn System>> = vec![
            Box::new(IncrementSytem),
            Box::new(HelloSystem)
        ];
        Self {
            archetype_manager: ArchetypeManager::new(),
            systems
        }
    }

    fn run(&mut self) {
        for s in &self.systems {
            s.run(&mut self.archetype_manager);
        }
    }
}

fn main() {
    let mut world = World::new();
    for i in 0..10 {
        world.archetype_manager.add_component(i, &ValueComponent::new((i as i32) * 100));
    }
    for i in 0..10 {
        world.archetype_manager.add_component(i + 10, &HelloComponent::new());
    }
    world.run();
}
