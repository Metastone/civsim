use std::{any::Any, collections::{HashMap, HashSet}};

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum ComponentType {
    Value = 0,
    Hello
}

pub trait Component: Any + CloneComponent {
    fn get_type(&self) -> ComponentType;
}

impl dyn Component {
    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait CloneComponent {
    fn clone_box(&self) -> Box<dyn Component>;
}

impl<T> CloneComponent for T
where
    T: 'static + Component + Clone,
{
    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

type EntityId = usize;

struct Archetype {
    component_types: HashSet<ComponentType>,
    data: HashMap<ComponentType, Vec<Box<dyn Component>>>,
    entities: Vec<EntityId>
}

impl Archetype {
    fn new(component_types: HashSet<ComponentType>) -> Self {
        let data = component_types.iter().map(|t| (*t, Vec::new())).collect();
        Self {
            component_types,
            data,
            entities: Vec::new()
        }
    }
}

pub trait System {
    fn run(&self, manager: &mut ArchetypeManager);
}

pub struct ArchetypeManager {
    archetypes: Vec<Archetype>
}

impl ArchetypeManager {
    fn new() -> Self {
        Self {
            archetypes: Vec::new()
        }
    }

    pub fn with(&self, required_ctypes: HashSet<ComponentType>) -> Vec<usize> {
        self.archetypes.iter().enumerate()
            .filter(|(_, a)| a.component_types.is_superset(&required_ctypes))
            .map(|(i, _)| i).collect()
    }

    pub fn get_components(&mut self, arch_index: usize, comp_type: &ComponentType) -> &mut Vec<Box<dyn Component>> {
        let data = &mut self.archetypes[arch_index].data;
        data.get_mut(comp_type).unwrap()
    }

    fn add_component(&mut self, entity_id: EntityId, new_comp: &dyn Component) {
        // Find in which archetype is the entity
        let mut entity_found = false;
        let mut archetype_id = 0;
        let mut entity_index = 0;
        for (a_index, archetype) in self.archetypes.iter().enumerate() {
            if let Some((e_index, _)) = archetype.entities.iter().enumerate().find(|(_, e_id)| entity_id == **e_id) {
                entity_found = true;
                archetype_id = a_index;
                entity_index = e_index;
                break;
            }
        }

        let mut cur_ctypes = HashSet::new();
        let mut required_ctypes = HashSet::from([new_comp.get_type()]);
        let mut cur_comps = Vec::new();

        if entity_found {
            // If the archetype has the component, replace it
            let archetype = &mut self.archetypes[archetype_id];
            if archetype.component_types.contains(&new_comp.get_type()) {
                archetype.data.get_mut(&new_comp.get_type()).unwrap()[entity_index] = new_comp.clone_box();
                return;
            }

            // Get already existing components associated to this entity
            cur_ctypes = cur_ctypes.union(&archetype.component_types).cloned().collect();
            required_ctypes = required_ctypes.union(&archetype.component_types).cloned().collect();
            let mut cur_comps_temp: Vec<Box<dyn Component>> = cur_ctypes.iter()
                .map(|ctype| archetype.data.get(ctype).unwrap()[entity_index].clone_box())
                .collect();
            cur_comps.append(&mut cur_comps_temp);

            // Remove entity from old archetype
            cur_ctypes.iter().for_each(|ctype| { archetype.data.get_mut(ctype).unwrap().remove(entity_index); });
            archetype.entities.remove(entity_index);
        }

        // If we find an archetype that has exactly the required components, move the entity to it
        if let Some(new_archetype) = self.archetypes.iter_mut().find(|a| a.component_types == required_ctypes) {
            Self::copy_entity_to_new_arch(new_archetype, cur_comps, new_comp, entity_id);
        }
        // Otherwise, create a new archetype and move the entity to it
        else {
            let mut new_archetype = Archetype::new(required_ctypes.clone());
            Self::copy_entity_to_new_arch(&mut new_archetype, cur_comps, new_comp, entity_id);
            self.archetypes.push(new_archetype);
        }
    }

    fn copy_entity_to_new_arch(new_archetype: &mut Archetype, cur_comps: Vec<Box<dyn Component>>, new_comp: &dyn Component, entity_id: EntityId) {
        for old_c in cur_comps {
            new_archetype.data.get_mut(&old_c.get_type()).unwrap().push(old_c);
        }
        new_archetype.data.get_mut(&new_comp.get_type()).unwrap().push(new_comp.clone_box());
        new_archetype.entities.push(entity_id);
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

    pub fn run(&mut self) {
        for s in &self.systems {
            s.run(&mut self.archetype_manager);
        }
    }
}
