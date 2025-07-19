use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use log::error;

pub type ComponentType = TypeId;

macro_rules! to_ctype {
    ($CompType:ident) => {
        TypeId::of::<$CompType>()
    };
}
macro_rules! iter_entities {
    ($self:expr, $CompType:ident) => {
        $self.iter_entities(TypeId::of::<$CompType>())
    };
}
macro_rules! iter_entities_with {
    ($self:expr, $($CompType:ident),+) => {
        $self.iter_entities_with(
            &[
                $(TypeId::of::<$CompType>()),+
            ]
        )
    };
}
macro_rules! iter_components {
    ($self:expr, $CompType:ident) => {
        $self
            .iter_components(TypeId::of::<$CompType>())
            .map(|(box_dyn_c, entity_info)| {
                (
                    box_dyn_c.as_any_mut().downcast_mut::<$CompType>().unwrap(),
                    entity_info,
                )
            })
    };
}
macro_rules! iter_components_with {
    ($self:expr, ($($RequiredCompType:ident),+), $CompType:ident) => {
        $self
            .iter_components_with(&[$(TypeId::of::<$RequiredCompType>()),+], TypeId::of::<$CompType>())
            .map(|(box_dyn_c, e)| {
                (
                    box_dyn_c.as_any_mut().downcast_mut::<$CompType>().unwrap(),
                    e,
                )
            })
    };
}

pub trait Component: Any + CloneComponent {
    fn get_type(&self) -> ComponentType {
        TypeId::of::<Self>()
    }
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
    pub fn new(ecs: &Ecs, required_ctypes: HashSet<ComponentType>) -> Self {
        EntityIterator {
            entity_index: 0,
            i_arch: 0,
            archetype_indexes: ecs.with_comps(required_ctypes),
            archetype_entities: ecs.get_archetype_entity_info(),
        }
    }
}

impl Iterator for EntityIterator {
    type Item = EntityInfo;

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
        let info = EntityInfo {
            entity,
            arch_index,
            entity_index: self.entity_index,
        };
        self.entity_index += 1;

        Some(info)
    }
}

#[derive(Clone, Copy)]
pub struct EntityInfo {
    pub entity: EntityId,
    pub arch_index: usize,
    pub entity_index: usize,
}

pub struct ComponentIterator<'a> {
    ecs: &'a mut Ecs,
    required_ctypes: HashSet<ComponentType>,
    as_ctype: ComponentType,
    arch_index: usize,
    component_index: usize,
}

