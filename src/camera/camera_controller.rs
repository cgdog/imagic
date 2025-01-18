use std::{cell::RefCell, f32::consts::{PI, TAU}, rc::Rc};

use crate::{
    input::{MouseEvent, MouseEventType, MouseInputListener},
    math::{Spherical, Vec3},
    scene::{SceneObject, TransformManager},
    types::{Dirtyable, ORR, RR},
};

use super::Camera;

struct MouseInputStatus {
    is_left_button_pressed: bool,
    is_right_button_pressed: bool,
    is_cursor_move: bool,

    start_x: f32,
    start_y: f32,
}

impl Default for MouseInputStatus {
    fn default() -> Self {
        Self {
            is_left_button_pressed: false,
            is_right_button_pressed: false,
            is_cursor_move: false,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
}

impl MouseInputStatus {
    fn delta_x(&self, cur_x: f32) -> f32 {
        cur_x - self.start_x
    }

    fn delta_y(&self, cur_y: f32) -> f32 {
        // Note here. In winit window, y: 0 is at the top and max positive value is at the bottom.
        self.start_y - cur_y
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

/// At presetn, only orbit control is supported.
/// 
/// We will support other camera control logic, e.g., Third Person Camera Controller, Free Camera Controller.
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
    /// True (default): enable the camera controller, False: disable it.
    pub is_enabled: bool,
    input_status: InputStatus,
    camera: ORR<Camera>,
    transform_manager: ORR<TransformManager>,
    target_pos: Vec3,
    init_spherical: Spherical,
    cur_spherical: Spherical,
    need_update_camera: bool,

    /// The phi (yaw) sensitivity
    pub phi_sensitivity: f32,
    /// The theta (pitch) sensitivity
    pub theta_sensitivity: f32,
    /// Camera zoom (pitch) sensitivity
    pub zoom_sensitivity: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            is_enabled: true,
            input_status: Default::default(),
            camera: None,
            transform_manager: None,
            target_pos: Vec3::ZERO,
            init_spherical: Spherical::default(),
            cur_spherical: Spherical::default(),
            need_update_camera: false,
            phi_sensitivity: 0.01,
            theta_sensitivity: 0.01,
            zoom_sensitivity: 0.01,
        }
    }
}

impl MouseInputListener for CameraController {
    fn on_mouse_input(&mut self, event: MouseEvent) {
        if !self.is_enabled {
            return;
        }

        self.need_update_camera = false;
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
                if self.input_status.mouse.is_left_button_pressed {
                    self.process_rotate(event);
                } else if self.input_status.mouse.is_right_button_pressed {
                    self.process_pan(event);
                }
            }
            MouseEventType::Scroll => {
                let delta_zoom = if event.x.abs() > event.y.abs() {
                    event.x
                } else {
                    -event.y
                };

                if !Self::is_delta_zoom_close_to_zero(delta_zoom) {
                    self.on_zoom(delta_zoom);
                    self.need_update_camera = true;
                }
            }
            _ => {}
        }
        if self.need_update_camera {
            self.update_camera_pos();
        }
    }
}

/// Camera controller implementation.
impl CameraController {
    const MIN_THETA: f32 = 0.001;
    const MAX_THETA: f32 = PI - 0.01;
    const MIN_RADIUS: f32 = 0.001;
    const EPSILON_THETA: f32 = 0.01;
    const EPSILON_PHI: f32 = 0.01;
    const EPSILON_ZOOM: f32 = 0.01;

    fn clamp_theta(&mut self) {
        self.cur_spherical.theta = self.cur_spherical.theta.clamp(Self::MIN_THETA, Self::MAX_THETA);
    }

    /// Limit phi in [0, TAU], to avoid precision errors.
    fn clamp_phi(&mut self) {
        if self.cur_spherical.phi < 0.0 {
            self.cur_spherical.phi += TAU;
        } else if self.cur_spherical.phi > TAU {
            self.cur_spherical.phi -= TAU;
        }
    }

    fn clamp_zoom(&mut self) {
        if self.cur_spherical.radius < Self::MIN_RADIUS {
            self.cur_spherical.radius = Self::MIN_RADIUS;
        }
    }

