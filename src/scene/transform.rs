pub struct Transform {
    position: glam::Vec3,
    rotation: glam::Vec3,
    quat: glam::Quat,
    scale: glam::Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Vec3::ZERO,
            quat: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn set_position(&mut self, new_pos: glam::Vec3) {
        self.position = new_pos;
        
    }

    pub fn get_position(&self) -> &glam::Vec3 {
        &self.position
    }

    pub fn set_rotation_euler(&mut self, new_rot: glam::Vec3) {
        self.rotation = new_rot;
        self.quat = glam::Quat::from_euler(glam::EulerRot::YXZ, self.rotation.x, self.rotation.y, self.rotation.z);
    }

    pub fn get_rotation_euler(&self) -> &glam::Vec3 {
        &self.rotation
    }

    pub fn get_rotation_quat(&self) -> &glam::Quat {
        &self.quat
    }

    pub fn set_scale(&mut self, new_scale: glam::Vec3) {
        self.scale = new_scale;
    }

    pub fn get_scale(&self) -> &glam::Vec3 {
        &self.scale
    }

    pub fn trs_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.quat, self.position)
    }
}