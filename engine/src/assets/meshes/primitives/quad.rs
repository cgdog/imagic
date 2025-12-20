use crate::{assets::meshes::{mesh::Mesh, sub_mesh::SubMesh, vertex_attribute::VertexAttributes, vertex_index::IndexData}, math::{Vec2, Vec3}};

/// Quad geometry. The normal directional is +z.
pub struct Quad {
    pub width: f32,
    pub height: f32,
    pub width_segments: u32,
    pub height_segments: u32,
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            width_segments: 1,
            height_segments: 1,
        }
    }
}

impl Quad {
    pub fn new(width: f32, height: f32, width_segments: u32, height_segments: u32) -> Self {
        let quad = Self {
            width,
            height,
            width_segments,
            height_segments,
        };
        quad
    }
}

/// Convert a Quad to Mesh.
impl From<Quad> for Mesh {
    fn from(quad: Quad) -> Self {
        let x_num = (quad.width_segments + 1) as usize;
        let y_num = (quad.height_segments + 1) as usize;
        let vertex_num = x_num * y_num;
        let last_x = quad.width_segments as f32;
        let last_y = quad.height_segments as f32;

        let mut vertex_attributes = VertexAttributes::default();

        // let mut position: Vec<Vec3> = Vec::new();
        vertex_attributes.position.reserve_exact(vertex_num);
        vertex_attributes.normal = vec![Vec3::new(0.0, 0.0, 1.0); vertex_num];
        vertex_attributes.uv.reserve_exact(vertex_num);

        let mut pos_x = Vec::new();
        pos_x.reserve_exact(x_num);
        for x_index in 0..x_num {
            let x_ratio = x_index as f32 / last_x - 0.5; // [-0.5, 0.5]
            let x_pos = x_ratio * quad.width;
            pos_x.push(x_pos);
        }

        let mut pos_y = Vec::new();
        pos_y.reserve_exact(y_num);
        for y_index in 0..y_num {
            let y_ratio = y_index as f32 / last_y - 0.5; // [-0.5, 0.5]
            let y_pos = y_ratio * quad.height;
            pos_y.push(y_pos);
        }

        let mut indices = Vec::<u16>::new();
        for y in 0..y_num {
            let not_last_y = y < (y_num - 1);
            for x in 0..x_num {
                let not_last_x = x < (x_num - 1);
                let pos = Vec3::new(pos_x[x], pos_y[y], 0.0);
                vertex_attributes.position.push(pos);
                // 在 WGPU 中，NDC Y 轴是向上的，范围是 [-1, 1]。
                // wgpu 对「正面」的判定是 基于「屏幕空间」的缠绕顺序，而非你定义顶点时的「模型空间」
                // https://www.w3.org/TR/webgpu/#coordinate-systems
                vertex_attributes.uv.push(Vec2::new(x as f32 / last_x, y as f32 / last_y));
                
                if not_last_x && not_last_y {
                    // 正常情况下，glTF 2.0 的模型空间（局部空间）y 轴是「向上」的
                    // v2 - v3
                    // |  / |
                    // v0 - v1
                    let v0 = (x + y * x_num) as u16;
                    let v1 = v0 + 1;
                    let v2 = (x + (y + 1) * x_num) as u16;
                    let v3 = v2 + 1;
                    // CCW triangle 1: (v0, v2, v1)
                    indices.push(v0);
                    indices.push(v3);
                    indices.push(v2);
                    // CCW triangle 2: (v1, v2, v3)
                    indices.push(v0);
                    indices.push(v1);
                    indices.push(v3);
                }
            }
        }

        let sub_mesh = SubMesh::new(IndexData::new_u16(indices));

        Mesh::new(vertex_attributes, vec![sub_mesh])
    }
}