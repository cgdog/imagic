use crate::prelude::{SceneObject, Transform, TransformManager};

pub struct PointLight {
    color: glam::Vec3,
    intensity: f32,
    transform: usize,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: glam::Vec3::ONE,
            intensity: 1.0,
            transform: usize::MAX,
        }
    }
}

impl SceneObject for PointLight {
    fn transform(&self) -> &usize {
        &self.transform
    }

    // fn transform_mut(&mut self) -> &mut Transform {
    //     &mut self.transform
    // }
}

impl PointLight {
    pub fn new(pos: glam::Vec3, color: glam::Vec3, transform_manager: &mut TransformManager) -> Self {
        let mut  transform = Transform::default();
        transform.set_position(pos);
        let transform_index = transform_manager.add_transform(transform);

        Self {
            color,
            intensity: 1.0,
            transform: transform_index,
        }
    }

    pub fn get_color(&self) -> &glam::Vec3 {
        &self.color
    }

    pub fn set_color(&mut self, color: glam::Vec3) {
        self.color = color;
    }

    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
}