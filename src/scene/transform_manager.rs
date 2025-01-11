use crate::types::ID;

use super::transform::Transform;

pub struct TransformManager {
    transforms: Vec<Transform>,
}

impl Default for TransformManager {
    fn default() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }
}

impl TransformManager {
    pub fn add_transform(&mut self, transform: Transform) -> ID {
        let index: ID = self.transforms.len();
        self.transforms.push(transform);
        index
    }

    pub fn get_transform(&self, index: ID) -> &Transform {
        &self.transforms[index]
    }

    pub fn get_transform_mut(&mut self, index: ID) -> &mut Transform {
        &mut self.transforms[index]
    }
}