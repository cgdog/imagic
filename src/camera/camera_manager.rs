use crate::prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, buffer::GPUBufferManager, GraphicsContext, TransformManager};

use super::camera::Camera;

pub struct CameraManager {
    cameras: Vec<Camera>,
}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            cameras: Vec::new(),
        }
    }
}

impl CameraManager {
    pub fn add_camera(&mut self, camera: Camera) -> usize {
        let index = self.cameras.len();
        self.cameras.push(camera);
        index
    }

    pub fn get_camera(&self, index: usize) -> &Camera {
        &self.cameras[index]
    }

    pub fn get_camera_mut(&mut self, index: usize) -> &mut Camera {
        &mut self.cameras[index]
    }

    pub fn init_after_app(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager
        , bind_group_layout_manager: &mut BindGroupLayoutManager, transform_manager: &TransformManager, buffer_manager: &mut GPUBufferManager) {
        for camera in self.cameras.iter_mut() {
            camera.init_after_app(graphics_context, bind_group_manager, bind_group_layout_manager, transform_manager, buffer_manager);
        }
    }
}