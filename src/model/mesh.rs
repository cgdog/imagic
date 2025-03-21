use wgpu::{util::DeviceExt, Buffer, Device};
// use std::hash::{Hash, Hasher};

use crate::{asset::{asset::{Asset, Handle}, asset_manager::AssetManager}, prelude::VertexOrIndexCount};

use super::Vertex;

#[derive(Clone)]
pub struct Mesh {
    // id: Handle::<Mesh>,
    vertex_buffer: Option<Handle<Buffer>>,
    index_buffer: Option<Handle<Buffer>>,
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

    pub fn upload(&mut self, device: &Device, asset_manager: &mut AssetManager) {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let vertex_buffer_handle = asset_manager.add(vertex_buffer);
        self.vertex_buffer = Some(vertex_buffer_handle);

        if let Some(indices) = &self.indices {
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Sphere Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            let index_buffer_handle = asset_manager.add(index_buffer);
            self.index_buffer = Some(index_buffer_handle);
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_indices(&self) -> &Option<Vec<u32>> {
        &self.indices
    }

    pub fn get_vertex_buffer(&self) -> &Option<Handle<Buffer>> {
        &self.vertex_buffer
    }

    pub fn get_index_buffer(&self) -> &Option<Handle<Buffer>> {
        &self.index_buffer
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