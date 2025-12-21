use wgpu::{CommandEncoder, RenderPass};

use crate::{
    assets::{TextureHandle, TextureSamplerManager, texture_view::TextureView},
    graphics::graphics_context::GraphicsContext,
    math::{Vec4, color::Color},
    renderer::frame_data::{CameraRenderData, ItemRenderData}
};

/// Utilities and APIs to draw a Mesh, or a scene.
pub struct RenderAPI {}

impl RenderAPI {
    pub(crate) fn render(
        graphics_context: & GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
        surface_texture_view_ref: &wgpu::TextureView,
        camera_render_data: &CameraRenderData,
    ) {
        let mut encoder =
            graphics_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("lxy render command encoder desc"),
                });

        Self::render_pass(
            graphics_context,
            texture_sampler_manager,
            &mut encoder,
            &surface_texture_view_ref,
            camera_render_data,
        );
        graphics_context.queue.submit(Some(encoder.finish()));
    }

    /// Render a single render pass.
    /// 
    /// - Per-Material uniforms should be a unique bind group.
    /// - Per-Item(model matrix, normal matrix, etc) uniforms should be a second unique bind group.
    /// - Per-Camera(view matrix, projection matrix, etc) uniforms should be a third unique bind group.
    /// - Per-Scene(sun light, ambient light, time, etc) uniforms should be a fourth unique bind group.
    /// 
    /// The order of bind groups set by render pipeline should be the same as the order in shader.
    /// 
    /// But the bind group index defined in shader is decided by users causually.
    /// Our [`Shader`] API will parse their bind group index using `naga. And we will set the bind groups in the order defined by shader.
    /// 
    /// And some bind group can be skipped by user's shader, e.g, when user's shader only uses Per-Material uniforms,
    /// this shader do not have to care about Per_Item, Per_Cameraor Per-Scene bind groups.
    fn render_pass(
        graphics_context: &GraphicsContext,
        texture_sampler_manager: &mut TextureSamplerManager,
        encoder: &mut CommandEncoder,
        surface_texture_view_ref: &wgpu::TextureView,
        camera_render_data: &CameraRenderData,
    ) {
        let load_op = if let Some(clear_color) = camera_render_data.clear_color {
            wgpu::LoadOp::Clear(clear_color.into())
        } else {
            wgpu::LoadOp::Load
        };

        texture_sampler_manager
            .ensure_depth_texture_valid(camera_render_data.depth_attachment);
        if camera_render_data.color_attchment != TextureHandle::INVALID {
            texture_sampler_manager
                .ensure_color_attachment_valid(camera_render_data.color_attchment);
        }

        let mut view_port = camera_render_data.view_port;
        let color_attachment_view = if camera_render_data.color_attchment == TextureHandle::INVALID
        {
            surface_texture_view_ref
        } else {
            let color_attachment = texture_sampler_manager
                .get_texture(&camera_render_data.color_attchment)
                .unwrap();
            view_port.x = 0.0;
            view_port.y = 0.0;
            view_port.z = color_attachment.size.width as f32;
            view_port.w = color_attachment.size.height as f32;
            &color_attachment.view.as_ref().unwrap().view
        };

        let depth_view: &wgpu::TextureView = texture_sampler_manager
            .get_texture_view_forcely(&camera_render_data.depth_attachment);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("imagic render pass desc"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_attachment_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_viewport(view_port.x, view_port.y, view_port.z, view_port.w, 0.0, 1.0);

        // draw opaque items.
        for render_item in &camera_render_data.opaque_item_data {
            Self::_render_item(&mut rpass, render_item, graphics_context);
        }
        // draw skybox items.
        if let Some(skybox_item) = &camera_render_data.skybox_item_data {
            Self::_render_item(&mut rpass, skybox_item, graphics_context);
        }
        // draw transparent items.
        for render_item in &camera_render_data.transparent_item_data {
            Self::_render_item(&mut rpass, render_item, graphics_context);
        }
    }

    fn _render_item(rpass: &mut RenderPass, render_item: &ItemRenderData, graphics_context: &GraphicsContext) {
        if let Some(render_pipeline) = graphics_context
                .render_pipelines
                .get(render_item.render_pipeline)
            {
                let bind_groups = &render_item.bind_group;
                rpass.set_pipeline(render_pipeline);
                for (index, bind_group_id) in bind_groups.iter().enumerate() {
                    let bind_group = graphics_context.bind_group_manager.get(bind_group_id);
                    if let Some(real_bind_group) = bind_group {
                        rpass.set_bind_group(index as u32, real_bind_group, &[]);
                    } else {
                        if cfg!(debug_assertions) {
                            log::warn!("Failed to get bind group: {}", bind_group_id);
                        }
                    }
                }
                let vertex_buffer_slice = graphics_context
                    .buffer_manager
                    .get_buffer_slice(&render_item.vertex_buffer);
                rpass.set_vertex_buffer(0, vertex_buffer_slice);
                let index_buffer_slice = graphics_context
                    .buffer_manager
                    .get_buffer_slice(&render_item.index_buffer.as_ref().unwrap());
                rpass.set_index_buffer(index_buffer_slice, render_item.index_format.into());
                rpass.draw_indexed(0..render_item.index_count, 0, 0..1);
            } else {
                if cfg!(debug_assertions) {
                    log::warn!("Failed to get render pipeline.");
                }
            }
    }

    pub(crate) fn render_item_to_rt_directly(
        graphics_context: & GraphicsContext,
        clear_color: Option<Color>,
        view_port: Vec4,
        color_attachment_view: &TextureView,
        depth_attachment_view: &TextureView,
        item_render_data: &ItemRenderData,
    ) {
        let mut encoder =
            graphics_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("lxy render command encoder desc render_to_rt_directly"),
                });
        let load_op = if let Some(clear_color) = clear_color {
            wgpu::LoadOp::Clear(clear_color.into())
        } else {
            wgpu::LoadOp::Load
        };
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("imagic render pass desc"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_attachment_view.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: load_op,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_attachment_view.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
    
            rpass.set_viewport(view_port.x, view_port.y, view_port.z, view_port.w, 0.0, 1.0);
            Self::_render_item(&mut rpass, item_render_data, graphics_context);
        }
        graphics_context.queue.submit(Some(encoder.finish()));
    }
}
