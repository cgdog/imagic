use crate::{
    camera::Layer, math::Vec3, prelude::{
        render_item_manager::RenderItemManager, SceneObject, Transform, TransformManager,
        INVALID_ID,
    }
};

pub struct PointLight {
    color: Vec3,
    intensity: f32,
    transform: usize,

    layer: Layer,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            color: Vec3::ONE,
            intensity: 1.0,
            transform: INVALID_ID,
            layer: Layer::Default,
        }
    }
}

impl SceneObject for PointLight {
    fn transform(&self) -> &usize {
        &self.transform
    }

    fn get_layer(&self) -> Layer {
        self.layer
    }

    fn set_layer(&mut self, layer: Layer, _render_item_manager: &mut RenderItemManager) {
        self.layer = layer;
        // render_item_manager.get_render_item_mut(self.render_item_id).layer = layer;
    }

    // fn transform_mut(&mut self) -> &mut Transform {
    //     &mut self.transform
    // }
}

impl PointLight {
    pub fn new(
        pos: Vec3,
        color: Vec3,
        transform_manager: &mut TransformManager,
    ) -> Self {
        let mut transform = Transform::default();
        transform.set_position(pos);
        let transform_index = transform_manager.add_transform(transform);

        Self {
            color,
            intensity: 1.0,
            transform: transform_index,
            layer: Layer::Default,
        }
    }

    pub fn get_color(&self) -> &Vec3 {
        &self.color
    }

    pub fn set_color(&mut self, color: Vec3) {
        self.color = color;
    }

    pub fn get_intensity(&self) -> f32 {
        self.intensity
    }

    pub fn set_intensity(&mut self, intensity: f32) {
        self.intensity = intensity;
    }
}
