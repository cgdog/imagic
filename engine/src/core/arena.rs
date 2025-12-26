use crate::types::Handle;

struct ArenaSlot<T> {
    value: Option<T>,
    generation: u32,
}

pub struct Arena<T> {
    slots: Vec<ArenaSlot<T>>,
    free_list: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn create(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index as usize];
            let handle = Handle::new_with_generation(index as u64, slot.generation);
            slot.value = Some(value);
            slot.generation += 1;
            handle
        } else {
            let id = self.slots.len() as u64;
            let handle = Handle::new(id);
            self.slots.push(ArenaSlot { value: Some(value), generation: 0 });
            handle
        }
    }
}