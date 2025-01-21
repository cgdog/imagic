use crate::{
    prelude::{
        bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, GraphicsContext,
        TransformManager, INVALID_ID,
    },
    types::ID,
};

use super::RenderItem;

pub struct RenderItemManager {
    render_items: Vec<RenderItem>,
}

impl Default for RenderItemManager {
    fn default() -> Self {
        Self {
            render_items: Vec::new(),
        }
    }
}

impl RenderItemManager {
    /// Initialize a render item, e.g., create model related uniform buffer,
    /// create bind group layout, bind group, and pipeline.
    fn init_item(
        &mut self,
        item: &mut RenderItem,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &BindGroupLayoutManager,
        transform_manager: &TransformManager,
    ) {
        if item.get_item_bind_group_id() == INVALID_ID {
            let transform = transform_manager.get_transform(item.get_transform_id());
            let model_matrix = transform.trs_matrix();
            let mut mx_ref: [f32; 16] = [0.0; 16];
            mx_ref[..16].copy_from_slice(model_matrix.as_ref());

            let uniform_buf =
                graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    contents: bytemuck::cast_slice(&mx_ref),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

            let model_vertex_bind_group_layout =
                bind_group_layout_manager.default_model_vertex_bind_group_layout();
            let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: model_vertex_bind_group_layout,
                label: Some("Vertex uniform buffer bind group"),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                }],
            });
            let item_bind_group_id = bind_group_manager.add_bind_group(bind_group);
            item.set_item_bind_group_id(item_bind_group_id);
        }
    }

    pub(crate) fn add_render_item(
        &mut self,
        mut render_item: RenderItem,
        graphics_context: &GraphicsContext,
        bind_group_manager: &mut BindGroupManager,
        bind_group_layout_manager: &BindGroupLayoutManager,
        transform_manager: &TransformManager,
    ) -> ID {
        self.init_item(
            &mut render_item,
            graphics_context,
            bind_group_manager,
            bind_group_layout_manager,
            transform_manager,
        );
        let index = self.render_items.len();
        self.render_items.push(render_item);
        index
    }

    pub fn render_items(&self) -> &Vec<RenderItem> {
        &self.render_items
    }

    pub fn render_items_mut(&mut self) -> &mut Vec<RenderItem> {
        &mut self.render_items
    }

    pub fn get_render_item(&self, index: usize) -> &RenderItem {
        &self.render_items[index]
    }

    pub fn get_render_item_mut(&mut self, index: usize) -> &mut RenderItem {
        &mut self.render_items[index]
    }
}
