use std::ops::{Deref, DerefMut};

/// A data wrapper, which tracks data change status.
pub struct Changeable<T> {
    /// Data wrapped
    pub data: T,
    is_changed: bool,
}

impl<T> Changeable<T> {
    pub fn new(data: T) -> Self {
        Self{
            data,
            is_changed: false,
        }
    }

    /// Is the data wrapped changed?
    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    /// Mark the data not changed.
    pub fn reset(&mut self) {
        self.is_changed = false;
    }
}

impl<T> Deref for Changeable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T>  DerefMut for Changeable<T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_changed = true;
        &mut self.data
    }
}