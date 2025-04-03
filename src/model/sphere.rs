use std::f32::consts::{PI, TAU};
use wgpu::util::DeviceExt;

use crate::{
    asset::asset::Handle, camera::Layer, prelude::{
        render_item_manager::RenderItemManager, ImagicContext, Material, RenderItem, VertexOrIndexCount, INVALID_ID
    }, scene::{scene_object::SceneObject, transform::Transform}, types::ID
};

use super::vertex_attribute::Vertex;

fn create_sphere_vertices(
    radius: f32,
    x_segments: u32,
    y_segments: u32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for x in 0..=x_segments {
        let x_segment = x as f32 / x_segments as f32;
        let x_angle = x_segment * TAU;
        for y in 0..=y_segments {
            let y_segment = y as f32 / x_segments as f32;
            let y_angle = y_segment * PI;
            let y_pos = f32::cos(y_angle);
            let zx = f32::sin(y_angle);
            let x_pos = zx * f32::cos(x_angle);
            let z_pos = zx * f32::sin(x_angle);

            let vertex = Vertex {
                pos: [radius * x_pos, radius * y_pos, radius * z_pos],
                normal: [x_pos, y_pos, z_pos],
                uv: [x_segment, y_segment],
            };
            vertices.push(vertex);
        }
    }

    for y in 0..y_segments {
        for x in 0..x_segments {
            let i1 = y * (x_segments + 1) + x;
            let i2 = (y + 1) * (x_segments + 1) + x;
            let i3 = (y + 1) * (x_segments + 1) + x + 1;
            let i4 = y * (x_segments + 1) + x + 1;

            indices.push(i1);
            indices.push(i2);
            indices.push(i4);

            indices.push(i2);
            indices.push(i3);
            indices.push(i4);
        }
    }

    (vertices, indices)
}

pub struct Sphere {
    pub radius: f32,
    pub x_segments: u32,
    pub y_segments: u32,
    pub transform: usize,

    render_item_id: usize,

    layer: Layer,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            radius: 1.0,
            x_segments: 64,
            y_segments: 64,
            transform: INVALID_ID,
            render_item_id: INVALID_ID,
            layer: Layer::Default,
        }
    }
}

impl SceneObject for Sphere {
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

impl Sphere {
    pub fn new(radius: f32, x_segments: u32, y_segments: u32) -> Self {
        Self {
            radius,
            x_segments,
            y_segments,
            ..Default::default()
        }
    }

    pub fn render_item_id(&self) -> ID {
        self.render_item_id
    }

    pub fn init(&mut self, imagic_context: &mut ImagicContext, pbr_material_index: Handle<Material>) {
        let transform = Transform::default();
        self.init_with_transform(imagic_context, pbr_material_index, transform);
    }

    pub fn init_with_transform(&mut self, imagic_context: &mut ImagicContext, pbr_material_index: Handle<Material>, transform: Transform,) {
        let transform_manager = imagic_context.transform_manager();
        let transform_index = transform_manager.borrow_mut().add_transform(transform);
        self.transform = transform_index;

        let (vertex_buffer_id, index_buffer_id, index_count) = self.create_buffer(imagic_context);
        let mut sphere_item = RenderItem::new(
            VertexOrIndexCount::IndexCount {
                index_count,
                base_vertex: 0,
                instance_count: 1,
                index_format: Sphere::index_buffer_format(),
            },
            vertex_buffer_id,
            index_buffer_id,
            transform_index,
            true,
        );
        sphere_item.set_material_id(pbr_material_index);
        self.render_item_id = imagic_context.add_render_item(sphere_item);
    }

    fn create_buffer(&mut self, imagic_context: &mut ImagicContext) -> (usize, usize, u32) {
        let (vertex_data, index_data) =
            create_sphere_vertices(self.radius, self.x_segments, self.y_segments);
        let device = imagic_context.graphics_context().get_device();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let buffer_manager = imagic_context.buffer_manager_mut();
        let vertex_buffer_id = buffer_manager.add_buffer(vertex_buffer);

        let index_buffer_id = buffer_manager.add_buffer(index_buffer);
        let index_count = index_data.len().try_into().unwrap();
        (vertex_buffer_id, index_buffer_id, index_count)
    }

    fn index_buffer_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint32
    }
}
