use crate::{math::Vec3, prelude::raytracer::material::Material};

use super::{HitResult, Primitive};

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material,) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Primitive for Sphere {
    fn ray_intersect(&mut self, orig: &Vec3, dir: &Vec3, cur_nearest_t: &mut f32) -> Option<HitResult> {
        let l: Vec3 = self.center - orig;
        let tca = l.dot(*dir);
        let d2 = l.dot(l) - tca * tca;
        let radius2 = self.radius * self.radius;
        if d2 > radius2 {
            return None;
        }

        let thc = (radius2 - d2).sqrt();
        let mut t0 = tca - thc;
        let t1 = tca + thc;

        if t0 < 0.0 {
            t0 = t1;
        }

        if t0 < 0.0 || t0 >= *cur_nearest_t {
            return None;
        }

        *cur_nearest_t = t0;
        let position = orig + dir * t0;
        let normal = (position - self.center).normalize();
        let hit_result = HitResult::new(position, normal, self.material);
        return Some(hit_result);
    }
}