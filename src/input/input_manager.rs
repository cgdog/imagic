use std::collections::HashMap;

use crate::types::ID;

use super::{MouseEvent, MouseInputListener};

pub struct InputManager {
    mouse_input_listeners: HashMap<ID, Box<dyn MouseInputListener>>,
    next_id: ID,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            mouse_input_listeners: HashMap::new(),
            next_id: 0,
        }
    }
}

impl InputManager {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn register_mouse_input_listener(&mut self, listener: Box<dyn MouseInputListener>) -> ID {
        self.mouse_input_listeners.insert(self.next_id, listener);
        self.next_id += 1;
        self.next_id
    }

    pub fn unregister_mouse_input_listener(&mut self, id: ID) {
        self.mouse_input_listeners.remove(&id);
    }

    pub fn trigger_mouse_input_event(&mut self, event: MouseEvent) {
        for listener in self.mouse_input_listeners.values_mut() {
            listener.on_mouse_input(event);
        }
    }
}