use std::any::Any;

use ahash::AHashMap;
use crate::prelude::{Component, ComponentId, ComponentTypeId, INVALID_COMPONENT_ID, component_arena::ComponentArena};

/// ComponentStorage is a trait that all component storages must implement.
pub(crate) trait AbstractComponentStorage  : Any{
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove_component_internal(&mut self, componente_id: &ComponentId);
}

pub(crate) struct ConcreteComponentStarage<T: Component> {
    component_arena:ComponentArena<T>,
}

impl<T: Component> AbstractComponentStorage for ConcreteComponentStarage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_component_internal(&mut self, componente_id: &ComponentId) {
        self.component_arena.remove_component(componente_id);
    }
}

impl<T: Component> ConcreteComponentStarage<T> {
    pub fn new() -> Self {
        Self {
            component_arena: ComponentArena::new(),
        }
    }

    pub fn remove_component(&mut self, component_id: &ComponentId) -> Option<T> {
        self.component_arena.remove_component(component_id)
    }

    pub fn add_component(&mut self, component: T) -> ComponentId {
        self.component_arena.add_component(component)
    }

    pub fn replace_component(&mut self, component_id: &ComponentId, new_component: T) -> Option<T> {
        self.component_arena.replace_component(component_id, new_component)
    }
}

pub(crate) struct ComponentStorages {
    component_storages: AHashMap<ComponentTypeId, Box<dyn AbstractComponentStorage>>,
}

impl ComponentStorages {
    pub fn new() -> Self {
        Self { component_storages: AHashMap::new() }
    }

    pub fn add_component<T: Component>(&mut self, component: T, component_type_id: ComponentTypeId) -> ComponentId {
        let component_storage = self.component_storages.entry(component_type_id).or_insert_with(|| {
            Box::new(ConcreteComponentStarage::<T>::new())
        });
        if let Some(concrete_component_storage) = component_storage.as_any_mut().downcast_mut::<ConcreteComponentStarage<T>>() {
            concrete_component_storage.add_component(component)
        } else {
            log::error!("Failed to add a component with component type id {:?}", component_type_id);
            INVALID_COMPONENT_ID
        }
    }

    pub fn remove_component<T: Component>(&mut self, component_id: &ComponentId, component_type_id: ComponentTypeId) -> Option<T> {
        if let Some(component_storage) = self.component_storages.get_mut(&component_type_id) {
            if let Some(concrete_component_storage) = component_storage.as_any_mut().downcast_mut::<ConcreteComponentStarage<T>>() {
                concrete_component_storage.remove_component(component_id)
            } else {
                log::error!("Failed to remove a component with component type id {:?}", component_type_id);
                None
            }
        } else {
            None
        }
    }

    pub fn replace_component<T: Component>(&mut self, component_id: &ComponentId, component_type_id: ComponentTypeId, new_component: T) -> Option<T> {
        if let Some(component_storage) = self.component_storages.get_mut(&component_type_id) {
            if let Some(concrete_component_storage) = component_storage.as_any_mut().downcast_mut::<ConcreteComponentStarage<T>>() {
                concrete_component_storage.replace_component(component_id, new_component)
            } else {
                log::error!("Failed to replace a component with component type id {:?}", component_type_id);
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn remove_component_internal(&mut self, component_id: &ComponentId, component_type_id: &ComponentTypeId) {
        if let Some(component_storage) = self.component_storages.get_mut(component_type_id) {
            component_storage.remove_component_internal(component_id);
        }
    }

    pub fn get_component<T: Component>(&self, component_id: &ComponentId) -> Option<&T> {
        let component_type_id = std::any::TypeId::of::<T>();
        let storage = self.component_storages.get(&component_type_id);
        if let Some(component_storage) = storage {
            if let Some(concrete_component_storage) = component_storage.as_any().downcast_ref::<ConcreteComponentStarage<T>>() {
                concrete_component_storage.component_arena.get(&component_id)
            } else {
                #[cfg(debug_assertions)]
                {
                    log::warn!("Failed to get a component with component type id {:?}, component_id: {}", component_type_id, component_id);
                }
                None
            }
        } else {
            #[cfg(debug_assertions)]
            {
                log::warn!("Failed to get a component with component type id {:?}, component_id: {}", component_type_id, component_id);
            }
            None
        }
    }

    pub fn get_component_mut<T: Component>(&mut self, component_id: &ComponentId) -> Option<&mut T> {
        let component_type_id = std::any::TypeId::of::<T>();
        let storage = self.component_storages.get_mut(&component_type_id);
        if let Some(component_storage) = storage {
            if let Some(concrete_component_storage) = component_storage.as_any_mut().downcast_mut::<ConcreteComponentStarage<T>>() {
                concrete_component_storage.component_arena.get_mut(&component_id)
            } else {
                #[cfg(debug_assertions)]
                {
                    log::warn!("Failed to get a component with component type id {:?}, component_id: {}", component_type_id, component_id);
                }
                None
            }
        } else {
            #[cfg(debug_assertions)]
            {
                log::warn!("Failed to get a component with component type id {:?}, component_id: {}", component_type_id, component_id);
            }
            None
        }
    }
}

