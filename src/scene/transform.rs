use crate::math::{EulerRot, Mat4, Quat, Vec3};

pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub quat: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            quat: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn set_position(&mut self, new_pos: Vec3) {
        self.position = new_pos;
    }

    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn set_rotation_euler(&mut self, new_rot: Vec3) {
        self.rotation = new_rot;
        self.quat = Quat::from_euler(EulerRot::YXZ, self.rotation.x, self.rotation.y, self.rotation.z);
    }

    pub fn get_rotation_euler(&self) -> &Vec3 {
        &self.rotation
    }

    pub fn get_rotation_quat(&self) -> &Quat {
        &self.quat
    }

    pub fn set_scale(&mut self, new_scale: Vec3) {
        self.scale = new_scale;
    }

    pub fn get_scale(&self) -> &Vec3 {
        &self.scale
    }

    pub fn trs_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.quat, self.position)
    }
}