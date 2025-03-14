use wgpu::{util::DeviceExt, Buffer, BufferSlice, Device};
// use std::hash::{Hash, Hasher};

use crate::{asset::asset::Asset, prelude::VertexOrIndexCount};

use super::Vertex;

#[derive(Clone)]
pub struct Mesh {
    // id: Handle::<Mesh>,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertices: Vec<Vertex>,
    indices: Option<Vec<u32>>,
    vertex_or_index_count: VertexOrIndexCount,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Option<Vec<u32>>,
        vertex_or_index_count: VertexOrIndexCount,
    ) -> Self {
        Self {
            // id: Handle::<Mesh>::generate(),
            vertex_buffer: None,
            index_buffer: None,
            vertices,
            indices,
            vertex_or_index_count,
        }
    }

    pub fn upload(&mut self, device: &Device) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.vertex_buffer = Some(vertex_buffer);

        if let Some(indices) = &self.indices {
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            self.index_buffer = Some(index_buffer);
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_indices(&self) -> &Option<Vec<u32>> {
        &self.indices
    }

    pub fn get_vertex_buffer_slice(&self) -> Option<BufferSlice> {
        match &self.vertex_buffer {
            Some(vertex_buffer) => {
                Some(vertex_buffer.slice(..))
            }
            None => None
        }
    }

    pub fn get_index_buffer_slice(&self) -> Option<BufferSlice> {
        match &self.index_buffer {
            Some(index_buffer) => {
                Some(index_buffer.slice(..))
            }
            None => None
        }
    }

    pub fn get_vertex_or_index_count(&self) -> &VertexOrIndexCount {
        &self.vertex_or_index_count
    }
}

// impl Hash for Mesh {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         // self.id.hash(state);
//         // self.vertices.hash(state);
//         // if let Some(indices) = &self.indices {
//         //     indices.hash(state);
//         // }
//     }
// }

impl Asset for Mesh {
    // type ID = Handle::<Mesh>;

    // fn get_id(&self) -> Self::ID {
    //     self.id.clone()
    // }
}