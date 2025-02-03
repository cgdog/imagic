use egui::*;
use egui_wgpu::ScreenDescriptor;
use wgpu::{Device, Queue, StoreOp, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window as WindowWinit;

use crate::prelude::{GraphicsContext, ImagicAppTrait};

pub struct UIRenderer {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    ui_scale_factor: f32,
}

impl UIRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &WindowWinit,
    ) -> UIRenderer {
        let egui_context = Context::default();

        let egui_state = egui_winit::State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            // TextureFormat::Rgba8Unorm,
            output_depth_format,
            msaa_samples,
            // TODO: choose a better value for dithering.
            false,
        );

        UIRenderer {
            state: egui_state,
            renderer: egui_renderer,
            ui_scale_factor: 1.0,
        }
    }

    pub fn state(&self) -> &egui_winit::State {
        &self.state
    }

    pub fn set_ui_scale_factor(&mut self, ui_scale_factor: f32) {
        self.ui_scale_factor = ui_scale_factor;
    }

    pub fn handle_input(&mut self, window: &WindowWinit, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, &event);
    }

    pub fn draw(
        &mut self,
        graphics_context: & GraphicsContext,
        window: &WindowWinit,
        window_surface_view: &TextureView,
        app: &mut Box<dyn ImagicAppTrait>,
    ) {
        let device: &Device = graphics_context.get_device();
        let queue: &Queue = graphics_context.get_queue();

        let mut screen_descriptor = ScreenDescriptor {
            size_in_pixels: graphics_context.get_surface().get_cur_size(),
            pixels_per_point: window.scale_factor() as f32,
        };

        screen_descriptor.pixels_per_point *= self.ui_scale_factor;
        self.state
            .egui_ctx()
            .set_pixels_per_point(screen_descriptor.pixels_per_point);

        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.state.egui_ctx().run(raw_input, |_ui| {
            app.on_render_ui(&self.state.egui_ctx());
        });

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&device, &queue, *id, &image_delta);
        }

        let mut encoder = graphics_context
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("imagic render ui command encoder desc") });
        
        self.renderer
            .update_buffers(&device, &queue, &mut encoder, &tris, &screen_descriptor);
        
        // 创建一个新的作用域来确保 rpass 在 submit 之前被 drop
        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                label: Some("ui render pass"),
                occlusion_query_set: None,
            });
            // TODO: study forget_lifetime()
            let mut rpass = rpass.forget_lifetime();
            self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        } // rpass 在这里自动 drop

        graphics_context.submit(Some(encoder.finish()));

        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
