use std::{any::{type_name, Any, TypeId}, collections::HashMap};

use super::{entity::Entity, sparse_set::SparseSet, types::TupleTypesInfo};

trait ComponentStore {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: Entity) -> bool;
}
pub struct ConcreteComponentStore<T: 'static> {
    component_data: SparseSet<T>,
}

impl<T> Default for ConcreteComponentStore<T> {
    fn default() -> Self {
        Self { component_data: SparseSet::default() }
    }
}

impl<T> ComponentStore for ConcreteComponentStore<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn remove(&mut self, entity: Entity) -> bool {
        match self.remove(entity) {
            Some(_) => true,
            None => false,
        }
    }

}

impl<T> ConcreteComponentStore<T> {
    fn add(&mut self, entity: Entity, component: T) {
        self.component_data.insert(entity, component);
    }
    fn remove(&mut self, entity: Entity) -> Option<T> {
        self.component_data.remove(entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.component_data.get_mut(entity)
    }

    fn get(& self, entity: Entity) -> Option<& T> {
        self.component_data.get(entity)
    }

    fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.component_data.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Entity, &mut T)> {
        self.component_data.iter_mut()
    }
}

type ComponentId = u32;
struct ComponentIdGenerator {

}
impl ComponentIdGenerator {
    pub(crate) fn generate_component_handle() -> ComponentId {
        static mut NEXT_COMPONENT_ID: ComponentId = 0;
        let result = unsafe { NEXT_COMPONENT_ID };
        unsafe { NEXT_COMPONENT_ID += 1 };
        result
    }
}

#[allow(unused)]
struct ComponentRefMetaInfo {
    pub(crate) is_mut_ref: bool,
    pub(crate) original_type_id: TypeId,
    pub(crate) type_name: &'static str,
}

impl ComponentRefMetaInfo {
    pub(crate) fn new(is_mut_ref: bool, original_type_id: TypeId, type_name: &'static str,) -> Self {
        Self { is_mut_ref, original_type_id, type_name }
    }

    pub(crate) fn new_ref(original_type_id: TypeId, type_name: &'static str,) -> Self {
        Self::new(false, original_type_id, type_name)
    }

    pub(crate) fn new_mut_ref(original_type_id: TypeId, type_name: &'static str,) -> Self {
        Self::new(true, original_type_id, type_name)
    }
}
type ComponentMask = u64;
pub struct Components {
    component_stores: HashMap<TypeId, Box<dyn ComponentStore>>,
    /// The mapping between ref or mut ref TypeId and original TypeId.
    ref_meta_info_map: HashMap<TypeId, ComponentRefMetaInfo>,
    component_type_to_id: HashMap<TypeId, ComponentId>,
    component_id_to_type_id: HashMap<ComponentId, TypeId>,
    // Something like archetypes.
    entity_to_component_masks: HashMap<Entity, ComponentMask>,

    cache_tuple_type_id_to_mask: HashMap<TypeId, ComponentMask>,
    cache_mask_to_entities: HashMap<ComponentMask, Vec<Entity>>,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            component_stores: HashMap::new(),
            ref_meta_info_map: HashMap::new(),
            component_type_to_id: HashMap::new(),
            component_id_to_type_id: HashMap::new(),
            entity_to_component_masks: HashMap::new(),
            cache_tuple_type_id_to_mask: HashMap::new(),
            cache_mask_to_entities: HashMap::new(),
        }
    }
}

