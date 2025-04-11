use std::collections::HashMap;

#[derive(Clone)]
enum Component {
    Talkative,
    Value(i32)
}

type EntityId = u64; 
struct Entity {
    components: Vec<Component>
}
impl Entity {
    fn new(comps: &[Component]) -> Self {
        Self {
            components: comps.to_vec()
        }
    }
}

trait SystemRunner {
    fn run(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>);

    fn run_for_one(&self, entity: &mut Entity);

    fn for_all(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>)
    {
        for id in subjects {
            if let Some(entity) = entities.get_mut(id) {
                self.run_for_one(entity);
            }
        }
    }

    fn accepts(&self, entity: &Entity) -> bool;
}

struct IncrementSytemRunner;
impl SystemRunner for IncrementSytemRunner {
    fn run(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>) {
        self.for_all(entities, subjects);
    }

    fn run_for_one(&self, entity: &mut Entity) {
        for c in &mut entity.components {
            match c {
                Component::Value(v) => { *v += 1; println!("{}", v) },
                _ => {}
            }
        }
    }

    fn accepts(&self, entity: &Entity) -> bool {
        entity.components.iter().any(|comp| matches!(comp, Component::Value(_)))
    }
}

struct HelloSytemRunner;
impl SystemRunner for HelloSytemRunner {
    fn run(&self, entities: &mut HashMap<EntityId, Entity>, subjects: &Vec<EntityId>) {
        self.for_all(entities, subjects);
    }

    fn run_for_one(&self, entity: &mut Entity) {
        for c in &mut entity.components {
            match c {
                Component::Talkative => { println!("Hello") },
                _ => {}
            }
        }
    }

    fn accepts(&self, entity: &Entity) -> bool { 
        entity.components.iter().any(|comp| matches!(comp, Component::Talkative))
    }
}

struct System {
    subjects: Vec<EntityId>,
    runner: Box<dyn SystemRunner>
}

impl System {
    fn new(runner: Box<dyn SystemRunner>) -> Self {
        Self {
            subjects: Vec::new(),
            runner
        }
    }

    fn run(&self, entities: &mut HashMap<EntityId, Entity>) {
        self.runner.run(entities, &self.subjects);
    }

    fn update_subjects(&mut self, entities: &HashMap<EntityId, Entity>) {
        self.subjects = entities.iter()
            .filter(|&(_, e)| self.runner.accepts(e))
            .map(|(id, _)| *id)
            .collect();
    }
}

struct World {
    entities: HashMap<EntityId, Entity>,
    next_entity_id: EntityId,
    systems: Vec<System>,
}

impl World {
    fn new() -> Self {
        let systems = vec![
            System::new(Box::new(IncrementSytemRunner)),
            System::new(Box::new(HelloSytemRunner))
        ];
        Self {
            entities: HashMap::new(),
            next_entity_id: 0,
            systems
        }
    }

    fn run(&mut self) {
        for s in &self.systems {
            s.run(&mut self.entities);
        }
    }

    fn add_entity(&mut self, entity: Entity) {
        self.entities.insert(self.next_entity_id, entity); 
        self.systems.iter_mut().for_each(|s| s.update_subjects(&self.entities));
        self.next_entity_id += 1;
    }
}

fn main() {
    let mut world = World::new();
    world.add_entity(Entity::new(&[Component::Value(0)]));
    world.add_entity(Entity::new(&[Component::Value(1000), Component::Talkative]));
    world.add_entity(Entity::new(&[Component::Talkative]));

    world.run();
}
