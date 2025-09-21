use log::error;
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

pub const MAX_OBSOLETE_ENTRIES: usize = 10000;

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
        EntityIdAllocator { next_id: 1 } // 0 is reserved for entities "to delete"
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
        // Find the next entity whose ID is not zero (0 are obsolete entries)
        let mut entity = 0;
        let mut arch_index = 0;
        let mut entity_index = self.entity_index;
        while entity == 0 && self.i_arch < self.archetype_indexes.len() {
            arch_index = self.archetype_indexes[self.i_arch];
            let mut entities = &self.archetype_entities[arch_index];

            // If we finished iterating over the current archertype, continue in next one (assume it's not empty)
            if entity_index >= entities.len() {
                entity_index = 0;
                self.i_arch += 1;
                if self.i_arch >= self.archetype_indexes.len() {
                    return None;
                }
                arch_index = self.archetype_indexes[self.i_arch];
                entities = &self.archetype_entities[arch_index];
            }
            entity = entities[entity_index];
            if entity == 0 {
                entity_index += 1;
            }
        }

        if entity == 0 {
            return None;
        }

        // Get entity reference info to return
        let info = EntityInfo {
            entity,
            arch_index,
            entity_index,
        };
        self.entity_index = entity_index + 1;

        Some(info)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
        // Find the next entity whose ID is not zero (0 are obsolete entries)
        let mut entity = 0;
        let mut comp_index = self.component_index;
        while entity == 0 && self.arch_index < self.ecs.archetypes.len() {
            // If we finished iterating over current archetype OR current archetype does not have the required component
            let end_reached = comp_index >= self.ecs.archetypes[self.arch_index].entities.len();
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
                        comp_index = 0;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return None;
                }
            }

            entity = self.ecs.archetypes[self.arch_index].entities[comp_index];

            if entity == 0 {
                comp_index += 1;
            }
        }

        if entity == 0 {
            return None;
        }

        let res_comp = unsafe {
            &mut *(self.ecs.archetypes[self.arch_index]
                .data
                .get_mut(&self.as_ctype)
                .unwrap()
                .as_mut_ptr()
                .add(comp_index))
        };
        let res = Some((
            res_comp,
            EntityInfo {
                entity,
                arch_index: self.arch_index,
                entity_index: comp_index,
            },
        ));
        self.component_index = comp_index + 1;
        res
    }
}

pub trait System {
    fn run(&self, ecs: &mut Ecs);
}

pub enum Update {
    Edit {
        info: EntityInfo,
        comp: Box<dyn Component>,
    },
    Add {
        info: EntityInfo,
        comp: Box<dyn Component>,
    },
    Delete {
        info: EntityInfo,
        c_type: ComponentType,
    },
    Create(Vec<Box<dyn Component>>),
    DeleteEntity(EntityInfo),
}

pub struct Ecs {
    archetypes: Vec<Archetype>,
    ids: EntityIdAllocator,
    nb_obsolete_entries: usize,
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
            nb_obsolete_entries: 0,
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

    pub fn get_entity_info(&self, entity: usize) -> Option<EntityInfo> {
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

    pub fn apply(&mut self, updates: Vec<Update>) {
        let mut pending_info: HashMap<EntityId, EntityInfo> = HashMap::new();
        for update in updates.iter() {
            match update {
                Update::Edit { info, comp } => {
                    self.edit(*info, comp.as_ref(), &mut pending_info);
                }
                Update::Add { info, comp } => {
                    self.add(*info, comp.as_ref(), &mut pending_info);
                }
                Update::Delete { info, c_type } => {
                    self.remove(*info, *c_type, &mut pending_info);
                }
                Update::Create(comps) => {
                    self.create(comps, &mut pending_info);
                }
                Update::DeleteEntity(info) => {
                    self.delete_entity(*info, &mut pending_info);
                }
            }
        }
        self.clear_obsolete_entries();
    }

    fn edit(
        &mut self,
        info: EntityInfo,
        comp: &dyn Component,
        pending_info: &mut HashMap<EntityId, EntityInfo>,
    ) {
        let actualized_info = pending_info.get(&info.entity).unwrap_or(&info);
        let EntityInfo {
            entity: _,
            arch_index,
            entity_index,
        } = *actualized_info;
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            error!(
                "Cannot apply a component edit for entity {:?}: Entity not found",
                actualized_info
            );
        } else if let Some(components) = self.archetypes[arch_index].data.get_mut(&comp.get_type())
        {
            components[entity_index] = comp.clone_box();
        } else {
            error!("Cannot apply a component edit for entity {:?}: Entity does not have this component", actualized_info);
        }
    }

