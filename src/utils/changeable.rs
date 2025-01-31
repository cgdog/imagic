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

    /// Mark the data has changed.
    pub fn set(&mut self) {
        self.is_changed = true;
    }

    /// Mark the data not changed.
    pub fn reset(&mut self) {
        self.is_changed = false;
    }

    /// Is the data wrapped changed?
    pub fn is_changed(&self) -> bool {
        self.is_changed
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
        &mut self.data
    }
}