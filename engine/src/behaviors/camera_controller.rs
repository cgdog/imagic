use std::f32::consts::PI;

use crate::{
    core::{LogicContext, NodeId},
    impl_as_any,
    math::{Vec3, spherical_coordinate::SphericalCoordinate},
    prelude::Behavior
};

/// The target of the camera controlled by the camera controller.
pub enum CameraTarget {
    /// The target is a node.
    Node(NodeId),
    /// The target is a position with type of Vec3.
    Position(Vec3),
}

/// A simple camera controller which is a node behavior.
/// 
/// - Press left mouse button and move mouse to rotate.
/// - Press right mouse button and move mouse to zoom.
/// - Press middle mouse button and move to pan.
pub struct CameraController {
    pub camera_node_id: NodeId,
    /// camera target
    pub target: CameraTarget,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub elevation: f32,
    pub min_elevation: f32,
    pub max_elevation: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,

    _spherical_coordinate: SphericalCoordinate,
    _pan_offset: Vec3,
}

impl CameraController {
    pub fn new(camera_node_id: NodeId, target: CameraTarget) -> Self {
        Self {
            camera_node_id,
            target,
            distance: 5.0,
            min_distance: 1.0,
            max_distance: 20.0,
            elevation: 0.0,
            min_elevation: -89.0_f32.to_radians(),
            max_elevation: 89.0_f32.to_radians(),
            rotate_speed: 0.15,
            zoom_speed: 1.0,
            pan_speed: 1.0,
            _spherical_coordinate: SphericalCoordinate::default(),
            _pan_offset: Vec3::ZERO,
        }
    }
}

impl Behavior for CameraController {
    impl_as_any!();

    fn on_start(&mut self, logic_context: &mut LogicContext) {
        log::info!("CameraController started.");
        if let Some(camera_node) = logic_context.world.current_scene().get_node(&self.camera_node_id) {
            self._spherical_coordinate = SphericalCoordinate::from_cartesian(camera_node.transform.position);
        }
    }
    fn on_update(&mut self, logic_context: &mut LogicContext) {
        
        if logic_context.input_manager.mouse_data.is_left_button_down { // rotate
            let cur_mouse_pos = &logic_context.input_manager.mouse_data.cur_mouse_pos;
            let last_mouse_pos = &logic_context.input_manager.mouse_data.last_mouse_pos;
            let delta_rotate_speed = self.rotate_speed * logic_context.time.delta();
            let delta_theta = (last_mouse_pos.y - cur_mouse_pos.y) * delta_rotate_speed;
            let delta_phi = (last_mouse_pos.x - cur_mouse_pos.x) * delta_rotate_speed;
            self._spherical_coordinate.theta += delta_theta;
            // Clamp the pitch (i.e., theta) to (0, 180) to avoid up vector relate bugs.
            self._spherical_coordinate.theta = self._spherical_coordinate.theta.clamp(0.01, PI * 0.99);
            self._spherical_coordinate.phi += delta_phi;
            let new_pos = self._spherical_coordinate.to_cartesian() + self.get_target_position_panned(logic_context);
            if let Some(camera) = logic_context.world.current_scene_mut().get_node_mut(&self.camera_node_id) {
                camera.transform.set_position(new_pos);
            }
            logic_context.input_manager.mouse_data.last_mouse_pos = logic_context.input_manager.mouse_data.cur_mouse_pos;
        } else if logic_context.input_manager.mouse_data.is_right_button_down { // zoom
            let delta_distance = logic_context.input_manager.mouse_data.cur_mouse_pos.y - logic_context.input_manager.mouse_data.last_mouse_pos.y;
            let delta_room = delta_distance * self.zoom_speed * logic_context.time.delta();
            self._spherical_coordinate.radius -= delta_room;
            let new_pos = self._spherical_coordinate.to_cartesian() + self.get_target_position_panned(logic_context);
            if let Some(camera) = logic_context.world.current_scene_mut().get_node_mut(&self.camera_node_id) {
                camera.transform.set_position(new_pos);
            }
            logic_context.input_manager.mouse_data.last_mouse_pos = logic_context.input_manager.mouse_data.cur_mouse_pos;
        } else if logic_context.input_manager.mouse_data.is_middle_button_down { // pan
            // let cur_mouse_pos = &(*self._input_manager_ptr).mouse_data.cur_mouse_pos;
            // let last_mouse_pos = &(*self._input_manager_ptr).mouse_data.last_mouse_pos;
            // let distance_delta = (cur_mouse_pos - last_mouse_pos) * self.pan_speed * _time.delta();
            // // TODO: _pan_offset should be in world space.
            // self._pan_offset.x += distance_delta.x;
            // self._pan_offset.y += distance_delta.y;
            // // let new_pos = self._spherical_coordinate.to_cartesian() + self.get_target_position_panned();
            // // camera_node.transform.set_position(new_pos);
        }
    }

    fn on_destroy(&mut self, _logic_context: &mut LogicContext) {
        log::info!("CameraController destroyed.");
    }
}

impl CameraController {
    fn get_target_position_panned(&self, logic_context: &mut LogicContext) -> Vec3 {
        match &self.target {
            CameraTarget::Node(target_node_id) => {
                if let Some(target_node) = logic_context.world.current_scene().get_node(target_node_id) {
                    self._pan_offset + target_node.transform.position
                } else {
                    log::warn!("Camera controller target {} is invalid", target_node_id);
                    Vec3::ZERO
                }
            },
            CameraTarget::Position(target_pos) => {
                self._pan_offset + target_pos
            },
        }
    }
}
