use std::fmt::Display;

use crate::{impl_component, math::{EulerRot, Mat3, Mat4, Quat, Vec3}};

/// A component that represents the transform of a node.
/// Every [`Node`] instance has a [`Transform`] component.
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    /// The position of the transform.
    pub position: Vec3,
    /// The rotation of the transform in euler angles (in radians).
    /// x: yaw
    /// y: pitch
    /// z: roll
    pub rotation: Vec3,
    /// The scale of the transform.
    pub scale: Vec3,
    /// The quaternion of the transform. It will be updated when [`rotation`](Self::rotation) is changed.
    pub(crate) quat: Quat,

    /// The model matrix of the transform.
    pub model_matrix: Mat4,
    /// The normal matrix of the transform.
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
    /// Set the position of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `new_pos` - The new position of the transform.
    pub fn set_position(&mut self, new_pos: Vec3) {
        self.position = new_pos;
        self.is_dirty = true;
    }

    /// Set the x component of the position of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `pos_x` - The new x component of the position of the transform.
    pub fn set_position_x(&mut self, pos_x: f32) {
        self.position.x = pos_x;
        self.is_dirty = true;
    }

    /// Set the y component of the position of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `pos_y` - The new y component of the position of the transform.
    pub fn set_position_y(&mut self, pos_y: f32) {
        self.position.y = pos_y;
        self.is_dirty = true;
    }

    /// Set the z component of the position of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `pos_z` - The new z component of the position of the transform.
    pub fn set_position_z(&mut self, pos_z: f32) {
        self.position.z = pos_z;
        self.is_dirty = true;
    }

    /// Get the position of the transform.
    /// 
    /// # Returns
    /// 
    /// * `&Vec3` - The position of the transform.
    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    /// Set the rotation of the transform in euler angles (in radians). The rotation sequence is yaw-pitch-roll(i.e., YXZ).
    /// 
    /// A quaternion will be updated when this method is called. This quaternion is used to construct the model matrix.
    /// 
    /// # Arguments
    /// 
    /// * `new_rot` - The new rotation of the transform in euler angles (in radians).
    /// x: yaw
    /// y: pitch
    /// z: roll
    pub fn set_rotation_euler(&mut self, new_rot: Vec3) {
        self.rotation = new_rot;
        self.quat = Quat::from_euler(EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z);
        self.is_dirty = true;
    }

    /// Set the rotation of the transform in euler angles (in radians). The rotation sequence is pitch-yaw-roll(i.e., XYZ).
    /// 
    /// A quaternion will be updated when this method is called. This quaternion is used to construct the model matrix.
    /// 
    /// You maybe use this method when setting up a spot light (see spot_light_demo.rs).
    /// 
    /// # Arguments
    /// 
    /// * `new_rot` - The new rotation of the transform in euler angles (in radians).
    /// x: roll
    /// y: pitch
    /// z: yaw
    pub fn set_rotation_euler_by_xyz(&mut self, new_rot: Vec3) {
        self.rotation = new_rot;
        self.quat = Quat::from_euler(EulerRot::XYZ, self.rotation.x, self.rotation.y, self.rotation.z);
        self.is_dirty = true;
    }

    /// Get the rotation of the transform in euler angles (in radians).
    /// 
    /// # Returns
    /// 
    /// * `&Vec3` - The rotation (yaw-pitch-roll) of the transform in euler angles (in radians).
    pub fn get_rotation_euler(&self) -> &Vec3 {
        &self.rotation
    }

    /// Get the rotation of the transform in quaternion.
    /// 
    /// # Returns
    /// 
    /// * `&Quat` - The rotation of the transform in quaternion.
    pub fn get_rotation_quat(&self) -> &Quat {
        &self.quat
    }

    /// Set the rotation of the transform in quaternion which contrubuting to construct the model matrix.
    /// 
    /// The methods [`set_rotation_euler`] and [`set_rotation_euler_by_xyz`] are recommended to use instead of this method.
    /// 
    /// Only use this method when you know what you are doing.
    /// 
    /// # Arguments
    /// 
    /// * `quat` - The new rotation of the transform in quaternion.
    pub fn set_rotation_quat(&mut self, quat: Quat) {
        self.quat = quat;
        self.is_dirty = true;
    }

    /// Set the scale of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `new_scale` - The new scale of the transform.
    pub fn set_scale(&mut self, new_scale: Vec3) {
        self.scale = new_scale;
        self.is_dirty = true;
    }

    /// Set the uniform scale of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `new_scale` - The new uniform scale of the transform.
    pub fn set_uniform_scale(&mut self, new_scale: f32) {
        self.scale.x *= new_scale;
        self.scale.y *= new_scale;
        self.scale.z *= new_scale;
        self.is_dirty = true;
    }

    /// Get the scale of the transform.
    /// 
    /// # Returns
    /// 
    /// * `&Vec3` - The scale of the transform.
    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    /// Set the position, rotation and scale of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `pos` - The new position of the transform.
    /// * `rot` - The new rotation (yaw-pitch-roll, by order YXZ) of the transform in euler angles (in radians).
    /// * `scale` - The new scale of the transform.
    pub fn set_position_rotation_scale(&mut self, pos: Vec3, rot: Vec3, scale: Vec3) {
        self.position = pos;
        self.rotation = rot;
        self.quat = Quat::from_euler(EulerRot::YXZ, self.rotation.y, self.rotation.x, self.rotation.z);
        self.scale = scale;
        self.is_dirty = true;
    }

    /// Set the position, rotation and scale of the transform from arrays.
    /// 
    /// # Arguments
    /// 
    /// * `pos` - The new position of the transform.
    /// * `rot_quat` - The new rotation (quaternion) of the transform.
    /// * `scale` - The new scale of the transform.
    pub fn set_position_rotation_scale_from_arrays(&mut self, pos: [f32; 3], rot_quat: [f32; 4], scale: [f32; 3]) {
        self.position = Vec3::from_array(pos);
        self.quat = Quat::from_array(rot_quat);
        let (rot_x, rot_y, rot_z) = self.quat.to_euler(EulerRot::YXZ);
        self.rotation = Vec3::new(rot_x, rot_y, rot_z);
        self.scale = Vec3::from_array(scale);
        self.is_dirty = true;
    }

    /// Update the model matrix of the transform.
    /// 
    /// # Arguments
    /// 
    /// * `parent` - The parent matrix of the transform.
    pub(crate) fn update_model_matrix(&mut self, parent: Option<Mat4>) {
        if self.is_dirty {
            self.model_matrix = Mat4::from_scale_rotation_translation(self.scale, self.quat, self.position);
            self.is_dirty = false;
        }
        if let Some(parent_matrix) = parent {
            self.model_matrix = parent_matrix * self.model_matrix;
        }
        self.normal_matrix = Mat3::from_mat4(self.model_matrix.inverse().transpose());
    }

}