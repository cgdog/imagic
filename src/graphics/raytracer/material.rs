use crate::math::{ColorRGB, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub albedo: Vec4,
    pub diffuse_color: ColorRGB,
    pub specular_exponent: f32,
    pub refractive_index: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec4::new(1.0, 0.0, 0.0, 0.0),
            diffuse_color: ColorRGB::ZERO,
            specular_exponent: 1.0,
            refractive_index: 1.0,
        }
    }
}

impl Material {
    pub fn new(refractive_index: f32, albedo: Vec4, diffuse_color: ColorRGB, specular_exponent: f32) -> Self {
        Self {
            refractive_index,
            albedo,
            diffuse_color,
            specular_exponent,
        }
    }
}