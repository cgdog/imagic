use core::f32;
use std::{fs::File, io::{BufRead, BufReader}};

use crate::{math::{U16Vec3, Vec3}, prelude::raytracer::material::Material};

use super::{HitResult, Primitive};

pub struct Model {
    verts: Vec<Vec3>,
    faces: Vec<U16Vec3>,
    bbox: (Vec3, Vec3),
    material: Material,

    #[allow(unused)]
    translation: Vec3,
}

impl Model {
    pub fn new(file_path: &str, material: Material, translation: Vec3) -> Self {
        let (mut verts, faces) = Self::load_obj(file_path);
        for vert in verts.iter_mut() {
            *vert = *vert + translation;
        }
        let bbox = Self::compute_bbox(&verts);
        Self {
            verts,
            faces,
            bbox,
            material,
            translation,
        }
    }

    fn load_obj(file_path: &str) -> (Vec<Vec3>, Vec<U16Vec3>) {
        let input = BufReader::new(File::open(file_path).expect("Failed to load model file"));
        let mut verts: Vec<Vec3> = Vec::new();
        let mut faces: Vec<U16Vec3> = Vec::new();
        for line in input.lines() {
            if let Ok(line ) = line {
                if line.starts_with("v ") {
                    let numbers: Vec<&str> = line.split(" ").collect();
                    let mut pos = Vec3::new(0.0, 0.0, 0.0);
                    for index in 1..=3 {
                        let number: f32 = numbers[index].parse().expect("failed to parse .obj vertex");
                        pos[index - 1] = number;
                    }
                    verts.push(pos);
                } else if line.starts_with("f ") {
                    let numbers: Vec<&str> = line.split(" ").collect();
                    let mut face = U16Vec3::new(0, 0, 0);
                    for index in 1..=3 {
                        let number: u16 = numbers[index].parse().expect("failed to parse .obj index");
                        face[index - 1] = number - 1;
                    }
                    faces.push(face);
                }
            }
        }
        (verts, faces)
    }

    pub fn nverts(&self) -> usize {
        self.verts.len()
    }

    pub fn nfaces(&self) -> usize {
        self.faces.len()
    }

    pub fn get_bbox(&self) -> &(Vec3, Vec3) {
        &self.bbox
    }

    fn compute_bbox(verts: &Vec<Vec3>) -> (Vec3, Vec3) {
        let mut min = verts[0];
        let mut max = verts[0];
        for v in verts {
            for i in 0..3 {
                min[i] = min[i].min(v[i]);
                max[i] = max[i].max(v[i]);
            }
        }

        (min, max)
    }

    fn point(&self, index: usize) -> &Vec3 {
        assert!(index < self.nverts());
        &self.verts[index]
    }

    fn vert(&self, fi: usize, li: usize) -> usize {
        assert!(fi < self.nfaces() && li < 3);
        self.faces[fi][li] as usize
    }

    // Moller and Trumbore
    fn ray_triangle_intersect(&self, fi: usize, orig: &Vec3, dir: &Vec3, tnear: &mut f32, normal: &mut Vec3) -> bool {
        let edge1 = self.point(self.vert(fi,1)) - self.point(self.vert(fi,0));
        let edge2 = self.point(self.vert(fi,2)) - self.point(self.vert(fi,0));
        let pvec = dir.cross(edge2);
        let det = edge1.dot(pvec);
        if det<1e-5 {
            return false;
        }

        let tvec = orig - self.point(self.vert(fi,0));
        let u = tvec.dot(pvec);
        if u < 0.0 || u > det {
            return false;
        }

        let qvec = tvec.cross(edge1);
        let v = dir.dot(qvec);
        if v < 0.0 || u + v > det {
            return false;
        }

        *normal = edge1.cross(edge2).normalize();
        *tnear = edge2.dot(qvec) * (1./det);
        return *tnear > 1e-5;
    }
}

impl Primitive for Model {
    fn ray_intersect(&mut self, orig: &Vec3, dir: &Vec3, cur_nearest_t: &mut f32) -> Option<HitResult> {
        let mut tnear = f32::INFINITY;
        let mut normal = Vec3::new(0.0, 1.0, 0.0);
        for fi in 0..self.faces.len() {
            let mut cur_normal = Vec3::new(0.0, 1.0, 0.0);
            if self.ray_triangle_intersect(fi, orig, dir, &mut tnear, &mut cur_normal) {
                if tnear < *cur_nearest_t {
                    *cur_nearest_t = tnear;
                    normal = cur_normal;
                }
            }
        }

        if tnear < f32::INFINITY {
            let position = orig + dir * tnear;
            let hit_result = HitResult::new(position, normal, self.material);
            return Some(hit_result);
        }
        None
    }
}