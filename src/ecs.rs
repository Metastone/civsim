use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use log::error;

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum ComponentType {
    Person = 0,
    Position,
    Food,
    Behavior,
    EatingFood,
}

pub trait Component: Any + CloneComponent {
    fn get_type(&self) -> ComponentType;
}

impl dyn Component {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
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

pub type EntityId = usize;

pub struct EntityIdAllocator {
    next_id: EntityId,
}
impl EntityIdAllocator {
    pub fn new() -> Self {
        EntityIdAllocator { next_id: 0 }
    }

    pub fn get_next_id(&mut self) -> usize {
        let ret = self.next_id;
        self.next_id += 1;
        ret
    }
}

pub struct Archetype {
    component_types: HashSet<ComponentType>,
    data: HashMap<ComponentType, Vec<Box<dyn Component>>>,
    entities: Vec<EntityId>,
}

impl Archetype {
    fn new(component_types: HashSet<ComponentType>) -> Self {
        let data = component_types.iter().map(|t| (*t, Vec::new())).collect();
        Self {
            component_types,
            data,
            entities: Vec::new(),
        }
    }
}

type ArchetypeEntityInfo = Vec<EntityId>;

pub struct EntityIterator {
    entity_index: usize,
    i_arch: usize,
    archetype_indexes: Vec<usize>, // Only refers to archetypes not empty
    archetype_entities: Vec<ArchetypeEntityInfo>,
}

impl EntityIterator {
    pub fn new(
        archetype_manager: &ArchetypeManager,
        required_ctypes: HashSet<ComponentType>,
    ) -> Self {
        EntityIterator {
            entity_index: 0,
            i_arch: 0,
            archetype_indexes: archetype_manager.with_comps(required_ctypes),
            archetype_entities: archetype_manager.get_archetype_entity_info(),
        }
    }
}

impl Iterator for EntityIterator {
    type Item = (usize, usize, EntityId);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i_arch >= self.archetype_indexes.len() {
            return None;
        }
        let mut arch_index = self.archetype_indexes[self.i_arch];

        let mut entities = &self.archetype_entities[arch_index];

        // If we finished iterating over the current archertype, continue in next one (assume it's not empty)
        if self.entity_index >= entities.len() {
            self.entity_index = 0;
            self.i_arch += 1;
            if self.i_arch >= self.archetype_indexes.len() {
                return None;
            }
            arch_index = self.archetype_indexes[self.i_arch];
            entities = &self.archetype_entities[arch_index];
        }

        // Get entity reference info to return
        let entity = entities[self.entity_index];
        let entity_ref = Some((arch_index, self.entity_index, entity));
        self.entity_index += 1;

        entity_ref
    }
}

pub trait System {
    fn run(&self, manager: &mut ArchetypeManager);
}

pub struct ArchetypeManager {
    archetypes: Vec<Archetype>,
}

impl Default for ArchetypeManager {
    fn default() -> Self {
        ArchetypeManager::new()
    }
}

