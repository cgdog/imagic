use std::{cell::RefCell, rc::Rc};

use log::info;

use crate::{
    input::InputManager,
    prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager,
        buffer::GPUBufferManager, texture_manager::TextureManager, GraphicsContext,
        TransformManager, INVALID_ID,
    },
    types::{ID, RR},
    window::Window,
};

use super::{camera::Camera, CameraController, CameraControllerOptions};

pub struct CameraManager {
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
    pub fn add_camera(&mut self, camera: Camera) -> ID {
        let index = self.cameras.len();
        self.cameras.push(Rc::new(RefCell::new(camera)));
        index
    }

    pub fn get_camera(&self, index: usize) -> RR<Camera> {
        self.cameras[index].clone()
    }

    pub fn get_cameras(&self) -> &Vec<RR<Camera>> {
        &self.cameras
    }

    pub fn init_after_app(
        &mut self,
        window: &Window,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &mut BindGroupLayoutManager,
        transform_manager: RR<TransformManager>,
        buffer_manager: &mut GPUBufferManager,
        texture_manager: &mut TextureManager,
        input_manager: &mut InputManager,
    ) {
        for camera in self.cameras.iter() {
            camera.borrow_mut().init_after_app(
                window,
                graphics_context,
                bind_group_manager,
                bind_group_layout_manager,
                &transform_manager.borrow(),
                buffer_manager,
                texture_manager,
            );

            let mut controller_id = INVALID_ID;
            if let Some(camera_controller_options) = camera.borrow().controller_options {
                let camera_controller =
                    CameraController::new(camera.clone(), camera_controller_options, transform_manager.clone());
                controller_id = input_manager.register_mouse_input_listener(Box::new(camera_controller));
            }
            camera.borrow_mut().controller_id = controller_id;
        }
    }

    pub fn on_update(
        &mut self,
        graphics_context: &GraphicsContext,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
    ) {
        for camera in self.cameras.iter() {
            camera.borrow_mut().on_update(graphics_context, transform_manager, buffer_manager);
        }
    }

    pub fn on_resize(
        &mut self,
        graphics_context: &GraphicsContext,
        texture_manager: &mut TextureManager,
        transform_manager: &TransformManager,
        buffer_manager: &GPUBufferManager,
        width: u32,
        height: u32,
    ) {
        for camera in self.cameras.iter() {
            camera.borrow_mut().on_resize(
                graphics_context,
                texture_manager,
                transform_manager,
                buffer_manager,
                width,
                height,
            );
        }
    }

    /// Change camera controller given a camera id.
    pub fn change_camera_controller(&mut self, input_manager: &mut InputManager, camera_id: ID, camera_controller_options: &CameraControllerOptions) {
        let controller_id = self.get_camera(camera_id).borrow().controller_id;
        info!("change control id: {}", controller_id);
        if let Some(input_listener) = input_manager.get_input_listener(controller_id) {
            if let Some(controller) =input_listener.as_any_mut().downcast_mut::<CameraController>() {
                controller.options = *camera_controller_options;
            }
        }
    }
}
