use std::{any::{type_name, Any, TypeId}, collections::{HashMap, HashSet}};

use super::{entity::Entity, sparse_set::SparseSet, types::TupleTypesInfo};

trait ComponentStore {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
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
    // Something like archetypes.
    component_masks_to_entities: HashMap<ComponentMask, HashSet<Entity>>,
    entity_to_component_masks: HashMap<Entity, ComponentMask>,
}

impl Default for Components {
    fn default() -> Self {
        Self {
            component_stores: HashMap::new(),
            ref_meta_info_map: HashMap::new(),
            component_type_to_id: HashMap::new(),
            component_masks_to_entities: HashMap::new(),
            entity_to_component_masks: HashMap::new(),
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
                        panic!("Failed to get entit component mask, unexpectecdlly.");
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
                self.set_entity_component_mask(entity, component_id);
            }
        }
    }

    fn set_entity_component_mask(&mut self, entity: Entity, component_id: u32) {
        if let Some(component_mask) = self.entity_to_component_masks.get_mut(&entity) {
            *component_mask |= 1 << component_id;
            if let Some(mask_entities) = self.component_masks_to_entities.get_mut(&component_mask) {
                mask_entities.insert(entity);
            } else {
                let mut entities = HashSet::new();
                entities.insert(entity);
                self.component_masks_to_entities.insert(*component_mask, entities);
            }
        } else {
            let component_mask = 1 << component_id;
            self.entity_to_component_masks.insert(entity, component_mask);
            let mut entities = HashSet::new();
            entities.insert(entity);
            self.component_masks_to_entities.insert(component_mask, entities);
        }
    }

    fn unset_entity_component_mask(&mut self, entity: Entity, component_id: u32) {
        if let Some(component_mask) = self.entity_to_component_masks.get_mut(&entity) {
            let old_component_mask = *component_mask;
            *component_mask &= !(1 << component_id);
            self.component_masks_to_entities.get_mut(&old_component_mask).expect("Failed to get component mask unexpectedlly").remove(&entity);
            if let Some(new_component_mask_entities) = self.component_masks_to_entities.get_mut(component_mask) {
                new_component_mask_entities.insert(entity);
            } else {
                let mut entities = HashSet::new();
                entities.insert(entity);
                self.component_masks_to_entities.insert(*component_mask, entities);
            }
        }
    }

    pub(crate) fn remove<T: 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(component_store) = self.component_stores.get_mut(&type_id) {
            if let Some(concrete_component_store) = component_store.as_any_mut().downcast_mut::<ConcreteComponentStore<T>>() {
                concrete_component_store.remove(entity);
                if let Some(component_id) = self.component_type_to_id.get(&type_id) {
                    self.unset_entity_component_mask(entity, *component_id);
                }
            }
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
    pub(crate) fn get_all<T: TupleTypesInfo>(&self) -> Option<Vec<Entity>> {
        let original_type_ids = T::type_ids();
        let mut mask: ComponentMask = 0;
        for original_type_id in &original_type_ids {
            if let Some(component_id) = self.component_type_to_id.get(original_type_id) {
                mask |= 1 << component_id;
            } else {
                return None;
            }
        }
        if let Some(entities) = self.component_masks_to_entities.get(&mask) {
            return Some(entities.iter().map(|e|e.clone()).collect())
        }
        None
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

    // pub(crate) fn query<T: TupleTypesInfo>(&mut self) -> Option<Vec<T>> {
    //     let type_ids = T::type_ids();
    //     let mut mask: ComponentMask = 0;
    //     for type_id in &type_ids {
    //         if let Some(ref_meta_info) = self.ref_meta_info_map.get(type_id) {
    //             if let Some(component_id) = self.component_type_to_id.get(&ref_meta_info.original_type_id) {
    //                 mask |= 1 << component_id;
    //             } else {
    //                 return None;
    //             }
    //         } else {
    //             return None;
    //         }
    //     }
    //     if let Some(entities) = self.component_masks_to_entities.get_mut(&mask) {
    //         for entity in entities.iter() {
    //             // get components & create tuple
    //         }
    //     }
    //     None
    // }

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