    fn process_rotate(&mut self, event: MouseEvent) {
        if !self.input_status.mouse.is_cursor_move {
            self.input_status.mouse.is_cursor_move = true;
            self.input_status.mouse.start_x = event.x;
            self.input_status.mouse.start_y = event.y;
            self.start_rotate();
        }

        let delta_yaw = self.input_status.mouse.delta_x(event.x);
        let delta_pitch = self.input_status.mouse.delta_y(event.y);
        
        if !Self::is_delta_theta_close_to_zero(delta_pitch) {
            self.on_pitch(delta_pitch);
            self.need_update_camera = true;
        }

        if !Self::is_delta_phi_close_to_zero(delta_yaw) {
            self.on_yaw(delta_yaw);
            self.need_update_camera = true;
        }
    }

    /// Pan camera target position, which moves positions of camera and target at the same time along the right andy up directions.
    /// TODO: compute world space postion delta from screen space delta, we will implement unproject() for Camera.
    fn process_pan(&mut self, _event: MouseEvent) {

    }
    
    /// Start to rotate camera's spherical coordinates. 
    /// 
    /// First transform camera's cartesian coordinates to spherical.
    fn start_rotate(&mut self) {
        let cartesian = self.get_camera_position();
        if let Some(camera_pos) = cartesian {
            let relative_camera_pos = camera_pos - self.target_pos;
            self.init_spherical.from_cartesian(relative_camera_pos);
            self.cur_spherical = self.init_spherical;
        }
    }

    pub fn new(
        camera: Rc<RefCell<Camera>>,
        options: CameraControllerOptions,
        transform_manager: RR<TransformManager>,
    ) -> Self {
        Self {
            camera: Some(camera),
            target_pos: options.target_pos,
            transform_manager: Some(transform_manager),
            ..Default::default()
        }
    }

    pub fn set_camera(&mut self, camera: Rc<RefCell<Camera>>) {
        self.camera = Some(camera);
    }

    fn is_delta_theta_close_to_zero(delta_theta: f32) -> bool {
        delta_theta.abs() < Self::EPSILON_THETA
    }

    fn is_delta_phi_close_to_zero(delta_phi: f32) -> bool {
        delta_phi.abs() < Self::EPSILON_PHI
    }

    fn is_delta_zoom_close_to_zero(delta_zoom: f32) -> bool {
        delta_zoom.abs() < Self::EPSILON_ZOOM
    }

    /// Rotate camera around x axis (Pitch)
    fn on_pitch(&mut self, delta_pitch: f32) {
        self.cur_spherical.theta = self.init_spherical.theta + delta_pitch * self.theta_sensitivity;
        self.clamp_theta();
    }

    fn on_yaw(&mut self, delta_yaw: f32) {
        self.cur_spherical.phi = self.init_spherical.phi + delta_yaw * self.phi_sensitivity;
        self.clamp_phi();
    }

    fn on_zoom(&mut self, delta_zoom: f32) {
        // note here. delta_zoom is different from delta_yaw and delta_pitch.
        self.cur_spherical.radius += delta_zoom * self.zoom_sensitivity;
        self.clamp_zoom();
    }

    fn get_camera_position(&self) -> Option<Vec3> {
        if let Some(camera) = &self.camera {
            let camera_transform_id = *camera.borrow().transform();
            if let Some(transform_manager) = &self.transform_manager {
                let camera_pos = transform_manager
                    .borrow()
                    .get_transform(camera_transform_id)
                    .position;
                return Some(camera_pos);
            }
        }
        None
    }

    fn update_camera_pos(&mut self) {
        if let Some(camera) = &self.camera {
            let camera_transform_id = *camera.borrow().transform();
            if let Some(transform_manager) = &self.transform_manager {
                let relative_camera_pos = self.cur_spherical.to_cartesian();
                let new_camera_pos = relative_camera_pos + self.target_pos;
                transform_manager.borrow_mut().get_transform_mut(camera_transform_id).set_position(new_camera_pos);
                camera.borrow_mut().set_dirty();
            }
        }
    }
}
