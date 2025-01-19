use std::usize;

use wgpu::util::DeviceExt;

use crate::{
    camera::Layer,
    prelude::{render_item_manager::RenderItemManager, ImagicContext, RenderItem, VertexOrIndexCount, INVALID_ID},
    scene::{SceneObject, Transform},
    types::ID,
};

use super::Vertex;

fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32],
        normal: [pos[0] as f32, pos[1] as f32, pos[2] as f32], // TODO: change to real normal.
        uv: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_cube_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

/// Cube mesh struct.
pub struct Cube {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub transform: usize,

    pub width_segments: u32,
    pub height_segments: u32,
    pub depth_segments: u32,

    render_item_id: usize,

    layer: Layer,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            width_segments: 1,
            height_segments: 1,
            depth_segments: 1,
            transform: INVALID_ID,
            render_item_id: INVALID_ID,
            layer: Layer::Default,
        }
    }
}

impl SceneObject for Cube {
    fn transform(&self) -> &usize {
        &self.transform
    }

    fn get_layer(&self) -> Layer {
        self.layer
    }

    fn set_layer(&mut self, layer: Layer, render_item_manager: &mut RenderItemManager) {
        self.layer = layer;
        render_item_manager
            .get_render_item_mut(self.render_item_id)
            .layer = layer;
    }
}

impl Cube {
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
            ..Default::default()
        }
    }

    pub fn render_item_id(&self) -> ID {
        self.render_item_id
    }

    pub fn init(&mut self, imagic_context: &mut ImagicContext, material_index: usize) {
        let transform_manager = imagic_context.transform_manager();
        let transform = Transform::default();
        let transform_index = transform_manager.borrow_mut().add_transform(transform);
        self.transform = transform_index;

        let (vertex_buffer_id, index_buffer_id, index_count) = self.create_buffer(imagic_context);
        let mut cube_item = RenderItem::new(
            VertexOrIndexCount::IndexCount {
                index_count,
                base_vertex: 0,
                instance_count: 1,
                index_format: Cube::index_buffer_format(),
            },
            vertex_buffer_id,
            index_buffer_id,
            transform_index,
            true,
        );
        cube_item.set_material_id(material_index);
        self.render_item_id = imagic_context
            .render_item_manager_mut()
            .add_render_item(cube_item);
    }

    pub fn create_buffer(&mut self, imagic_context: &mut ImagicContext) -> (usize, usize, u32) {
        let (vertex_data, index_data) = create_cube_vertices();
        let device = imagic_context.graphics_context().get_device();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let buffer_manager = imagic_context.buffer_manager_mut();
        let vertex_buffer_id = buffer_manager.add_buffer(vertex_buffer);

        let index_buffer_id = buffer_manager.add_buffer(index_buffer);
        let index_count = index_data.len().try_into().unwrap();
        (vertex_buffer_id, index_buffer_id, index_count)
    }

    pub fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
