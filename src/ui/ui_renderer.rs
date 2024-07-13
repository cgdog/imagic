use egui::*;
use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use winit::event::WindowEvent;
use winit::window::Window as WindowWinit;

use crate::graphics::GraphicsContext;

pub struct UIRenderer {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    ui_scale_factor: f32,
    ui_drawer: Option<Box<dyn Fn(&Context)>>
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
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            // TextureFormat::Rgba8Unorm,
            output_depth_format,
            msaa_samples,
        );

        UIRenderer {
            state: egui_state,
            renderer: egui_renderer,
            ui_scale_factor: 1.0,
            ui_drawer: None,
        }
    }

    pub fn set_ui_drawer(&mut self, ui_drawer: Option<Box<dyn Fn(&Context)>>) {
        self.ui_drawer = ui_drawer;
    }

    pub fn set_ui_scale_factor(&mut self, ui_scale_factor: f32) {
        self.ui_scale_factor = ui_scale_factor;
    }

    pub fn handle_input(&mut self, window: &WindowWinit, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, &event);
    }

    pub fn draw(
        &mut self,
        graphics_context: &GraphicsContext,
        encoder: &mut CommandEncoder,
        window: &WindowWinit,
        window_surface_view: &TextureView,
    ) {
        if self.ui_drawer.is_none() {
            return;
        }

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
            self.ui_drawer.as_ref().unwrap()(&self.state.egui_ctx());
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
        self.renderer
            .update_buffers(&device, &queue, encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