    fn add(
        &mut self,
        info: EntityInfo,
        comp: &dyn Component,
        pending_info: &mut HashMap<EntityId, EntityInfo>,
    ) {
        let actualized_info = pending_info.get(&info.entity).unwrap_or(&info);
        let EntityInfo {
            entity,
            arch_index,
            entity_index,
        } = *actualized_info;
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            error!(
                "Cannot apply a component addition for entity {:?}: Entity not found",
                actualized_info
            );
        } else if self.archetypes[arch_index]
            .data
            .get_mut(&comp.get_type())
            .is_some()
        {
            error!(
                            "Cannot apply a component addition for entity {:?}: Entity already has a component of this type",
                            actualized_info
                        );
        } else {
            let mut cur_ctypes = HashSet::new();
            let mut required_ctypes = HashSet::from([comp.get_type()]);

            // Get already existing components associated to this entity
            let archetype = &mut self.archetypes[arch_index];
            cur_ctypes = cur_ctypes
                .union(&archetype.component_types)
                .cloned()
                .collect();
            required_ctypes = required_ctypes
                .union(&archetype.component_types)
                .cloned()
                .collect();
            let cur_comps: Vec<Box<dyn Component>> = cur_ctypes
                .iter()
                .map(|ctype| archetype.data.get(ctype).unwrap()[entity_index].clone_box())
                .collect();

            // Mark entity as Remove entity from old archetype
            archetype.entities[entity_index] = 0;
            self.nb_obsolete_entries += 1;

            // If we find an archetype that has exactly the required components, move the entity to it
            if let Some((new_arch_index, new_archetype)) = self
                .archetypes
                .iter_mut()
                .enumerate()
                .find(|(_, a)| a.component_types == required_ctypes)
            {
                pending_info.insert(
                    entity,
                    EntityInfo {
                        entity,
                        arch_index: new_arch_index,
                        entity_index: new_archetype.entities.len(),
                    },
                );
                Self::copy_entity_to_new_arch(new_archetype, cur_comps, Some(comp), entity);
            }
            // Otherwise, create a new archetype and move the entity to it
            else {
                pending_info.insert(
                    entity,
                    EntityInfo {
                        entity,
                        arch_index: self.archetypes.len(),
                        entity_index: 0,
                    },
                );
                let mut new_archetype = Archetype::new(required_ctypes.clone());
                Self::copy_entity_to_new_arch(&mut new_archetype, cur_comps, Some(comp), entity);
                self.archetypes.push(new_archetype);
            }
        }
    }

    fn remove(
        &mut self,
        info: EntityInfo,
        c_type: ComponentType,
        pending_info: &mut HashMap<EntityId, EntityInfo>,
    ) {
        let actualized_info = pending_info.get(&info.entity).unwrap_or(&info);
        let EntityInfo {
            entity,
            arch_index,
            entity_index,
        } = *actualized_info;
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            error!(
                "Cannot apply a component deletion for entity {:?}: Entity not found",
                actualized_info
            );
        } else if !self.archetypes[arch_index]
            .component_types
            .contains(&c_type)
        {
            error!("Cannot apply a component deletion for entity {:?}: Entity does not have this component", actualized_info);
        } else {
            let archetype = &mut self.archetypes[arch_index];
            let mut required_ctypes = archetype.component_types.clone();
            required_ctypes.remove(&c_type);

            // Get other existing components associated to this entity
            let other_comps: Vec<Box<dyn Component>> = required_ctypes
                .iter()
                .map(|ctype| archetype.data.get(ctype).unwrap()[entity_index].clone_box())
                .collect();

            // Mark entity as Remove entity from old archetype
            archetype.entities[entity_index] = 0;
            self.nb_obsolete_entries += 1;

            // If we find an archetype that has exactly the required components, move the entity to it
            if let Some((new_arch_index, new_archetype)) = self
                .archetypes
                .iter_mut()
                .enumerate()
                .find(|(_, a)| a.component_types == required_ctypes)
            {
                pending_info.insert(
                    entity,
                    EntityInfo {
                        entity,
                        arch_index: new_arch_index,
                        entity_index: new_archetype.entities.len(),
                    },
                );
                Self::copy_entity_to_new_arch(new_archetype, other_comps, None, entity);
            }
            // Otherwise, create a new archetype and move the entity to it
            else {
                pending_info.insert(
                    entity,
                    EntityInfo {
                        entity,
                        arch_index: self.archetypes.len(),
                        entity_index: 0,
                    },
                );
                let mut new_archetype = Archetype::new(required_ctypes.clone());
                Self::copy_entity_to_new_arch(&mut new_archetype, other_comps, None, entity);
                self.archetypes.push(new_archetype);
            }
        }
    }

    fn create(
        &mut self,
        comps: &Vec<Box<dyn Component>>,
        pending_info: &mut HashMap<EntityId, EntityInfo>,
    ) {
        let required_ctypes: HashSet<ComponentType> = comps.iter().map(|c| c.get_type()).collect();

        // Get archetype with the right components or create one
        let (arch_index, archetype) = if let Some(a) = self
            .archetypes
            .iter_mut()
            .enumerate()
            .find(|(_, a)| a.component_types == required_ctypes)
        {
            a
        } else {
            self.archetypes.push(Archetype::new(required_ctypes));
            (
                self.archetypes.len() - 1,
                self.archetypes.last_mut().unwrap(),
            )
        };

        // Create the entity to the archetype
        let entity = self.ids.get_next_id();
        archetype.entities.push(entity);
        for comp in comps {
            archetype
                .data
                .get_mut(&comp.get_type())
                .unwrap()
                .push(comp.clone_box());
        }

        pending_info.insert(
            entity,
            EntityInfo {
                entity,
                arch_index,
                entity_index: archetype.entities.len() - 1,
            },
        );
    }

    fn delete_entity(
        &mut self,
        info: EntityInfo,
        pending_info: &mut HashMap<EntityId, EntityInfo>,
    ) {
        let actualized_info = pending_info.get(&info.entity).unwrap_or(&info);
        let EntityInfo {
            entity,
            arch_index,
            entity_index,
        } = *actualized_info;
        if arch_index >= self.archetypes.len()
            || entity_index >= self.archetypes[arch_index].entities.len()
        {
            error!(
                "Cannot delete entity {:?}: Entity not found",
                actualized_info
            );
        } else {
            // Mark entity as a 'to remove entity'
            let archetype = &mut self.archetypes[arch_index];
            archetype.entities[entity_index] = 0;
            self.nb_obsolete_entries += 1;
            pending_info.remove(&entity);
        }
    }

    fn clear_obsolete_entries(&mut self) {
        if self.nb_obsolete_entries > MAX_OBSOLETE_ENTRIES {
            for arch in self.archetypes.iter_mut() {
                let mut to_remove_idx: Vec<usize> = arch
                    .entities
                    .iter()
                    .enumerate()
                    .filter(|&(_, entity)| *entity == 0)
                    .map(|(id, _)| id)
                    .collect();
                to_remove_idx.reverse(); // Sort big index first, for safe vec remove
                for comps in arch.data.values_mut() {
                    for idx in to_remove_idx.iter() {
                        comps.remove(*idx);
                    }
                }
                for idx in to_remove_idx.iter() {
                    arch.entities.remove(*idx);
                }
            }
            self.nb_obsolete_entries = 0;
        }
    }
}
