use core::f32;
use std::f32::consts::FRAC_PI_3;

use glam::Vec2;
use image::{ImageBuffer, Rgb};

use crate::math::{ColorRGB, Vec3};

use super::{
    light::Light,
    primitives::{HitResult, Primitive},
};

/// Ray tracer
pub struct RayTracer {
    width: u32,
    height: u32,
    env_map: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
}

impl RayTracer {
    pub fn new(width: u32, height: u32, env_map: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>) -> Self {
        Self {
            width,
            height,
            env_map,
        }
    }

    pub fn scene_intersect(
        &self,
        orig: &Vec3,
        dir: &Vec3,
        items: &mut Vec<Box<dyn Primitive>>,
        // hit: &mut Vec3,
        // n: &mut Vec3,
        // material: &mut Material,
    ) -> Option<HitResult> {
        let mut cur_nearest_t = f32::INFINITY;
        let mut result: Option<HitResult> = None;
        for item in items {
            if let Some(hit_result) = item.ray_intersect(orig, dir, &mut cur_nearest_t) {
                // *hit = hit_result.position;
                // *n = hit_result.normal;
                // *material = hit_result.material;
                result = Some(hit_result);
            }
        }

        if cur_nearest_t < 1000.0 {
            result
        } else {
            None
        }
    }

    fn reflect(&self, dir: &Vec3, normal: &Vec3) -> Vec3 {
        dir - normal * 2.0 * dir.dot(*normal)
    }

    fn refract(&self, incident: &Vec3, normal: &Vec3, refractive_index: f32) -> Vec3 {
        // Snell's law
        let mut cosi = -incident.dot(*normal).min(1.0).max(-1.0); //- std::max(-1.f, std::min(1.f, I*N));
        let mut etai = 1.0;
        let mut etat = refractive_index;
        let mut n = *normal;
        if cosi < 0.0 {
            // if the ray is inside the object, swap the indices and invert the normal to get the correct result
            cosi = -cosi;
            let tmp = etai;
            etai = etat;
            etat = tmp;
            n = -n;
        }
        let eta = etai / etat;
        let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
        if k < 0.0 {
            Vec3::ZERO
        } else {
            incident * eta + n * (eta * cosi - k.sqrt())
        }
    }

    fn sample_spherical_map(
        &self,
        dir: &Vec3,
        env_map: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    ) -> ColorRGB {
        let inv_atan = Vec2::new(0.1591, 0.3183);
        let mut uv = Vec2::new(dir.z.atan2(dir.x), dir.y.asin());
        uv *= inv_atan;
        uv += 0.5;
        let (width, height) = env_map.dimensions();
        let x = (uv.x * width as f32) as u32;
        let y = (uv.y * height as f32) as u32;
        let pixel_color = env_map.get_pixel(x, y);
        let color = ColorRGB::new(
            pixel_color[0] as f32 / 255.0,
            pixel_color[1] as f32 / 255.0,
            pixel_color[2] as f32 / 255.0,
        );
        return color;
    }

    fn cast_ray(
        &self,
        orig: &Vec3,
        dir: &Vec3,
        spheres: &mut Vec<Box<dyn Primitive>>,
        lights: &Vec<Light>,
        depth: u32,
    ) -> ColorRGB {
        if depth > 4 {
            if let Some(env_map) = &self.env_map {
                self.sample_spherical_map(dir, env_map)
            } else {
                ColorRGB::new(0.2, 0.7, 0.8)
            }
        } else {
            let result = self.scene_intersect(orig, dir, spheres);
            match result {
                None => {
                    if let Some(env_map) = &self.env_map {
                        self.sample_spherical_map(dir, env_map)
                    } else {
                        ColorRGB::new(0.2, 0.7, 0.8)
                    }
                }
                Some(hit_result) => {
                    let point = hit_result.position;
                    let n = hit_result.normal;
                    let material = hit_result.material;
                    let reflect_dir = self.reflect(dir, &n).normalize();
                    let refract_dir = self.refract(dir, &n, material.refractive_index).normalize();
                    let reflect_orig = if reflect_dir.dot(n) < 0.0 {
                        point - n * 1e-3
                    } else {
                        point + n * 1e-3
                    };
                    let refract_orig = if refract_dir.dot(n) < 0.0 {
                        point - n * 1e-3
                    } else {
                        point + n * 1e-3
                    };
                    let reflect_color =
                        self.cast_ray(&reflect_orig, &reflect_dir, spheres, lights, depth + 1);
                    let refract_color =
                        self.cast_ray(&refract_orig, &refract_dir, spheres, lights, depth + 1);
                    let mut diffuse_light_intensity = 0.0;
                    let mut specular_light_intensity = 0.0;
                    for light in lights {
                        let shade_point_to_light = light.position - point;
                        let light_distance = shade_point_to_light.length();
                        let light_dir = shade_point_to_light / light_distance;

                        let shadow_orig = if light_dir.dot(n) < 0.0 {
                            point - n * 1e-3
                        } else {
                            point + n * 1e-3
                        };

                        if let Some(shadow_hit_result) =
                            self.scene_intersect(&shadow_orig, &light_dir, spheres)
                        {
                            if (shadow_hit_result.position - shadow_orig).length() < light_distance
                            {
                                continue;
                            }
                        }

                        diffuse_light_intensity += light.intensity * light_dir.dot(n);
                        specular_light_intensity += self
                            .reflect(&light_dir, &n)
                            .dot(*dir)
                            .max(0.0)
                            .powf(material.specular_exponent)
                            * light.intensity;
                    }
                    material.diffuse_color * diffuse_light_intensity * material.albedo[0]
                        + ColorRGB::new(1.0, 1.0, 1.0)
                            * specular_light_intensity
                            * material.albedo[1]
                        + reflect_color * material.albedo[2]
                        + refract_color * material.albedo[3]
                }
            }
        }
    }

    pub fn render(
        &mut self,
        items: &mut Vec<Box<dyn Primitive>>,
        lights: &Vec<Light>,
    ) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
        let mut frame_buffer = vec![ColorRGB::ZERO; (self.width * self.height) as usize];
        let aspect_ratio = self.width as f32 / self.height as f32;
        let fov = FRAC_PI_3;
        let half_fov = fov * 0.5;
        let tan_half_fov = half_fov.tan();
        let half_width_in_world_space = tan_half_fov * aspect_ratio;

        for j in 0..self.height {
            for i in 0..self.width {
                let color = &mut frame_buffer[(i + j * self.width) as usize];

                let x =
                    (2.0 * (i as f32 + 0.5) / self.width as f32 - 1.0) * half_width_in_world_space;
                let y = -(2.0 * (j as f32 + 0.5) / self.height as f32 - 1.0) * tan_half_fov;
                let dir = Vec3::new(x, y, -1.0).normalize();

                *color = self.cast_ray(&Vec3::ZERO, &dir, items, lights, 0);
            }
        }

        let mut imgbuf = image::ImageBuffer::new(self.width, self.height);
        // Iterate over the coordinates and pixels of the image
        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let index = (y * self.width + x) as usize;
            let color = &mut frame_buffer[index];
            let max_component = color.x.max(color.y).max(color.z);
            if max_component > 1.0 {
                *color /= max_component;
            }
            let r = (255.0 * color.x.clamp(0.0, 1.0)) as u8;
            let g = (255.0 * color.y.clamp(0.0, 1.0)) as u8;
            let b = (255.0 * color.z.clamp(0.0, 1.0)) as u8;

            *pixel = image::Rgb([r, g, b]);
        }

        imgbuf
    }
}
