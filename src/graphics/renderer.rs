use wgpu::TextureView;
use winit::window::Window as WindowWinit;

use crate::{camera::Camera, imagic_core::imagic_context::ImagicContext, prelude::VertexOrIndexCount, ui::ui_renderer::UIRenderer};

pub struct Renderer {
    ui_renderer: Option<UIRenderer>,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
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

    pub fn render(&mut self, context: & ImagicContext, _window: &WindowWinit) {
        let surface_texture = context.graphics_context().get_surface().get_current_texture();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let cameras = context.camera_manager().get_cameras();
        for (index, camera) in cameras.iter().enumerate() {
            self.render_with_camera(context, camera, index, &surface_texture_view);
        }

        self.render_ui(context, _window, &surface_texture_view);
        surface_texture.present();
    }

    pub fn render_with_camera(&mut self, context: & ImagicContext, camera: &Camera, camera_index: usize, surface_texture_view: &TextureView) {
        let mut encoder = context.graphics_context()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("imagic render command encoder desc") });
        // Render scene
        {
            let camera_clear_color = camera.get_clear_color();
            let camera_depth_textue = camera.get_depth_texture();
            let dpeth_texture_view = context.texture_manager().get_texture_view(camera_depth_textue);

            let mut load_op = wgpu::LoadOp::Load;
            if camera_index == 0 {
                let clear_color = wgpu::Color {
                    r: camera_clear_color.x as f64,
                    g: camera_clear_color.y as f64,
                    b: camera_clear_color.z as f64,
                    a: camera_clear_color.w as f64
                };
                load_op = wgpu::LoadOp::Clear(clear_color);
            }

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("imagic render pass desc"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // load: wgpu::LoadOp::Clear(clear_color),
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                // depth_stencil_attachment: None,
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: dpeth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let view_port = camera.get_physical_viewport();
            rpass.set_viewport(view_port.x, view_port.y, view_port.z, view_port.w, 0.0, 1.0);

            let camera_bind_group_id = camera.get_bind_group_id();

            let render_items = context.render_item_manager().render_items();
            for item in render_items.iter() {
                if item.is_visible {
                    let render_pipeline = context.pipeline_manager().get_render_pipeline(item.get_pipeline());

                    let material = context.material_manager().get_material(item.get_material_id());
                    let material_bind_group_id = material.get_bind_group_id();
                    let lighting_bind_group_id = context.light_manager().get_bind_group_id();
                    let item_bind_groups = [item.get_item_bind_group_id(), camera_bind_group_id,
                        material_bind_group_id, lighting_bind_group_id];

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
        
        context.graphics_context().submit(Some(encoder.finish()));
    }

    pub fn render_ui(&mut self, context: & ImagicContext, window: &WindowWinit, surface_texture_view: &TextureView) {

        let mut encoder = context.graphics_context()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("imagic render ui command encoder desc") });
        
        self.ui_renderer().draw(
            context.graphics_context(),
            &mut encoder,
            &window,
            surface_texture_view,
        );

        context.graphics_context().submit(Some(encoder.finish()));
    }
}