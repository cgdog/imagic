use std::usize;

use wgpu::util::DeviceExt;

use crate::{
    camera::Layer,
    prelude::{render_item_manager::RenderItemManager, RenderItem, VertexOrIndexCount, INVALID_ID},
    scene::{SceneObject, Transform, TransformManager},
    Imagic,
};

use super::Vertex;

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

    pub fn render_item_id(&self) -> usize {
        self.render_item_id
    }

    pub fn init(&mut self, imagic: &mut Imagic, material_index: usize) {
        let transform_manager: &mut TransformManager = imagic.context_mut().transform_manager_mut();
        let transform = Transform::default();
        let transform_index = transform_manager.add_transform(transform);
        self.transform = transform_index;

        let (vertex_buffer_id, index_buffer_id, index_count) = self.create_buffer(imagic);
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
        self.render_item_id = imagic
            .context_mut()
            .render_item_manager_mut()
            .add_render_item(plane_item);
    }

    fn create_vertices_data(&self) -> (Vec<Vertex>, Vec<u16>) {
        let vertices: Vec<Vertex> = vec![
            Vertex::new([-1.0, 1.0, 0.0], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex::new([1.0, 1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new([1.0, -1.0, 0.0], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new([-1.0, -1.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0]),
        ];
        // ccw
        let indices: Vec<u16> = vec![0, 3, 1, 1, 3, 2];

        (vertices, indices)
    }

    fn create_buffer(&mut self, imagic: &mut Imagic) -> (usize, usize, u32) {
        let (vertex_data, index_data) = self.create_vertices_data();
        let device = imagic.context().graphics_context().get_device();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let buffer_manager = imagic.context_mut().buffer_manager_mut();
        let vertex_buffer_id = buffer_manager.add_buffer(vertex_buffer);

        let index_buffer_id = buffer_manager.add_buffer(index_buffer);
        let index_count = index_data.len().try_into().unwrap();
        (vertex_buffer_id, index_buffer_id, index_count)
    }

    fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }
}
