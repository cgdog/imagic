use std::fmt::Display;

use crate::{impl_component, math::{EulerRot, Mat3, Mat4, Quat, Vec3}};

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub quat: Quat,

    pub model_matrix: Mat4,
    pub normal_matrix: Mat3,
    
    /// A flag that indicates the Transform is dirty.
    /// Engine will recompute [`model_matrix`](Self::model_matrix) when it is true and reset it to be false.
    /// You need to make this field be true when you change other fields, e.g., [`position`](Self::position).
    pub(crate) is_dirty: bool,
}

impl_component!(Transform);

impl Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "position: ({}, {}, {}), rotation: ({}, {}, {}), scale: ({}, {}, {})", self.position.x, self.position.y, self.position.z,
        self.rotation.x, self.rotation.y, self.rotation.z, self.scale.x, self.scale.y, self.scale.z)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            quat: Quat::IDENTITY,
            scale: Vec3::ONE,
            is_dirty: true,
            model_matrix: Mat4::IDENTITY,
            normal_matrix: Mat3::IDENTITY,
        }
    }
}

impl Transform {
    pub fn set_position(&mut self, new_pos: Vec3) {
        self.position = new_pos;
        self.is_dirty = true;
    }

    pub fn set_position_x(&mut self, pos_x: f32) {
        self.position.x = pos_x;
        self.is_dirty;
    }

    pub fn set_position_y(&mut self, pos_y: f32) {
        self.position.y = pos_y;
        self.is_dirty;
    }

    pub fn set_position_z(&mut self, pos_z: f32) {
        self.position.z = pos_z;
        self.is_dirty;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_rotation_euler(&mut self, new_rot: Vec3) {
        self.rotation = new_rot;
        self.quat = Quat::from_euler(EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z);
        self.is_dirty = true;
    }

    pub fn get_rotation_euler(&self) -> &Vec3 {
        &self.rotation
    }

    pub fn get_rotation_quat(&self) -> &Quat {
        &self.quat
    }

    pub fn set_scale(&mut self, new_scale: Vec3) {
        self.scale = new_scale;
        self.is_dirty = true;
    }

    pub fn set_uniform_scale(&mut self, new_scale: f32) {
        self.scale.x *= new_scale;
        self.scale.y *= new_scale;
        self.scale.z *= new_scale;
        self.is_dirty = true;
    }

    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn set_position_rotation_scale(&mut self, pos: Vec3, rot: Vec3, scale: Vec3) {
        self.position = pos;
        self.rotation = rot;
        self.quat = Quat::from_euler(EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z);
        self.scale = scale;
        self.is_dirty = true;
    }

    pub fn set_position_rotation_scale_from_arrays(&mut self, pos: [f32; 3], rot_quat: [f32; 4], scale: [f32; 3]) {
        self.position = Vec3::from_array(pos);
        self.quat = Quat::from_array(rot_quat);
        let (rot_x, rot_y, rot_z) = self.quat.to_euler(EulerRot::YXZ);
        self.rotation = Vec3::new(rot_x, rot_y, rot_z);
        self.scale = Vec3::from_array(scale);
        self.is_dirty = true;
    }

    pub(crate) fn update_model_matrix(&mut self, parent: Option<Mat4>) {
        self.model_matrix = Mat4::from_scale_rotation_translation(self.scale, self.quat, self.position);
        if let Some(parent_matrix) = parent {
            self.model_matrix = parent_matrix * self.model_matrix;
        }
        self.normal_matrix = Mat3::from_mat4(self.model_matrix.inverse().transpose());
    }

}