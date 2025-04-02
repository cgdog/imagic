use std::{cell::RefCell, rc::Rc};

use log::info;

use crate::{
    asset::asset_manager::AssetManager, input::InputManager, prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
        buffer::GPUBufferManager, GraphicsContext,
        TransformManager, INVALID_ID,
    }, types::{ID, RR}, window::WindowSize
};

use super::{camera::Camera, CameraController, CameraControllerOptions};

pub struct CameraManager {
    /// Some other struct (e.g., CameraController) also holds camera instance.
    /// TODO: may be we can avoid this. and remove RR(RC::RefCell) by utilizing Update method.
    /// (Let CameraController only holds camera id).
    cameras: Vec<RR<Camera>>,
}

impl Default for CameraManager {
    fn default() -> Self {
        Self {
            cameras: Vec::new(),
        }
    }
}

impl CameraManager {
    pub fn add_camera(
        &mut self,
        camera: Camera,
        logical_size: &WindowSize,
        physical_size: &WindowSize,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: RR<TransformManager>,
        buffer_manager: &mut GPUBufferManager,
        asset_manager: &mut AssetManager,
        input_manager: &mut InputManager,
    ) -> ID {
        let camera = Rc::new(RefCell::new(camera));
        camera.borrow_mut().on_init(
            logical_size,
            physical_size,
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
            &transform_manager.borrow(),
            buffer_manager,
            asset_manager,
        );

        let mut controller_id = INVALID_ID;
        if let Some(camera_controller_options) = camera.borrow().controller_options {
            let camera_controller = CameraController::new(
                camera.clone(),
                camera_controller_options,
                transform_manager.clone(),
            );
            controller_id =
                input_manager.register_mouse_input_listener(Box::new(camera_controller));
        }
        camera.borrow_mut().controller_id = controller_id;
        let index = self.cameras.len();
        self.cameras.push(camera);
        index
    }

    pub fn get_camera(&self, index: usize) -> RR<Camera> {
        self.cameras[index].clone()
    }

    pub fn get_cameras(&self) -> &Vec<RR<Camera>> {
        &self.cameras
    }

    pub fn on_update(
        &mut self,
        graphics_context: &GraphicsContext,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
    ) {
        for camera in self.cameras.iter() {
            camera
                .borrow_mut()
                .on_update(graphics_context, transform_manager, buffer_manager);
        }
    }

    pub fn on_resize(
        &mut self,
        graphics_context: &GraphicsContext,
        asset_manager: &mut AssetManager,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
        physical_width: u32,
        physical_height: u32,
        logical_width: u32,
        logical_height: u32,
    ) {
        for camera in self.cameras.iter() {
            camera.borrow_mut().on_resize(
                graphics_context,
                asset_manager,
                transform_manager,
                buffer_manager,
                physical_width,
                physical_height,
                logical_width,
                logical_height,
            );
        }
    }

    /// Change camera controller given a camera id.
    pub fn change_camera_controller(
        &mut self,
        input_manager: &mut InputManager,
        camera_id: ID,
        camera_controller_options: &CameraControllerOptions,
    ) {
        let controller_id = self.get_camera(camera_id).borrow().controller_id;
        info!("change control id: {}", controller_id);
        if let Some(input_listener) = input_manager.get_input_listener(controller_id) {
            if let Some(controller) = input_listener
                .as_any_mut()
                .downcast_mut::<CameraController>()
            {
                controller.options = *camera_controller_options;
            }
        }
    }
}
