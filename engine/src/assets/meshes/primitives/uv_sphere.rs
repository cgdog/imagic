use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::{assets::meshes::{mesh::Mesh, sub_mesh::SubMesh, vertex_attribute::VertexAttributes, vertex_index::IndexData}, math::{Vec2, Vec3}};

// https://www.songho.ca/opengl/gl_sphere.html
/// UV Sphere geometry generator.
pub struct UVSphere {
    pub radius: f32,
    pub sector_count: u32,
    pub stack_count: u32,
}

impl Default for UVSphere {
    fn default() -> Self {
        Self {
            radius: 0.5,
            sector_count: 36,
            stack_count: 18,
        }
    }
}

impl UVSphere {
    pub fn new(radius: f32, sector_count: u32, stack_count: u32) -> Self {
        Self {
            radius,
            sector_count,
            stack_count,
        }
    }
}

impl From<UVSphere> for Mesh {
    fn from(sphere: UVSphere) -> Self {
        let angle_per_sector = TAU / sphere.sector_count as f32;
        let angle_per_stack = PI / sphere.stack_count as f32;
        let radius_inverse = 1.0 / sphere.radius;
        let mut vertex_attributes = VertexAttributes::default();
        for i in 0..=sphere.stack_count {
            let stack_angle = FRAC_PI_2 - i as f32 * angle_per_stack;
            let y = sphere.radius * stack_angle.sin();
            let xz = sphere.radius * stack_angle.cos();
            let t = i as f32 / sphere.stack_count as f32;

            // There are sector_count + 1 vertices per stack.
            // The first and last vertices have same position, normal but different uvs.
            for j in 0..=sphere.sector_count {
                let sector_angle = j as f32 * angle_per_sector;
                let x = xz * sector_angle.sin();
                let z = xz * sector_angle.cos();
                let position = Vec3::new(x, y, z);
                vertex_attributes.position.push(position);

                let normal = position * radius_inverse;
                vertex_attributes.normal.push(normal);

                let s = j as f32 / sphere.sector_count as f32;
                let uv = Vec2::new(s, t);
                vertex_attributes.uv.push(uv);
            }
        }

        let mut indices = Vec::<u32>::new();

        // CCW
        // v1--v1+1
        // |  / |
        // | /  |
        // v2--v2+1
        for i in 0..sphere.stack_count {
            let mut v1 = i * (sphere.sector_count + 1);
            let mut v2 = v1 + sphere.sector_count + 1;

            for _j in 0..sphere.sector_count {
                if i != 0 {
                    indices.push(v1);
                    indices.push(v2);
                    indices.push(v1 + 1);
                }

                if i != sphere.stack_count - 1 {
                    indices.push(v1 + 1);
                    indices.push(v2);
                    indices.push(v2 + 1);
                }
                v1 += 1;
                v2 += 1;
            }
        }

        let sub_mesh = SubMesh::new(IndexData::new_u32(indices));

        Mesh::new(vertex_attributes, vec![sub_mesh])
    }
}