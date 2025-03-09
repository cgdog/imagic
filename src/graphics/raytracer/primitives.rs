use crate::math::Vec3;

use super::material::Material;

pub mod sphere;
pub mod model;
pub mod quad;

#[derive(Debug, Clone, Copy)]
pub struct HitResult {
    pub position: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl HitResult {
    pub fn new(position: Vec3, normal: Vec3, material: Material,) -> Self {
        Self {
            position,
            normal,
            material,
        }
    }
}

pub trait Primitive {
    fn ray_intersect(&mut self, orig: &Vec3, dir: &Vec3, cur_nearest_t: &mut f32) -> Option<HitResult>;
}