impl ArchetypeManager {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
        }
    }

    pub fn get_component<C>(
        &self,
        arch_index: usize,
        entity_index: usize,
        ctype: &ComponentType,
    ) -> Option<&C>
    where
        C: Component,
    {
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            None
        } else if let Some(components) = self.archetypes[arch_index].data.get(ctype) {
            components[entity_index].as_any().downcast_ref::<C>()
        } else {
            None
        }
    }

    pub fn get_component_mut<C>(
        &mut self,
        arch_index: usize,
        entity_index: usize,
        ctype: &ComponentType,
    ) -> Option<&mut C>
    where
        C: Component,
    {
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            None
        } else if let Some(components) = self.archetypes[arch_index].data.get_mut(ctype) {
            components[entity_index].as_any_mut().downcast_mut::<C>()
        } else {
            None
        }
    }

    fn with_comps(&self, required_ctypes: HashSet<ComponentType>) -> Vec<usize> {
        self.archetypes
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                a.component_types.is_superset(&required_ctypes) && !a.entities.is_empty()
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn get_archetype_entity_info(&self) -> Vec<ArchetypeEntityInfo> {
        self.archetypes.iter().map(|a| a.entities.clone()).collect()
    }

    pub fn iter_entities(&self, required_ctype: ComponentType) -> EntityIterator {
        self.iter_entities_with_types(HashSet::from([required_ctype]))
    }

    pub fn iter_entities_with(&self, required_ctypes: &[ComponentType]) -> EntityIterator {
        self.iter_entities_with_types(required_ctypes.iter().copied().collect())
    }

    fn iter_entities_with_types(&self, required_ctypes: HashSet<ComponentType>) -> EntityIterator {
        EntityIterator::new(self, required_ctypes)
    }

    pub fn remove_entity(&mut self, entity: usize) {
        let mut entity_found = false;

        // Look in all archetypes to find the entity
        for archetype in self.archetypes.iter_mut() {
            if let Some((entity_index, _)) = archetype
                .entities
                .iter()
                .enumerate()
                .find(|(_, e_id)| entity == **e_id)
            {
                // If found, remove entity from archetype
                entity_found = true;
                //let cur_ctypes = &archetype.component_types.clone();
                archetype.component_types.iter().for_each(|ctype| {
                    archetype.data.get_mut(ctype).unwrap().remove(entity_index);
                });
                archetype.entities.remove(entity_index);
                break;
            }
        }

        if !entity_found {
            error!("Cannot remove entity {entity}: Entity does not exist");
        }
    }

    pub fn add_component(&mut self, entity: EntityId, new_comp: &dyn Component) {
        // Find in which archetype is the entity
        let mut entity_found = false;
        let mut archetype_id = 0;
        let mut entity_index = 0;
        for (a_index, archetype) in self.archetypes.iter().enumerate() {
            if let Some((e_index, _)) = archetype
                .entities
                .iter()
                .enumerate()
                .find(|(_, e_id)| entity == **e_id)
            {
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
                archetype.data.get_mut(&new_comp.get_type()).unwrap()[entity_index] =
                    new_comp.clone_box();
                return;
            }

            // Get already existing components associated to this entity
            cur_ctypes = cur_ctypes
                .union(&archetype.component_types)
                .cloned()
                .collect();
            required_ctypes = required_ctypes
                .union(&archetype.component_types)
                .cloned()
                .collect();
            let mut cur_comps_temp: Vec<Box<dyn Component>> = cur_ctypes
                .iter()
                .map(|ctype| archetype.data.get(ctype).unwrap()[entity_index].clone_box())
                .collect();
            cur_comps.append(&mut cur_comps_temp);

            // Remove entity from old archetype
            cur_ctypes.iter().for_each(|ctype| {
                archetype.data.get_mut(ctype).unwrap().remove(entity_index);
            });
            archetype.entities.remove(entity_index);
        }

        // If we find an archetype that has exactly the required components, move the entity to it
        if let Some(new_archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.component_types == required_ctypes)
        {
            Self::copy_entity_to_new_arch(new_archetype, cur_comps, Some(new_comp), entity);
        }
        // Otherwise, create a new archetype and move the entity to it
        else {
            let mut new_archetype = Archetype::new(required_ctypes.clone());
            Self::copy_entity_to_new_arch(&mut new_archetype, cur_comps, Some(new_comp), entity);
            self.archetypes.push(new_archetype);
        }
    }

    pub fn remove_component(&mut self, entity: EntityId, comp_type: &ComponentType) {
        // Find in which archetype is the entity
        let mut entity_found = false;
        let mut archetype_id = 0;
        let mut entity_index = 0;
        for (a_index, archetype) in self.archetypes.iter().enumerate() {
            if let Some((e_index, _)) = archetype
                .entities
                .iter()
                .enumerate()
                .find(|(_, e_id)| entity == **e_id)
            {
                entity_found = true;
                archetype_id = a_index;
                entity_index = e_index;
                break;
            }
        }

        // Check if the entity exists
        if !entity_found {
            error!(
                "Cannot remove component {comp_type:?} from entity {entity}: Entity does not exist"
            );
        }

        // Check if the entity has the component
        let archetype = &mut self.archetypes[archetype_id];
        let cur_ctypes = archetype.component_types.clone();
        let mut required_ctypes = archetype.component_types.clone();
        required_ctypes.remove(comp_type);
        if !cur_ctypes.contains(comp_type) {
            error!(
                "Cannot remove component {comp_type:?} from entity {entity}: Entity does not have this component"
            );
        }

        // Get other existing components associated to this entity
        let other_comps: Vec<Box<dyn Component>> = required_ctypes
            .iter()
            .map(|ctype| archetype.data.get(ctype).unwrap()[entity_index].clone_box())
            .collect();

        // Remove entity from old archetype
        cur_ctypes.iter().for_each(|ctype| {
            archetype.data.get_mut(ctype).unwrap().remove(entity_index);
        });
        archetype.entities.remove(entity_index);

        // If we find an archetype that has exactly the required components, move the entity to it
        if let Some(new_archetype) = self
            .archetypes
            .iter_mut()
            .find(|a| a.component_types == required_ctypes)
        {
            Self::copy_entity_to_new_arch(new_archetype, other_comps, None, entity);
        }
        // Otherwise, create a new archetype and move the entity to it
        else {
            let mut new_archetype = Archetype::new(required_ctypes.clone());
            Self::copy_entity_to_new_arch(&mut new_archetype, other_comps, None, entity);
            self.archetypes.push(new_archetype);
        }
    }

    fn copy_entity_to_new_arch(
        new_archetype: &mut Archetype,
        cur_comps: Vec<Box<dyn Component>>,
        new_comp_opt: Option<&dyn Component>,
        entity: EntityId,
    ) {
        for old_c in cur_comps {
            new_archetype
                .data
                .get_mut(&old_c.get_type())
                .unwrap()
                .push(old_c);
        }
        if let Some(new_comp) = new_comp_opt {
            new_archetype
                .data
                .get_mut(&new_comp.get_type())
                .unwrap()
                .push(new_comp.clone_box());
        }
        new_archetype.entities.push(entity);
    }
}
