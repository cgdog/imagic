// use super::transform::Transform;

use crate::{camera::Layer, prelude::render_item_manager::RenderItemManager};

pub trait SceneObject {
    fn transform(&self) -> &usize;
    
    fn get_layer(&self) -> Layer;
    fn set_layer(&mut self, layer: Layer, render_item_manager: &mut RenderItemManager);

    // fn transform_mut(&mut self) -> &mut Transform;

    // fn is_renderable(&self) -> bool {
    //     self.get_render_item_id() != INVALID_ID
    // }

    // fn get_render_item_id(&self) -> usize {
    //     INVALID_ID
    // }

    // fn get_position(&self) -> &glam::Vec3 {
    //     self.transform().get_position()
    // }

    // fn set_position(&mut self, new_pos: glam::Vec3) {
    //     self.transform_mut().set_position(new_pos);
    // }

    // fn get_rotation_euler(&self) -> &glam::Vec3 {
    //     self.transform().get_rotation_euler()
    // }

    // fn get_rotation_quat(&self) -> &glam::Quat {
    //     self.transform().get_rotation_quat()
    // }

    // fn set_rotation_euler(&mut self, new_rot: glam::Vec3) {
    //     self.transform_mut().set_rotation_euler(new_rot);
    // }

    // fn get_scale(&self) -> &glam::Vec3 {
    //     self.transform().get_scale()
    // }

    // fn set_scale(&mut self, new_scale: glam::Vec3) {
    //     self.transform_mut().set_scale(new_scale);
    // }

    // fn trs_matrix(&self) -> glam::Mat4 {
    //     self.transform().trs_matrix()
    // }
}