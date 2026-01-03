use crate::{
    behaviors::behavior_wrapper::BehaviorWrapper, core::LogicContext, graphics::{graphics_context::GraphicsContext,
        render_api::RenderAPI
    }, renderer::{frame_data::FrameRenderData, ui_renderer::UIRenderer}
};

/// The frame renderer of the engine.
/// 
/// It is responsible for rendering the frame.
pub struct FrameRenderer {
    pub(crate) frame_render_data: FrameRenderData,
    pub(crate) ui_renderer: Option<UIRenderer>,
}

impl FrameRenderer {
    pub(crate) fn new() -> Self {
        Self {
            frame_render_data: FrameRenderData::default(),
            ui_renderer: None,
        }
    }

    pub(crate) fn render(
        &mut self,
        logic_context: &mut LogicContext,
        graphics_context: & GraphicsContext,
        behavior_wrappers: &mut Vec<BehaviorWrapper>,
    ) {
        if let Ok(surface_texture) = graphics_context.surface.get_current_texture() {
            // TODO: 不需要每帧创建 surface_texture_view?
            let surface_texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.frame_render_data
                .camera_data
                .sort_by(|camera_data_a, camera_data_b| {
                    camera_data_a.priority.cmp(&camera_data_b.priority)
                });

            let surface_texture_view_ref = &surface_texture_view;
            for camera_render_data in &self.frame_render_data.camera_data {
                // TODO: support render texture
                RenderAPI::render(
                    graphics_context,
                    logic_context.texture_sampler_manager,
                    surface_texture_view_ref,
                    camera_render_data,
                );
            }
            if self.ui_renderer.is_none() {
                self.ui_renderer = Some(UIRenderer::new(
                    &graphics_context.device,
                    graphics_context.surface_config.format,
                    graphics_context.main_window().get_ref(),
                ));
            }
            if let Some(ui_renderer) = &mut self.ui_renderer {
                ui_renderer.draw(logic_context, graphics_context, surface_texture_view_ref, behavior_wrappers);
            }
            surface_texture.present();
        }
    }
}
