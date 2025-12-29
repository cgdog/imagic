use crate::{
    assets::{
        asset::IAsset,
        meshes::{sub_mesh::SubMesh, vertex_attribute::VertexAttributes}, vertex_index::IndexData,
    },
    graphics::{buffer_view::BufferView, graphics_context::GraphicsContext}, types::Handle,
};

/// The tag of the mesh used to define MeshHandle.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum MeshTag {}

/// The handle of the mesh.
pub type MeshHandle = Handle<MeshTag>;

/// Mesh is a collection of submeshes.
pub struct Mesh {
    /// Vertex attributes, e.g., Position, Normal, UV, etc.
    pub vertex_attributes: VertexAttributes,
    // Gpu buffer of vertex attributes.
    pub(crate) vertex_buffer: BufferView,
    /// Index data.
    pub index_data: IndexData,
    /// Gpu buffer of index data.
    pub(crate) index_buffer: BufferView,
    /// Submeshes.
    pub sub_meshes: Vec<SubMesh>,

    /// When is_dirty is true, mesh data shoud be upload to GPU.
    pub(crate) is_dirty: bool,
}

impl IAsset for Mesh {}

impl Mesh {
    /// Create a new mesh.
    /// 
    /// # Arguments
    /// * `vertex_attributes` - Vertex attributes, e.g., Position, Normal, UV, etc.
    /// * `index_data` - Index data.
    /// * `sub_meshes` - Submeshes.
    /// 
    /// # Returns
    /// * `Mesh` - The new mesh.
    pub fn new(mut vertex_attributes: VertexAttributes, index_data: IndexData, sub_meshes: Vec<SubMesh>) -> Mesh {
        vertex_attributes.compute_vertex_attributes();
        Self {
            vertex_buffer: BufferView::INVALID,
            vertex_attributes,
            index_data,
            index_buffer: BufferView::INVALID,
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

        self.upload_index_buffer(graphics_context);

        self.is_dirty = false;
    }

    fn upload_index_buffer(&mut self, graphics_context: &mut GraphicsContext) {
        let content = self.index_data.content();
        let content_size = (content.len() * size_of::<u8>()) as u64;
        self.index_buffer = graphics_context.buffer_manager.allocate_index_buffer(content_size);
        graphics_context.buffer_manager.write_data(&self.index_buffer, content);
    }
}
