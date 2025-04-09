use std::collections::HashMap;

enum Component {
    Empty,
    Value(i32)
}

type EntityId = u64; 
struct Entity {
    id: EntityId,
    components: Vec<Component>
}
impl Entity {
    fn new(id: EntityId, component: Component) -> Self {
        Self {
            id,
            components: vec![component]
        }
    }
}

trait SystemRunner {
    fn run(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>);
}

struct IncrementSytemRunner;
impl SystemRunner for IncrementSytemRunner { 
    fn run(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>) {
        for id in subjects {
            if let Some(entity) = entities.get_mut(id) {
                for c in &mut entity.components {
                    match c {
                        Component::Value(v) => { *v += 1; println!("{}", v) },
                        _ => {}
                    }
                }
            }
        }
    }
}

struct System {
    subjects: Vec<EntityId>,
    runner: Box<dyn SystemRunner>
}

impl System {
    fn new(runner: Box<dyn SystemRunner>) -> Self {
        Self {
            subjects: vec![0],
            runner
        }
    }
    fn run(&self, entities: &mut HashMap<EntityId, Entity>) {
        self.runner.run(entities, &self.subjects);
    }
}

struct World {
    entities: HashMap<EntityId, Entity>,
    systems: Vec<System>,
}

impl World {
    fn new() -> Self {
        let mut entities = HashMap::new();
        let id = 0;
        entities.insert(id, Entity::new(id, Component::Value(0)));

        Self {
            entities,
            systems: vec![System::new(Box::new(IncrementSytemRunner))]
        }
    }

    fn run(&mut self) {
        for s in &self.systems {
            s.run(&mut self.entities);
        }
    }
}

fn main() {
    let mut world = World::new();
    world.run();
}