impl<'a> Iterator for ComponentIterator<'a> {
    type Item = (&'a mut Box<dyn Component>, EntityInfo);
    fn next(&mut self) -> Option<Self::Item> {
        if self.arch_index >= self.ecs.archetypes.len() {
            return None;
        }

        // If we finished iterating over current archetype OR current archetype does not have the
        // required component
        let end_reached =
            self.component_index >= self.ecs.archetypes[self.arch_index].entities.len();
        let has_comp = self
            .ecs
            .has_components(self.arch_index, &self.required_ctypes);
        if end_reached || !has_comp {
            // Find the next non-empty archetype that has the required component
            let mut found = false;
            for i in self.arch_index + 1..self.ecs.archetypes.len() {
                if !self.ecs.archetypes[i].entities.is_empty()
                    && self.ecs.has_components(i, &self.required_ctypes)
                {
                    self.arch_index = i;
                    self.component_index = 0;
                    found = true;
                }
            }
            if !found {
                return None;
            }
        }

        let res = unsafe {
            &mut *(self.ecs.archetypes[self.arch_index]
                .data
                .get_mut(&self.as_ctype)
                .unwrap()
                .as_mut_ptr()
                .add(self.component_index))
        };
        let entity = self.ecs.archetypes[self.arch_index].entities[self.component_index];
        let res = (
            res,
            EntityInfo {
                entity,
                arch_index: self.arch_index,
                entity_index: self.component_index,
            },
        );

        self.component_index += 1;

        Some(res)
    }
}

pub trait System {
    fn run(&self, ecs: &mut Ecs);
}

pub struct Ecs {
    archetypes: Vec<Archetype>,
    ids: EntityIdAllocator,
}

impl Default for Ecs {
    fn default() -> Self {
        Ecs::new()
    }
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            ids: EntityIdAllocator::new(),
        }
    }

    pub fn has_component(&self, arch_index: usize, ctype: &ComponentType) -> bool {
        if arch_index < self.archetypes.len() {
            return self.archetypes[arch_index].component_types.contains(ctype);
        }
        false
    }

    pub fn has_components(&self, arch_index: usize, ctypes: &HashSet<ComponentType>) -> bool {
        if arch_index < self.archetypes.len() {
            return self.archetypes[arch_index]
                .component_types
                .is_superset(ctypes);
        }
        false
    }

    pub fn get_component_from_entity<C>(&self, entity: usize) -> Option<&C>
    where
        C: Component,
    {
        if let Some(info) = self.get_entity_info(entity) {
            return self.get_component::<C>(&info);
        }
        None
    }

    pub fn get_component_mut_from_entity<C>(&mut self, entity: usize) -> Option<&mut C>
    where
        C: Component,
    {
        if let Some(info) = self.get_entity_info(entity) {
            return self.get_component_mut::<C>(&info);
        }
        None
    }

    pub fn get_component<C>(&self, info: &EntityInfo) -> Option<&C>
    where
        C: Component,
    {
        let ctype = TypeId::of::<C>() as ComponentType;
        if info.arch_index >= self.archetypes.len()
            || info.entity_index >= self.archetypes[info.arch_index].entities.len()
        {
            None
        } else if let Some(components) = self.archetypes[info.arch_index].data.get(&ctype) {
            components[info.entity_index].as_any().downcast_ref::<C>()
        } else {
            None
        }
    }

    pub fn get_component_mut<C>(&mut self, info: &EntityInfo) -> Option<&mut C>
    where
        C: Component,
    {
        let ctype = TypeId::of::<C>() as ComponentType;
        if info.arch_index >= self.archetypes.len()
            || info.entity_index >= self.archetypes[info.arch_index].entities.len()
        {
            None
        } else if let Some(components) = self.archetypes[info.arch_index].data.get_mut(&ctype) {
            components[info.entity_index]
                .as_any_mut()
                .downcast_mut::<C>()
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

    pub fn iter_components(&mut self, as_ctype: ComponentType) -> ComponentIterator {
        self.iter_components_with_types(HashSet::from([as_ctype]), as_ctype)
    }

    pub fn iter_components_with(
        &mut self,
        required_ctypes: &[ComponentType],
        as_ctype: ComponentType,
    ) -> ComponentIterator {
        self.iter_components_with_types(required_ctypes.iter().copied().collect(), as_ctype)
    }

    fn iter_components_with_types(
        &mut self,
        required_ctypes: HashSet<ComponentType>,
        as_ctype: ComponentType,
    ) -> ComponentIterator {
        ComponentIterator {
            ecs: self,
            required_ctypes,
            as_ctype,
            arch_index: 0,
            component_index: 0,
        }
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

    fn get_entity_info(&self, entity: usize) -> Option<EntityInfo> {
        // Look in all archetypes to find the entity
        for (arch_index, archetype) in self.archetypes.iter().enumerate() {
            if let Some((entity_index, entity)) = archetype
                .entities
                .iter()
                .enumerate()
                .find(|(_, e)| entity == **e)
            {
                return Some(EntityInfo {
                    entity: *entity,
                    arch_index,
                    entity_index,
                });
            }
        }

        // Entity not found
        None
    }

    pub fn create_entity_with(&mut self, components: &[&dyn Component]) {
        let entity = self.ids.get_next_id();
        for comp in components {
            self.add_component(entity, *comp);
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

    pub fn remove_component(&mut self, entity: EntityId, comp_type: ComponentType) {
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
        required_ctypes.remove(&comp_type);
        if !cur_ctypes.contains(&comp_type) {
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
