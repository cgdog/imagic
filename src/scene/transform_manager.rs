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
    pub fn add_transform(&mut self, transform: Transform) -> usize {
        let index: usize = self.transforms.len();
        self.transforms.push(transform);
        index
    }

    pub fn get_transform(&self, index: usize) -> &Transform {
        &self.transforms[index]
    }
}