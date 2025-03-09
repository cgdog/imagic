
use crate::{math::Vec3, prelude::raytracer::material::Material};

use super::{HitResult, Primitive};

/// TODO: implement a general quad geometry.
/// 
/// At present, this special quad is in the plane whose equation is y = -4.
pub struct Quad {
    material: Material,
}

impl Quad {
    pub fn new(material: Material,) -> Self {
        Self {
            material,
        }
    }

    pub fn checker_pattern() -> Self {
        Self {
            material: Material::default(),
        }
    }
}

impl Primitive for Quad {
    fn ray_intersect(&mut self, orig: &Vec3, dir: &Vec3, cur_nearest_t: &mut f32) -> Option<HitResult> {
        if dir.y.abs() > 1e-3 {
            let d = -(orig.y + 4.0) / dir.y;
            let pt = orig + dir * d;
            if d > 0.0 && pt.x.abs() < 10.0 && pt.z < -10.0 && pt.z > -30.0 && d < *cur_nearest_t {
                *cur_nearest_t = d;
                let hit = pt;
                let normal = Vec3::new(0.0,1.0,0.0);
                self.material.diffuse_color = if (((0.5 * hit.x + 1000.0) as i32 + (0.5 * hit.z) as i32) & 1) != 0 {
                    Vec3::new(1.0,1.0,1.0)
                } else {
                    Vec3::new(1.0, 0.7, 0.3)
                };
                self.material.diffuse_color = self.material.diffuse_color * 0.3;
                let hit_result = HitResult::new(hit, normal, self.material);
                return Some(hit_result);
            }
        }
        return None;
    }
}