use std::{any::{Any, TypeId}, collections::HashMap};

use super::{entity::Entity, sparse_set::SparseSet};

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
    fn remove(&mut self, entity: Entity) {
        self.component_data.remove(entity);
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

pub struct Components {
    component_stores: HashMap<TypeId, Box<dyn ComponentStore>>,
}

impl Default for Components {
    fn default() -> Self {
        Self { component_stores: HashMap::new() }
    }
}

impl Components {
    pub(crate) fn add<T: 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        match self.component_stores.get_mut(&type_id) {
            Some(component_sotre) => {
                if let Some(concrete_component_sotre) = component_sotre.as_any_mut().downcast_mut::<ConcreteComponentStore<T>>() {
                    concrete_component_sotre.add(entity, component);
                }
            }
            None => {
                let mut concrete_component_store = ConcreteComponentStore::<T>::default();
                concrete_component_store.add(entity, component);
                self.component_stores.insert(type_id, Box::new(concrete_component_store));
            }
        }
    }

    pub(crate) fn remove<T: 'static>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();
        if let Some(component_store) = self.component_stores.get_mut(&type_id) {
            if let Some(concrete_component_store) = component_store.as_any_mut().downcast_mut::<ConcreteComponentStore<T>>() {
                concrete_component_store.remove(entity);
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