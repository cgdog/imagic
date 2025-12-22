use crate::{
    assets::meshes::{
        mesh::Mesh, sub_mesh::SubMesh, vertex_attribute::VertexAttributes, vertex_index::IndexData,
    },
    math::{Vec2, Vec3},
};

/// Cuboid geometry.
pub struct Cuboid {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub width_segments: u32,
    pub height_segments: u32,
    pub depth_segments: u32,
}

impl Default for Cuboid {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            width_segments: 1,
            height_segments: 1,
            depth_segments: 1,
        }
    }
}

enum CuboidFace {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl Cuboid {
    pub fn new(
        width: f32,
        height: f32,
        depth: f32,
        width_segments: u32,
        height_segments: u32,
        depth_segments: u32,
    ) -> Self {
        Self {
            width,
            height,
            depth,
            width_segments,
            height_segments,
            depth_segments,
        }
    }

    fn create_face(
        width: f32,
        height: f32,
        x_num: usize,
        y_num: usize,
        last_x: f32,
        last_y: f32,
        z: f32,
        vertex_attributes: &mut VertexAttributes,
        indices: &mut Vec<u32>,
        normal: Vec3,
        cur_base_index: &mut u32,
        face: CuboidFace,
    ) {
        let is_reverse_x = matches!(face, CuboidFace::Back | CuboidFace::Right);
        let is_reverse_y = matches!(face, CuboidFace::Top);
        let mut pos_x = Vec::new();
        pos_x.reserve_exact(x_num);
        for x_index in 0..x_num {
            let x_ratio = if is_reverse_x {
                1.0 - x_index as f32 / last_x - 0.5 // [0.5, -0.5]
            } else {
                x_index as f32 / last_x - 0.5 // [-0.5, 0.5]
            };
            let x_pos = x_ratio * width;
            pos_x.push(x_pos);
        }
        let mut pos_y = Vec::new();
        pos_y.reserve_exact(y_num);
        for y_index in 0..y_num {
            let y_ratio = if is_reverse_y {
                1.0 - y_index as f32 / last_y - 0.5 // [0.5, -0.5]
            } else {
                y_index as f32 / last_y - 0.5 // [-0.5, 0.5]
            };
            let y_pos = y_ratio * height;
            pos_y.push(y_pos);
        }

        let (remapped_x_index, remapped_y_index, remapped_z_index) = match face {
            CuboidFace::Front | CuboidFace::Back => {
                (0, 1, 2)
            }
            CuboidFace::Left | CuboidFace::Right => {
                (2, 1, 0)
            },
            CuboidFace::Top | CuboidFace::Bottom => {
                (0, 2, 1)
            },
        };
        for y in 0..y_num {
            let not_last_y: bool = y < y_num - 1;
            for x in 0..x_num {
                let not_last_x = x < x_num - 1;
                // let pos = Vec3::new(pos_x[x], pos_y[y], 0.0);
                let mut pos = Vec3::ZERO;
                pos[remapped_x_index] = pos_x[x];
                pos[remapped_y_index] = pos_y[y];
                pos[remapped_z_index] = z;
                vertex_attributes.position.push(pos);
                // In wgpu, y of texture coordinates is in [0, 1] from top to bottom.
                vertex_attributes
                    .uv
                    .push(Vec2::new(x as f32 / last_x, 1.0 - y as f32 / last_y));
                vertex_attributes.normal.push(normal);

                if not_last_x && not_last_y {
                    // v2 - v3
                    // |  / |
                    // v0 - v1
                    let v0 = (x + y * x_num) as u32;
                    let v1 = v0 + 1;
                    let v2 = (x + (y + 1) * x_num) as u32;
                    let v3 = v2 + 1;
                    // CCW triangle 1: (v0, v2, v1)
                    indices.push(*cur_base_index + v0);
                    indices.push(*cur_base_index + v3);
                    indices.push(*cur_base_index + v2);
                    // CCW triangle 2: (v1, v2, v3)
                    indices.push(*cur_base_index + v0);
                    indices.push(*cur_base_index + v1);
                    indices.push(*cur_base_index + v3);
                }
            }
        }
        *cur_base_index += (x_num * y_num) as u32;
    }
}

impl From<Cuboid> for Mesh {
    fn from(cuboid: Cuboid) -> Self {
        let x_num = (cuboid.width_segments + 1) as usize;
        let y_num = (cuboid.height_segments + 1) as usize;
        let z_num = (cuboid.depth_segments + 1) as usize;
        let vertex_num = (x_num * y_num + y_num * z_num + z_num * x_num) * 2;
        let last_x = cuboid.width_segments as f32;
        let last_y = cuboid.height_segments as f32;
        let last_z = cuboid.depth_segments as f32;

        let mut vertex_attributes = VertexAttributes::default();
        vertex_attributes.position.reserve_exact(vertex_num);
        vertex_attributes.normal.reserve_exact(vertex_num);
        vertex_attributes.uv.reserve_exact(vertex_num);

        let mut indices = Vec::<u32>::new();
        let mut cur_base_index = 0;
        // front face
        Cuboid::create_face(
            cuboid.width,
            cuboid.height,
            x_num,
            y_num,
            last_x,
            last_y,
            cuboid.depth * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(0.0, 0.0, 1.0),
            &mut cur_base_index,
            CuboidFace::Front,
        );

        // back face
        Cuboid::create_face(
            cuboid.width,
            cuboid.height,
            x_num,
            y_num,
            last_x,
            last_y,
            -cuboid.depth * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(0.0, 0.0, -1.0),
            &mut cur_base_index,
            CuboidFace::Back,
        );

        // left face
        Cuboid::create_face(
            cuboid.depth,
            cuboid.height,
            z_num,
            y_num,
            last_z,
            last_y,
            -cuboid.width * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(-1.0, 0.0, 0.0),
            &mut cur_base_index,
            CuboidFace::Left,
        );

        // right face
        Cuboid::create_face(
            cuboid.depth,
            cuboid.height,
            z_num,
            y_num,
            last_z,
            last_y,
            cuboid.width * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(1.0, 0.0, 0.0),
            &mut cur_base_index,
            CuboidFace::Right,
        );

        // top face
        Cuboid::create_face(
            cuboid.width,
            cuboid.depth,
            x_num,
            z_num,
            last_x,
            last_z,
            cuboid.height * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(0.0, 1.0, 0.0),
            &mut cur_base_index,
            CuboidFace::Top,
        );

        // bottom face
        Cuboid::create_face(
            cuboid.width,
            cuboid.depth,
            x_num,
            z_num,
            last_x,
            last_z,
            -cuboid.height * 0.5,
            &mut vertex_attributes,
            &mut indices,
            Vec3::new(0.0, -1.0, 0.0),
            &mut cur_base_index,
            CuboidFace::Bottom,
        );

        let index_data = IndexData::new_u32(indices);

        let sub_mesh = SubMesh::new(0, index_data.index_count(), 0);

        Mesh::new(vertex_attributes, index_data, vec![sub_mesh])
    }
}
