use crate::types::Handle;

struct ArenaSlot<T> {
    value: Option<T>,
    generation: u32,
}

/// A generic arena for managing objects of type T.
pub struct Arena<T> {
    slots: Vec<ArenaSlot<T>>,
    free_list: Vec<u32>,
}

impl<T> Arena<T> {
    /// Create a new empty Arena.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Add a new object to the arena.
    /// # Arguments
    /// * `value` - The value to be stored in the arena.
    /// 
    /// # Returns
    /// A handle to the newly created object.
    pub fn add<Tag>(&mut self, value: T) -> Handle<Tag> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            let handle = Handle::new_with_generation(index as u64, slot.generation);
            slot.value = Some(value);
            handle
        } else {
            let id = self.slots.len() as u64;
            let handle = Handle::new(id);
            self.slots.push(ArenaSlot { value: Some(value), generation: 0 });
            handle
        }
    }

    /// Get a reference to the object stored in the arena.
    /// # Arguments
    /// * `handle` - The handle to the object.
    /// 
    /// # Returns
    /// A reference to the object if the handle is valid, otherwise None.
    pub fn get<Tag>(&self, handle: &Handle<Tag>) -> Option<&T> {
        let slot = self.slots.get(handle.id as usize)?;
        if slot.generation != handle.generation {
            None
        } else {
            slot.value.as_ref()
        }
    }

    /// Get a reference to the object stored in the arena, panicking if the handle is invalid.
    pub fn get_forcely<Tag>(&self, handle: &Handle<Tag>) -> &T {
        let slot = self.slots.get(handle.id as usize)
            .expect("Invalid Handle when Arena get_forcely()");
        if slot.generation != handle.generation {
            panic!("Invalid Handle generation ({}) when Arena get_forcely()", handle);
        } else {
            slot.value.as_ref().expect("Value has been removed in Arena get_forcely()")
        }
    }

    /// Get a mutable reference to the object stored in the arena.
    /// # Arguments
    /// * `handle` - The handle to the object.
    /// 
    /// # Returns
    /// A mutable reference to the object if the handle is valid, otherwise None.
    pub fn get_mut<Tag>(&mut self, handle: &Handle<Tag>) -> Option<&mut T> {
        let slot = self.slots.get_mut(handle.id as usize)?;
        if slot.generation != handle.generation {
            None
        } else {
            slot.value.as_mut()
        }
    }

    /// Get a mutable reference to the object stored in the arena, panicking if the handle is invalid.
    /// # Arguments
    /// * `handle` - The handle to the object.
    /// 
    /// # Returns
    /// A mutable reference to the object if the handle is valid, otherwise None.
    pub fn get_mut_forcely<Tag>(&mut self, handle: &Handle<Tag>) -> &mut T {
        let slot = self.slots.get_mut(handle.id as usize)
            .expect("Invalid Handle when Arena get_mut_forcely()");
        if slot.generation != handle.generation {
            panic!("Invalid Handle generation ({}) when Arena get_mut_forcely()", handle);
        } else {
            slot.value.as_mut().expect("Value has been removed in Arena get_mut_forcely()")
        }
    }

    /// Remove the object associated with the given handle.
    /// # Arguments
    /// * `handle` - The handle to the object to be removed.
    /// 
    /// # Returns
    /// The removed object if the handle is valid, otherwise None.
    pub fn remove<Tag>(&mut self, handle: &Handle<Tag>) -> Option<T> {
        if let Some(slot) = self.slots.get_mut(handle.id as usize) {
            if slot.generation != handle.generation {
                return None;
            }
            let value = slot.value.take();
            slot.generation = slot.generation.wrapping_add(1);
            self.free_list.push(handle.id as u32);
            value
        } else {
            None
        }
    }
}