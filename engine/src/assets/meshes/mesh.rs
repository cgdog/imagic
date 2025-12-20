use crate::{
    assets::{
        asset::IAsset,
        meshes::{sub_mesh::SubMesh, vertex_attribute::VertexAttributes},
    },
    graphics::{buffer_view::BufferView, graphics_context::GraphicsContext},
};

pub struct Mesh {
    /// Vertex attributes, e.g., Position, Normal, UV, etc.
    pub vertex_attributes: VertexAttributes,
    // Gpu buffer of vertex attributes.
    pub(crate) vertex_buffer: BufferView,
    /// Submeshes.
    pub sub_meshes: Vec<SubMesh>,

    /// When is_dirty is true, mesh data shoud be upload to GPU.
    pub(crate) is_dirty: bool,
}

impl IAsset for Mesh {}

impl Mesh {
    pub fn new(mut vertex_attributes: VertexAttributes, sub_meshes: Vec<SubMesh>) -> Mesh {
        vertex_attributes.compute_vertex_attributes();
        Self {
            vertex_buffer: BufferView::INVALID,
            vertex_attributes,
            sub_meshes,
            is_dirty: true,
        }
    }

    /// Upload mesh data to GPU when it is dirty.
    pub(crate) fn upload(&mut self, graphics_context: &mut GraphicsContext) {
        if !self.is_dirty {
            return;
        }
        let content = self.vertex_attributes.content();
        let content_size = (content.len() * size_of::<f32>()) as u64;

        self.vertex_buffer = graphics_context.buffer_manager.allocate_vertex_buffer(content_size);
        graphics_context.buffer_manager.write_data(&self.vertex_buffer, bytemuck::cast_slice(&content));

        for sub_mesh in &mut self.sub_meshes {
            sub_mesh.upload(graphics_context);
        }

        self.is_dirty = false;
    }
}
