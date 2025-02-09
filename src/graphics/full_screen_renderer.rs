use wgpu::{TextureFormat, TextureView};

use crate::prelude::MaterialTrait;

use super::{GraphicsContext, RenderTexture, RenderTexture2D};

#[allow(unused)]
/// TODO: implement this convient method to render full screen with a given material.
/// I will use a big triangle to render the full screen, without vertex/index buffer supplied.
pub fn render_full_screen(graphics_context: &GraphicsContext, material: &Box<dyn MaterialTrait>, render_texture: Option<&RenderTexture2D>) {
    let mut encoder = graphics_context.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("full screen command encoder desc"),
    });

    let mut color_attachment_view: &TextureView;
    let mut color_attachment_format: TextureFormat;
    let mut depth_attachment_view: &TextureView;
    let mut depth_attachment_format: TextureFormat;
    if let Some(rt) = render_texture {
        color_attachment_view = &rt.get_color_attachment_views()[0];
        color_attachment_format = rt.get_color_attachment_format();
        depth_attachment_view = &rt.get_depth_attachment_views()[0];
        // depth_attachment_format = rt.get_dep
    }
}