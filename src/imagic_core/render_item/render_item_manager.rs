use crate::prelude::{bind_group::BindGroupManager, bind_group_layout::BindGroupLayoutManager, render_pipeline::RenderPipelineManager, GraphicsContext, MaterialManager, TransformManager};

use super::RenderItem;

pub struct  RenderItemManager {
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
    pub fn init_after_app(&mut self, graphics_context: &GraphicsContext, bind_group_manager: &mut BindGroupManager, bind_group_layout_manager: &BindGroupLayoutManager,
        material_manager: &MaterialManager, transform_manager: &TransformManager, render_pipeline_manager: &mut RenderPipelineManager) {
        for item in self.render_items.iter_mut() {
            if item.get_item_bind_group_id() == usize::MAX {
                let transform = transform_manager.get_transform(item.get_transform_id());
                let model_matrix = transform.trs_matrix();
                let mut mx_ref: [f32; 16] = [0.0; 16];
                mx_ref[..16].copy_from_slice(model_matrix.as_ref());

                let uniform_buf = graphics_context.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Uniform Buffer"),
                    contents: bytemuck::cast_slice(&mx_ref),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let model_vertex_bind_group_layout = bind_group_layout_manager.default_model_vertex_bind_group_layout();
                let bind_group = graphics_context.create_bind_group(&wgpu::BindGroupDescriptor{
                    layout: model_vertex_bind_group_layout,
                    label: Some("Vertex uniform buffer bind group"),
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: uniform_buf.as_entire_binding(),
                        },
                    ]
                });
                let item_bind_group_id = bind_group_manager.add_bind_group(bind_group);
                item.set_item_bind_group_id(item_bind_group_id);
            }

            if item.pipeline_id == usize::MAX {
                let material = material_manager.get_material(item.get_material_id());
                let pipeline_id = render_pipeline_manager.create_pipeline(graphics_context, bind_group_layout_manager, material);
                item.pipeline_id = pipeline_id;
            }
        }
    }

    pub fn add_render_item(&mut self, render_item: RenderItem) -> usize {
        let index = self.render_items.len();
        self.render_items.push(render_item);
        index
    }

    pub fn render_items(&self) -> &Vec<RenderItem> {
        &self.render_items
    }

    pub fn get_render_item(&self, index: usize) -> &RenderItem {
        &self.render_items[index]
    }

    pub fn get_render_item_mut(&mut self, index: usize) -> &mut RenderItem {
        &mut self.render_items[index]
    }
}