use std::{cell::RefCell, rc::Rc};

use log::info;

use crate::{
    input::{MouseEvent, MouseEventType, MouseInputListener},
    math::Vec3,
    scene::{SceneObject, TransformManager},
    types::{ORR, RR},
};

use super::Camera;

struct MouseInputStatus {
    is_left_button_pressed: bool,
    is_right_button_pressed: bool,
    is_cursor_move: bool,

    start_x: f64,
    start_y: f64,
}

impl Default for MouseInputStatus {
    fn default() -> Self {
        Self {
            is_left_button_pressed: false,
            is_right_button_pressed: false,
            is_cursor_move: false,
            start_x: f64::default(),
            start_y: f64::default(),
        }
    }
}

impl MouseInputStatus {
    fn delta_x(&self, cur_x: f64) -> f64 {
        cur_x - self.start_x
    }

    fn delta_y(&self, cur_y: f64) -> f64 {
        cur_y - self.start_y
    }
}

struct InputStatus {
    mouse: MouseInputStatus,
}

impl Default for InputStatus {
    fn default() -> Self {
        Self {
            mouse: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraControllerOptions {
    pub target_pos: Vec3,
    pub radius: f64,
}

impl Default for CameraControllerOptions {
    fn default() -> Self {
        Self {
            target_pos: Vec3::ZERO,
            radius: 1.0,
        }
    }
}

impl CameraControllerOptions {
    pub fn new(target_pos: Vec3, radius: f64) -> Self {
        Self { target_pos, radius }
    }
}

pub struct CameraController {
    pub is_enabled: bool,
    input_status: InputStatus,
    target_pos: Vec3,
    radius: f64,
    camera: ORR<Camera>,
    transform_manager: ORR<TransformManager>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            is_enabled: false,
            input_status: Default::default(),
            target_pos: Vec3::ZERO,
            radius: 1.0,
            camera: None,
            transform_manager: None,
        }
    }
}

impl MouseInputListener for CameraController {
    fn on_mouse_input(&mut self, event: MouseEvent) {
        match event.event_type {
            MouseEventType::LeftPressed => {
                self.input_status.mouse.is_left_button_pressed = true;
            }
            MouseEventType::LeftReleased => {
                self.input_status.mouse.is_left_button_pressed = false;
                self.input_status.mouse.is_cursor_move = false;
            }
            MouseEventType::RightPressed => {
                self.input_status.mouse.is_right_button_pressed = true;
            }
            MouseEventType::RightReleased => {
                self.input_status.mouse.is_right_button_pressed = false;
            }
            MouseEventType::Move => {
                if !self.input_status.mouse.is_cursor_move {
                    self.input_status.mouse.is_cursor_move = true;
                    self.input_status.mouse.start_x = event.x;
                    self.input_status.mouse.start_y = event.y;
                    info!(
                        "start_x: {}, start_y: {}",
                        self.input_status.mouse.start_x, self.input_status.mouse.start_y
                    );
                }
                if self.input_status.mouse.is_left_button_pressed {
                    let delta_yaw = self.input_status.mouse.delta_x(event.x);
                    let delta_pitch = self.input_status.mouse.delta_y(event.y);
                    self.on_pitch(delta_pitch);
                    self.on_yaw(delta_yaw);
                }
            }
            _ => {}
        }
    }
}

/// Camera controller implementation.
impl CameraController {
    pub fn new(
        camera: Rc<RefCell<Camera>>,
        options: CameraControllerOptions,
        transform_manager: RR<TransformManager>,
    ) -> Self {
        Self {
            camera: Some(camera),
            target_pos: options.target_pos,
            radius: options.radius,
            transform_manager: Some(transform_manager),
            ..Default::default()
        }
    }

    pub fn set_camera(&mut self, camera: Rc<RefCell<Camera>>) {
        self.camera = Some(camera);
    }

    fn on_pitch(&mut self, delta_pitch: f64) {
        if let Some(camera) = &self.camera {
            let camera_transform_id = *camera.borrow().transform();
            info!("camera transform id: {camera_transform_id}, delta_pitch: {delta_pitch}");
            if let Some(transform_manager) = &self.transform_manager {
                let mut last_rotation_eulr = transform_manager
                    .borrow()
                    .get_transform(camera_transform_id)
                    .rotation;
                last_rotation_eulr.x += delta_pitch as f32;
                transform_manager
                    .borrow_mut()
                    .get_transform_mut(camera_transform_id)
                    .set_rotation_euler(last_rotation_eulr);
                // camera.borrow_mut().update_uniform_buffers(graphics_context, &transform_manager.borrow(), buffer_manager);
            }
        }
    }
    fn on_yaw(&mut self, delta_yaw: f64) {
        info!("delta_yaw: {delta_yaw}");
    }
}
