use std::usize;

use wgpu::util::DeviceExt;

use crate::{
    asset::asset::Handle, camera::Layer, prelude::{
        buffer::Buffer, render_item_manager::RenderItemManager, ImagicContext, Material, RenderItem, VertexOrIndexCount, INVALID_ID
    }, scene::{SceneObject, Transform}, types::ID
};

use super::Vertex;

/// Plane mesh struct.
pub struct Plane {
    pub width: f32,
    pub height: f32,
    pub width_segments: u32,
    pub height_segments: u32,

    pub transform: usize,

    render_item_id: usize,

    layer: Layer,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            width_segments: 1,
            height_segments: 1,

            transform: INVALID_ID,
            render_item_id: INVALID_ID,

            layer: Layer::Default,
        }
    }
}

impl SceneObject for Plane {
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

impl Plane {
    pub fn new(width: f32, height: f32, width_segments: u32, height_segments: u32) -> Self {
        Self {
            width,
            height,
            width_segments,
            height_segments,
            ..Default::default()
        }
    }

    pub fn render_item_id(&self) -> ID {
        self.render_item_id
    }

    pub fn init(&mut self, imagic_context: &mut ImagicContext, material_index: Handle<Material>) {
        let transform_manager = imagic_context.transform_manager();
        let transform = Transform::default();
        let transform_index = transform_manager.borrow_mut().add_transform(transform);
        self.transform = transform_index;

        let (vertex_buffer_id, index_buffer_id, index_count) = self.create_buffer(imagic_context);
        let mut plane_item = RenderItem::new(
            VertexOrIndexCount::IndexCount {
                index_count,
                base_vertex: 0,
                instance_count: 1,
                index_format: Plane::index_buffer_format(),
            },
            vertex_buffer_id,
            index_buffer_id,
            transform_index,
            true,
        );
        plane_item.set_material_id(material_index);
        self.render_item_id = imagic_context.add_render_item(plane_item);
    }

    fn create_vertices_data(&self) -> (Vec<Vertex>, Vec<u16>) {
        // (0.0, 0.0) left botton corner of the texture。
        // (1.0, 1.0) right top corner。
        let vertices: Vec<Vertex> = vec![
            Vertex::new([-1.0, -1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([1.0, -1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-1.0, 1.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        ];
        // ccw
        let indices: Vec<u16> = vec![0, 1, 2, 0, 2, 3];

        (vertices, indices)
    }

    fn create_buffer(&mut self, imagic_context: &mut ImagicContext) -> (Handle<Buffer>, Handle<Buffer>, u32) {
        let (vertex_data, index_data) = self.create_vertices_data();
        let device = imagic_context.graphics_context().get_device();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let vertex_buffer = Buffer::new(vertex_buffer);

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_buffer = Buffer::new(index_buffer);

        let vertex_buffer_id = imagic_context.asset_manager_mut().add(vertex_buffer);

        let index_buffer_id = imagic_context.asset_manager_mut().add(index_buffer);
        let index_count = index_data.len().try_into().unwrap();
        (vertex_buffer_id, index_buffer_id, index_count)
    }

    fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
