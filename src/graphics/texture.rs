use wgpu::{TextureView, TextureViewDescriptor};

use super::GraphicsContext;

pub struct Texture {
    texture: wgpu::Texture,
    view: Option<TextureView>,
    size: wgpu::Extent3d,
}

impl Texture {
    pub fn create(graphics_context: &GraphicsContext, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = graphics_context.get_device().create_texture(
            &wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("imagic_texture"),
                view_formats: &[],
            }
        );

        Self {
            texture,
            view: None,
            size,
        }
    }

    pub fn create_from_bytes(graphics_context: &GraphicsContext, buffer: &[u8], format: wgpu::TextureFormat) -> Self {
        let img = image::load_from_memory(buffer).unwrap();
        let img_rgba = img.to_rgba8();
        use image::GenericImageView;
        let dimensions = img.dimensions();
        let mut texture = Texture::create(graphics_context, dimensions.0, dimensions.1, format);
        texture.fill_content(graphics_context, &img_rgba);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        texture.view = Some(texture_view);
        texture
    }

    pub fn fill_content(&self, graphics_context: &GraphicsContext, content: &[u8]) {
        graphics_context.get_queue().write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &content,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.size.width),
                rows_per_image: Some(self.size.height),
            },
            self.size,
        );
    }

    pub fn create_view(&self, desc: &TextureViewDescriptor<'_>) -> TextureView {
        self.texture.create_view(desc)
    }

    pub fn get_texture_view(&self) -> &TextureView {
        &self.view.as_ref().expect("Texture view is not created!")
    }
}