impl Components {
    pub(crate) fn add<T: 'static>(&mut self, entity: Entity, component: T) {
        let original_type_id = TypeId::of::<T>();
        match self.component_stores.get_mut(&original_type_id) {
            Some(component_sotre) => {
                if let Some(concrete_component_sotre) = component_sotre.as_any_mut().downcast_mut::<ConcreteComponentStore<T>>() {
                    concrete_component_sotre.add(entity, component);
                    if let Some(component_id) = self.component_type_to_id.get(&original_type_id) {
                        self.set_entity_component_mask(entity, *component_id);
                    } else {
                        panic!("Failed to get entity component mask, unexpectecdlly.");
                    }
                } else {
                    panic!("No component component store is found, unexpectedlly.");
                }
            }
            None => {
                let mut concrete_component_store = ConcreteComponentStore::<T>::default();
                concrete_component_store.add(entity, component);
                self.component_stores.insert(original_type_id, Box::new(concrete_component_store));
                let ref_type_id = TypeId::of::<&T>();
                let mut_ref_type_id = TypeId::of::<&mut T>();
                self.ref_meta_info_map.insert(ref_type_id, ComponentRefMetaInfo::new_ref(original_type_id, type_name::<&T>()));
                self.ref_meta_info_map.insert(mut_ref_type_id, ComponentRefMetaInfo::new_mut_ref(original_type_id, type_name::<&mut T>()));
                let component_id = ComponentIdGenerator::generate_component_handle();
                self.component_type_to_id.insert(original_type_id, component_id);
                self.component_id_to_type_id.insert(component_id, original_type_id);
                self.set_entity_component_mask(entity, component_id);
            }
        }
    }

    fn set_entity_component_mask(&mut self, entity: Entity, component_id: u32) {
        if let Some(component_mask) = self.entity_to_component_masks.get_mut(&entity) {
            *component_mask |= 1 << component_id;
        } else {
            let component_mask = 1 << component_id;
            self.entity_to_component_masks.insert(entity, component_mask);
        }
    }

    pub(crate) fn remove<T: 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        self.remove_component_by_type_id(&type_id, entity);
        self.clean_cache_mask_to_entity(entity);
    }

    fn remove_component_by_type_id(&mut self, type_id: &TypeId, entity: Entity) {
        if let Some(component_store) = self.component_stores.get_mut(&type_id) {
            if component_store.remove(entity) {
                self.entity_to_component_masks.remove(&entity);
            }
        }
    }

    fn clean_cache_mask_to_entity(&mut self, entity: Entity) {
        self.cache_mask_to_entities.retain(|_, v|!v.contains(&entity));
    }

    pub(crate) fn remove_all(&mut self, entity: Entity) {
        if let Some(&component_mask) = self.entity_to_component_masks.get(&entity) {
            for component_id in 0..64 {
                if (component_mask & (1 << component_id)) != 0 {
                    if let Some(component_type_id) = self.component_id_to_type_id.get(&component_id).cloned() {
                        self.remove_component_by_type_id(&component_type_id, entity);
                    }
                }
            }
            self.clean_cache_mask_to_entity(entity);
        }
    }

    pub(crate) fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if let Some(component_store) = self.component_stores.get(&type_id) {
            if let Some(concrete_component_store) = component_store.as_any().downcast_ref::<ConcreteComponentStore<T>>() {
                return concrete_component_store.get(entity);
            }
        }
        None
    }

    /// e.g., let entities = get_all<(i32, Transform, u32)>();
    pub(crate) fn get_all<T: TupleTypesInfo>(&mut self) -> Option<Vec<Entity>> {
        let mut mask: ComponentMask = 0;

        let tuple_type_id = TypeId::of::<T>();
        if let Some(&tuple_mask) = self.cache_tuple_type_id_to_mask.get(&tuple_type_id) {
            mask = tuple_mask;
        } else {
            let original_type_ids = T::type_ids();
            for original_type_id in &original_type_ids {
                if let Some(component_id) = self.component_type_to_id.get(original_type_id) {
                    mask |= 1 << component_id;
                } else {
                    return None;
                }
            }
            self.cache_tuple_type_id_to_mask.insert(tuple_type_id, mask);
        }

        if let Some(cached_entities) = self.cache_mask_to_entities.get(&mask) {
            if cached_entities.len() > 0 {
                Some(cached_entities.clone())
            } else {
                None
            }
        } else {
            let mut entities = Vec::new();
            for (&entity, &cur_mask) in self.entity_to_component_masks.iter() {
                if (cur_mask & mask) == mask {
                    entities.push(entity);
                }
            }
    
            self.cache_mask_to_entities.insert(mask, entities.clone());
            if entities.len() > 0 {
                Some(entities)
            } else {
                None
            }
        }
    }

    pub(crate) fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if let Some(component_store) = self.component_stores.get_mut(&type_id) {
            if let Some(concrete_component_store) = component_store.as_any_mut().downcast_mut::<ConcreteComponentStore<T>>() {
                return concrete_component_store.get_mut(entity);
            }
        }
        None
    }

    pub(crate) fn iter<T: 'static>(&self) -> impl Iterator<Item = (&Entity, & T)> {
        let type_id = TypeId::of::<T>();
        self.component_stores
            .get(&type_id)
            .and_then(|component_store| {
                component_store
                    .as_any()
                    .downcast_ref::<ConcreteComponentStore<T>>()
                    .map(|concrete_component_store| concrete_component_store.iter())
            })
            .into_iter()
            .flatten()
    }

    pub(crate) fn iter_mut<T: 'static>(&mut self) -> impl Iterator<Item = (&mut Entity, &mut T)> {
        let type_id = TypeId::of::<T>();
        self.component_stores
            .get_mut(&type_id)
            .and_then(|component_store| {
                component_store
                    .as_any_mut()
                    .downcast_mut::<ConcreteComponentStore<T>>()
                    .map(|concrete_component_store| concrete_component_store.iter_mut())
            })
            .into_iter()
            .flatten()
    }

}