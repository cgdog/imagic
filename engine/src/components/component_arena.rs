use crate::prelude::{Component, ComponentId};


pub(crate) struct ComponentArena<T: Component> {
    components: Vec<Option<T>>,
    free_list: Vec<u32>,
}

impl<T: Component> ComponentArena<T> {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn add_component(&mut self, component: T) -> ComponentId {
        if let Some(free_id) = self.free_list.pop() {
            let component_id = ComponentId::new(free_id);
            self.components[free_id as usize] = Some(component);
            component_id
        } else {
            let id = self.components.len() as u32;
            let component_id = ComponentId::new(id);
            self.components.push(Some(component));
            component_id
        }
    }

    pub fn remove_component(&mut self, component_id: &ComponentId) -> Option<T> {
        let id = component_id.0 as usize;
        if id >= self.components.len() {
            None
        } else {
            let component = self.components[id].take();
            self.free_list.push(id as u32);
            component
        }
    }

    pub fn replace_component(&mut self, component_id: &ComponentId, new_component: T) -> Option<T> {
        let id = component_id.0 as usize;
        if id >= self.components.len() {
            None
        } else {
            let component = self.components[id].take();
            self.components[id] = Some(new_component);
            component
        }
    }

    pub fn get(&self, component_id: &ComponentId) -> Option<&T> {
        let id = component_id.0 as usize;
        if id >= self.components.len() {
            None
        } else {
            self.components[id].as_ref()
        }
    }

    pub fn get_mut(&mut self, component_id: &ComponentId) -> Option<&mut T> {
        let id = component_id.0 as usize;
        if id >= self.components.len() {
            None
        } else {
            self.components[id].as_mut()
        }
    }
}