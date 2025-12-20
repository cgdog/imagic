use crate::{
    assets::meshes::vertex_index::IndexData,
    graphics::{buffer_view::BufferView, graphics_context::GraphicsContext},
};

pub struct SubMesh {
    pub index_data: IndexData,
    pub index_buffer: BufferView,
}

impl SubMesh {
    pub fn new(index_data: IndexData) -> Self {
        Self {
            index_data,
            index_buffer: BufferView::INVALID,
        }
    }

    pub(crate) fn upload(&mut self, graphics_context: &mut GraphicsContext) {
        let content = self.index_data.content();
        let content_size = (content.len() * size_of::<u8>()) as u64;
        self.index_buffer = graphics_context.buffer_manager.allocate_index_buffer(content_size);
        graphics_context.buffer_manager.write_data(&self.index_buffer, content);
    }
}
