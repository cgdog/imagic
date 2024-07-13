use winit::window::Window as WindowWinit;

use crate::{imagic_core::imagic_context::ImagicContext, prelude::VertexOrIndexCount, ui::ui_renderer::UIRenderer};

pub struct Renderer {
    clear_color: wgpu::Color,
    ui_renderer: Option<UIRenderer>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            ui_renderer: None,
        }
    }
}

impl Renderer {
    pub fn set_ui_renderer(&mut self, ui_renderer: Option<UIRenderer>) {
        self.ui_renderer = ui_renderer;
    }

    pub fn ui_renderer(&mut self) -> &mut UIRenderer {
        self.ui_renderer.as_mut().expect("ui_renderer is None.")
    }

    pub fn render(&mut self, context: &mut ImagicContext, window: &WindowWinit) {
        let surface_texture = context.graphics_context().get_surface().get_current_texture();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = context.graphics_context()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("imagic encoder desc") });
        // Render scene
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("imagic render pass desc"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let render_items = context.render_item_manager().render_items();
            for item in render_items.iter() {
                if item.is_visible {
                    let render_pipeline = context.pipeline_manager().get_render_pipeline(item.get_pipeline());
                    let item_bind_groups = item.get_bind_group();
                    // let bind_group = context.bind_group_manager().get_bind_group();
                    rpass.set_pipeline(render_pipeline);
                    for (index, bind_group_index) in item_bind_groups.iter().enumerate() {
                        let bind_group = context.bind_group_manager().get_bind_group(*bind_group_index);
                        rpass.set_bind_group(index as u32, bind_group, &[]);
                    }

                    let vertex_or_index_count = item.get_vertex_or_index_count();
                    match vertex_or_index_count {
                        VertexOrIndexCount::VertexCount { vertex_count, instance_count } => {
                            rpass.draw(0..*vertex_count, 0..*instance_count);
                        }
                        VertexOrIndexCount::IndexCount { index_count, base_vertex, instance_count, index_format } => {
                            let vertex_buffer = context.buffer_manager().get_buffer(item.get_vertex_buffer_id());
                            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                            let index_buffer = context.buffer_manager().get_buffer(item.get_index_buffer_id());
                            rpass.set_index_buffer(index_buffer.slice(..), *index_format);
                            rpass.draw_indexed(0..*index_count, *base_vertex, 0..*instance_count);
                        }
                    }
                }
            }
        }
        // UI
        {
            self.ui_renderer().draw(
                context.graphics_context(),
                &mut encoder,
                &window,
                &surface_texture_view,
            );
        }
        context.graphics_context().submit(Some(encoder.finish()));
        surface_texture.present();
    }
}